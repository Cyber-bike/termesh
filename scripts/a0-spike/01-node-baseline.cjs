// A0 步骤 1：纯 Node 基线（不涉及 Electron）。
//
// 验证 @number0/iroh 的本平台预编译 .node 能加载，且两个本机端点能
// 完成一次真实 QUIC 连接 + 连接码字符串往返。Linux 上已于 2026-07-31
// 验证通过；在 Windows 上跑它验证的是 win32-x64-msvc 预编译包。
//
// 运行：cd scripts/a0-spike && npm install && node 01-node-baseline.cjs
// 预期：最后一行输出 "A0 NODE BASELINE: OK"，退出码 0。

const { Endpoint, EndpointTicket, RelayMode, presetMinimal } = require('@number0/iroh');

const ALPN = Array.from(Buffer.from('termy-a0-baseline/1'));

async function bindLoopback() {
  const builder = Endpoint.builder();
  presetMinimal(builder);
  builder.relayMode(RelayMode.disabled());
  builder.alpns([ALPN]);
  builder.bindAddr('127.0.0.1:0');
  return builder.bind();
}

async function main() {
  const server = await bindLoopback();
  const client = await bindLoopback();

  const ticketStr = EndpointTicket.fromAddr(server.addr()).toString();
  console.log('connection code:', ticketStr);
  const parsedAddr = EndpointTicket.fromString(ticketStr).endpointAddr();
  if (!parsedAddr.id().equals(server.addr().id())) {
    throw new Error('ticket round-trip changed the EndpointId');
  }

  const acceptSide = (async () => {
    const incoming = await server.acceptNext();
    const conn = await (await incoming.accept()).connect();
    const bi = await conn.acceptBi();
    const got = Buffer.from(await bi.recv.readExact(5)).toString();
    await bi.send.writeAll(Array.from(Buffer.from(got.toUpperCase())));
    await bi.send.finish();
  })();

  const conn = await client.connect(parsedAddr, ALPN);
  const bi = await conn.openBi();
  await bi.send.writeAll(Array.from(Buffer.from('hello')));
  await bi.send.finish();
  const reply = Buffer.from(await bi.recv.readExact(5)).toString();
  if (reply !== 'HELLO') throw new Error(`echo mismatch: ${reply}`);

  await acceptSide;
  await client.close();
  await server.close();
  console.log('A0 NODE BASELINE: OK');
}

const watchdog = setTimeout(() => {
  console.error('A0 NODE BASELINE: TIMED OUT after 30s');
  process.exit(1);
}, 30_000);

main()
  .then(() => { clearTimeout(watchdog); process.exit(0); })
  .catch((e) => {
    console.error('A0 NODE BASELINE: FAILED:', e);
    process.exit(1);
  });
