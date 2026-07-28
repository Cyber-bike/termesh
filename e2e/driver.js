'use strict';

/**
 * Stands in for the Obsidian plugin: opens a control WSS, runs a real command in
 * the remote shell, then pushes a note plus an attachment through the transfer
 * protocol and checks both landed on disk.
 */

const WebSocket = require('ws');
const crypto = require('crypto');
const fs = require('fs');

const BASE = process.env.BASE || 'http://127.0.0.1:18090';
const TOKEN = process.env.TOKEN;
const DEVICE_ID = process.env.DEVICE_ID;
const RECV = process.env.RECV || '/tmp/e2e/recv';

const HEADER_BYTES = 38;
const KIND_INPUT = 0x01;
const KIND_OUTPUT = 0x02;
const KIND_FILE = 0x03;

function encodeFrame(kind, streamUuid, fileIndex, offset, payload) {
  const header = Buffer.alloc(HEADER_BYTES);
  header[0] = 0x54;
  header[1] = 0x4d;
  header[2] = 0x01;
  header[3] = kind;
  header[4] = 0;
  header[5] = 0;
  header.writeUInt32BE(payload.length, 6);
  Buffer.from(streamUuid.replace(/-/g, ''), 'hex').copy(header, 10);
  header.writeUInt32BE(kind === KIND_FILE ? fileIndex : 0xffffffff, 26);
  header.writeBigUInt64BE(BigInt(offset), 30);
  return Buffer.concat([header, payload]);
}

function decodeFrame(buf) {
  return {
    kind: buf[3],
    payloadLength: buf.readUInt32BE(6),
    offset: buf.readBigUInt64BE(30),
    payload: buf.subarray(HEADER_BYTES),
  };
}

const results = [];
const check = (name, ok, detail = '') => {
  results.push({ name, ok, detail });
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ' :: ' + detail : ''}`);
};

function envelope(type, payload, extra = {}) {
  return JSON.stringify({
    protocolVersion: 1,
    type,
    requestId: extra.requestId ?? null,
    deviceId: DEVICE_ID,
    sessionId: extra.sessionId ?? null,
    payload,
  });
}

async function main() {
  const ws = new WebSocket(`${BASE.replace('http', 'ws')}/v1/control/ws`, 'termy.v1', {
    headers: { authorization: `Bearer ${TOKEN}` },
  });

  // Messages are never discarded: a waiter that times out must not be able to
  // swallow a later message meant for someone else.
  const inbox = [];
  let pending = [];

  const pump = () => {
    pending = pending.filter((waiter) => {
      const hit = inbox.findIndex(waiter.predicate);
      if (hit < 0) return true;
      clearTimeout(waiter.timer);
      waiter.resolve(inbox.splice(hit, 1)[0]);
      return false;
    });
  };

  const deliver = (msg) => {
    inbox.push(msg);
    pump();
  };

  const next = (predicate, label, timeoutMs = 15000) =>
    new Promise((resolve, reject) => {
      const waiter = { predicate, resolve };
      waiter.timer = setTimeout(() => {
        pending = pending.filter((w) => w !== waiter);
        reject(new Error(`timed out waiting for ${label}`));
      }, timeoutMs);
      pending.push(waiter);
      pump();
    });

  ws.on('message', (data, isBinary) => {
    deliver(isBinary ? { binary: decodeFrame(data) } : JSON.parse(data.toString()));
  });
  ws.on('error', (e) => {
    console.error('control socket error', e.message);
    process.exit(1);
  });

  await new Promise((resolve) => ws.on('open', resolve));

  // --- terminal ------------------------------------------------------------
  const openId = crypto.randomUUID();
  ws.send(envelope('terminal.open', { cols: 100, rows: 30 }, { requestId: openId }));

  const opened = await next((m) => m.type === 'terminal.opened', 'terminal.opened');
  check('terminal opens on the remote host', !!opened.sessionId, `shell=${opened.payload.shell}`);
  const sessionId = opened.sessionId;

  const marker = `TERMY-E2E-${Date.now()}`;
  ws.send(encodeFrame(KIND_INPUT, sessionId, 0, 0, Buffer.from(`echo ${marker}\n`)));

  let seen = '';
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline && !seen.includes(marker + '\r\n')) {
    const frame = await next((m) => m.binary && m.binary.kind === KIND_OUTPUT, 'terminal output');
    seen += frame.binary.payload.toString('utf8');
  }
  check('a real command runs and its output comes back', seen.includes(marker));

  const shellEvents = [];
  const drainUntil = Date.now() + 1500;
  while (Date.now() < drainUntil) {
    try {
      const m = await next((x) => x.type === 'terminal.shellEvent', 'shellEvent', 400);
      shellEvents.push(m.payload.type);
    } catch {
      break;
    }
  }
  console.log(`      (shell integration events observed: ${shellEvents.join(', ') || 'none'})`);

  // --- transfer ------------------------------------------------------------
  const transferId = crypto.randomUUID();
  const note = Buffer.from(`# Demo\n\n![img](assets/pic.bin)\n\n${marker}\n`);
  const attachment = crypto.randomBytes(300 * 1024);

  ws.send(
    envelope(
      'transfer.start',
      {
        transferId,
        rootNote: 'notes/demo.md',
        entries: [
          { index: 0, relativePath: 'notes/demo.md', size: note.length },
          { index: 1, relativePath: 'assets/pic.bin', size: attachment.length },
        ],
      },
      { requestId: crypto.randomUUID() }
    )
  );

  const accepted = await next((m) => m.type === 'transfer.accepted', 'transfer.accepted');
  check('transfer is accepted with an initial credit', accepted.payload.grantedBytes > 0,
    `grantedBytes=${accepted.payload.grantedBytes}`);

  let granted = accepted.payload.grantedBytes;
  let sent = 0;
  const sendFile = async (index, buf) => {
    const CHUNK = 256 * 1024;
    for (let offset = 0; offset < buf.length; offset += CHUNK) {
      const slice = buf.subarray(offset, Math.min(offset + CHUNK, buf.length));
      while (sent + slice.length > granted) {
        const credit = await next((m) => m.type === 'transfer.credit', 'transfer.credit');
        granted = Math.max(granted, credit.payload.grantedBytes);
      }
      ws.send(encodeFrame(KIND_FILE, transferId, index, offset, slice));
      sent += slice.length;
    }
    ws.send(envelope('transfer.fileEnd', { transferId, fileIndex: index, sentSize: buf.length }));
  };

  await sendFile(0, note);
  await sendFile(1, attachment);
  ws.send(envelope('transfer.complete', { transferId }));

  const result = await next((m) => m.type === 'transfer.result', 'transfer.result');
  check('transfer reports success', result.payload.success === true,
    result.payload.message || result.payload.code || '');

  const notePath = `${RECV}/notes/demo.md`;
  const attachPath = `${RECV}/assets/pic.bin`;
  check('note landed at its vault-relative path', fs.existsSync(notePath));
  check('attachment landed at its vault-relative path', fs.existsSync(attachPath));
  if (fs.existsSync(notePath)) {
    check('note content matches byte for byte', fs.readFileSync(notePath).equals(note));
  }
  if (fs.existsSync(attachPath)) {
    check('attachment content matches byte for byte',
      fs.readFileSync(attachPath).equals(attachment));
  }

  // --- teardown ------------------------------------------------------------
  ws.send(envelope('terminal.close', { reason: 'user', exitCode: null }, { sessionId }));
  await new Promise((r) => setTimeout(r, 500));
  ws.close();

  const failed = results.filter((r) => !r.ok);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length === 0 ? 0 : 1);
}

main().catch((e) => {
  console.error('driver failed:', e.message);
  process.exit(1);
});
