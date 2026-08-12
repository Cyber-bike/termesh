// A0 步骤 2：Electron 渲染进程加载验证——A0 真正要回答的问题。
//
// 用法：
//   1. 先在本目录 `npm install`（脚本 01 已经装过就不用了）。
//   2. 打开 Obsidian -> Ctrl+Shift+I 打开开发者工具 -> Console。
//   3. 把下面 REQUIRE_PATH 改成你机器上 a0-spike/node_modules 里
//      @number0/iroh 的绝对路径（Windows 注意用双反斜杠或正斜杠）。
//   4. 整段复制粘贴到 Console 回车。
//
// 判定：
//   - 打印 "A0 ELECTRON: OK" + 一个 endpoint 开头的连接码 => 通过，
//     结论 = 直接嵌入路径可行（实现方案 §6.2）。
//   - require 抛错（找不到模块 / NODE_MODULE_VERSION 不匹配 / DLL
//     加载失败）=> 记录完整报错，结论 = 走 termy-bridge 兜底路径。
//
// 原理说明：Obsidian 渲染进程开着 nodeIntegration，require 一个
// N-API 原生扩展理论上可行（N-API 是跨 Node/Electron ABI 稳定的），
// 但 Electron 对原生模块的实际行为必须实测——这正是 A0 存在的意义。

(async () => {
  // 改成你机器上的真实路径！在 scripts/a0-spike 目录里运行
  //   node -e "console.log(require.resolve('@number0/iroh'))"
  // 把它打印的绝对路径填到这里（反斜杠换成 / 或 \\）。
  const REQUIRE_PATH = 'C:/ReqFirst/scripts/a0-spike/node_modules/@number0/iroh/index.js';

  let iroh;
  try {
    iroh = require(REQUIRE_PATH);
  } catch (e) {
    if (e.code === 'MODULE_NOT_FOUND') {
      console.error(
        'A0 ELECTRON: 路径上没找到模块——这不是 A0 失败，是 REQUIRE_PATH 不对或还没 npm install。' +
        '按文件头部注释用 require.resolve 拿到真实路径后重试。'
      );
    } else {
      console.error('A0 ELECTRON: REQUIRE FAILED ——记录下面的完整报错，结论=termy-bridge 兜底');
    }
    throw e;
  }
  console.log('module loaded, exports:', Object.keys(iroh).length, 'symbols');

  const { Endpoint, EndpointTicket, RelayMode, presetMinimal } = iroh;
  const builder = Endpoint.builder();
  presetMinimal(builder);
  builder.relayMode(RelayMode.disabled());
  builder.alpns([Array.from(new TextEncoder().encode('termy-a0-electron/1'))]);
  builder.bindAddr('127.0.0.1:0');

  const endpoint = await builder.bind();
  const code = EndpointTicket.fromAddr(endpoint.addr()).toString();
  console.log('connection code:', code);
  await endpoint.close();

  console.log('A0 ELECTRON: OK ——结论=直接嵌入可行');
})();
