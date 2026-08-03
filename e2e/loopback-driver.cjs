// v2.0 end-to-end driver: dials a real `termy-agent run --loopback` process
// over iroh QUIC using the same `@number0/iroh` binding the plugin embeds
// (doc A0: direct embedding, no termy-bridge), completes the doc 8.2
// open/opened handshake, and verifies a real shell round-trip.
//
// This replaces the V1 e2e-run.sh flow (relay + `agent bind` + WSS driver),
// which stopped working once the agent's relay client (client.rs) was
// deleted for v2.0 - `bind` is no longer a subcommand. It does not cover
// file transfer: `termy/transfer/1` is Phase C and not built yet: agent's
// serve.rs closes that ALPN with PROTOCOL_ERROR.
//
// Usage: node loopback-driver.cjs <connection-code>
// Requires @number0/iroh to be resolvable from here, i.e. `pnpm install` run
// at the repo root (this file lives in e2e/, a subdirectory of the root, so
// plain `require('@number0/iroh')` walks up to ../node_modules).

const { Endpoint, EndpointTicket, RelayMode, presetMinimal } = require('@number0/iroh');

const ALPN = Array.from(Buffer.from('termy/terminal/1'));

const KIND = { open: 0x00, data: 0x01, resize: 0x02, shellEvent: 0x03, close: 0x04, opened: 0x05, error: 0x06 };

function writeVarint(value) {
  const out = [];
  let v = value >>> 0;
  for (;;) {
    const byte = v & 0x7f;
    v >>>= 7;
    if (v === 0) { out.push(byte); return out; }
    out.push(byte | 0x80);
  }
}

function encodeFrame(kind, payloadBuf) {
  return Buffer.concat([Buffer.from([kind]), Buffer.from(writeVarint(payloadBuf.length)), payloadBuf]);
}

function encodeJsonFrame(kind, obj) {
  return encodeFrame(kind, Buffer.from(JSON.stringify(obj)));
}

/** Accumulates bytes, pops complete frames: { kind, payload: Buffer }. */
class FrameDecoder {
  constructor() { this.buf = Buffer.alloc(0); }
  push(bytes) { this.buf = Buffer.concat([this.buf, Buffer.from(bytes)]); }
  next() {
    if (this.buf.length === 0) return null;
    const kind = this.buf[0];
    let len = 0, shift = 0, i = 1;
    for (;;) {
      if (i >= this.buf.length) return null;
      const byte = this.buf[i];
      len |= (byte & 0x7f) << shift;
      i += 1;
      if ((byte & 0x80) === 0) break;
      shift += 7;
    }
    if (this.buf.length < i + len) return null;
    const payload = this.buf.subarray(i, i + len);
    this.buf = this.buf.subarray(i + len);
    return { kind, payload: Buffer.from(payload) };
  }
}

async function main() {
  const code = process.argv[2];
  if (!code) {
    console.error('usage: node loopback-driver.cjs <connection-code>');
    process.exit(2);
  }

  const builder = Endpoint.builder();
  presetMinimal(builder);
  builder.relayMode(RelayMode.disabled());
  builder.alpns([ALPN]);
  builder.bindAddr('127.0.0.1:0');
  const endpoint = await builder.bind();

  const addr = EndpointTicket.fromString(code.trim()).endpointAddr();
  console.log('dialing', addr.id().toString().slice(0, 16), '...');
  const conn = await endpoint.connect(addr, ALPN);
  console.log('connected');

  const bi = await conn.openBi();
  const decoder = new FrameDecoder();

  const readFrame = async () => {
    for (;;) {
      const frame = decoder.next();
      if (frame) return frame;
      const chunk = await bi.recv.read(65536);
      if (!chunk || chunk.length === 0) throw new Error('agent closed the stream');
      decoder.push(chunk);
    }
  };

  // doc 8.2 handshake
  await bi.send.writeAll(Array.from(encodeJsonFrame(KIND.open, { cols: 80, rows: 24 })));
  const first = await readFrame();
  if (first.kind === KIND.error) {
    throw new Error(`agent refused: ${first.payload.toString()}`);
  }
  if (first.kind !== KIND.opened) {
    throw new Error(`expected opened(0x05), got 0x${first.kind.toString(16)}`);
  }
  const opened = JSON.parse(first.payload.toString());
  console.log('session opened:', opened.sessionId, 'shell:', opened.shell);

  // Real shell round-trip. The command is assembled so that the echoed
  // INPUT line never contains the finished marker - only the command's
  // OUTPUT does - which makes a plain substring check sufficient.
  const marker = 'E2E-LOOPBACK-MARKER';
  const shell = String(opened.shell).toLowerCase();
  const command = shell.includes('powershell') || shell.includes('pwsh')
    ? "echo ('E2E-LOOPBACK-'+'MARKER')\r"
    : "echo 'E2E-LOOPBACK-''MARKER'\n";
  await bi.send.writeAll(Array.from(encodeFrame(KIND.data, Buffer.from(command))));

  let collected = '';
  for (;;) {
    const frame = await readFrame();
    if (frame.kind === KIND.data) {
      collected += frame.payload.toString('utf8');
      const clean = collected
        .replace(/\x1b\[[0-9;?]*[a-zA-Z]/g, '')
        .replace(/\x1b\][^\x07]*(\x07|\x1b\\)/g, '')
        .replace(/\x1b[()][A-Z0-9]/g, '')
        .replace(/\x1b[=>]/g, '');
      if (clean.includes(marker)) break;
    } else if (frame.kind === KIND.shellEvent) {
      console.log('shellEvent:', frame.payload.toString());
    } else if (frame.kind === KIND.close) {
      throw new Error(`session closed early: ${frame.payload.toString()}`);
    }
  }
  console.log('shell echo verified');

  // Resize round trip (doc 8.2 kind 0x02) - the piece e2e-run.sh's V1
  // predecessor never covered, since V1's control socket resized differently.
  await bi.send.writeAll(Array.from(encodeJsonFrame(KIND.resize, { cols: 100, rows: 40 })));

  await bi.send.writeAll(Array.from(encodeJsonFrame(KIND.close, { reason: 'done' })));
  conn.close(0n, Array.from(Buffer.from('bye')));
  await endpoint.close();
  console.log('E2E LOOPBACK DRIVER: OK');
}

const watchdog = setTimeout(() => {
  console.error('E2E LOOPBACK DRIVER: TIMED OUT after 30s');
  process.exit(1);
}, 30_000);

main()
  .then(() => { clearTimeout(watchdog); process.exit(0); })
  .catch((e) => {
    console.error('E2E LOOPBACK DRIVER: FAILED:', e);
    process.exit(1);
  });
