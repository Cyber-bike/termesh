// A0 步骤 3：假控制端——用 JS binding 连真实的 Rust agent。
//
// 这是插件网络层落地前的端到端替身：贴入 `termesh-agent run --loopback`
// 打印的连接码，脚本会以 termy/terminal/1 ALPN 建连、完成 doc 8.2 的
// open/opened 握手、在真实 shell 里执行 echo 并校验回显，最后正常关闭。
// 通过即证明：JS binding 与 Rust agent 的 QUIC + 帧协议互通。
//
// 运行（先在另一个窗口启动 agent）：
//   termesh-agent run --loopback
//   cd scripts/a0-spike && npm install
//   node 03-fake-controller.cjs <连接码>
// 预期：输出 "FAKE CONTROLLER: OK"，退出码 0。
//
// 帧格式（doc 8.2，与 agent/src/termstream.rs、
// src/services/remote/terminalStreamFrame.ts 一致）：
//   kind(1B) + length(LEB128 varint) + payload
//   0x00 open  0x01 data  0x02 resize  0x03 shellEvent
//   0x04 close 0x05 opened 0x06 error

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
    console.error('usage: node 03-fake-controller.cjs <connection-code>');
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
  // Line ending is \r (a pty's Enter): bash maps CR to NL via ICRNL, and
  // PowerShell's PSReadLine only submits on CR - a bare \n is swallowed,
  // which is what stalled the 2026-07-31 Windows run.
  const marker = 'A0-FC-MARKER';
  const shell = String(opened.shell).toLowerCase();
  const command = shell.includes('powershell') || shell.includes('pwsh')
    ? "echo ('A0-FC-'+'MARKER')\r"
    : "echo 'A0-FC-''MARKER'\r";
  await bi.send.writeAll(Array.from(encodeFrame(KIND.data, Buffer.from(command))));

  let collected = '';
  for (;;) {
    const frame = await readFrame();
    if (frame.kind === KIND.data) {
      collected += frame.payload.toString('utf8');
      // Strip terminal escapes (CSI, OSC, two-byte ESC sequences) so a
      // sequence landing mid-marker cannot hide the match.
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

  await bi.send.writeAll(Array.from(encodeJsonFrame(KIND.close, { reason: 'done' })));
  conn.close(0n, Array.from(Buffer.from('bye')));
  await endpoint.close();
  console.log('FAKE CONTROLLER: OK');
}

const watchdog = setTimeout(() => {
  console.error('FAKE CONTROLLER: TIMED OUT after 30s');
  process.exit(1);
}, 30_000);

main()
  .then(() => { clearTimeout(watchdog); process.exit(0); })
  .catch((e) => {
    console.error('FAKE CONTROLLER: FAILED:', e);
    process.exit(1);
  });
