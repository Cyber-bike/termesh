# 交接：Windows 验证清单（A0 + Agent 验收）

> 更新：2026-07-31 · 分支 `v2.0` · 配套脚本在 `scripts/a0-spike/`

## 0. 现状：已证明什么、还差什么

**沙箱（Linux, 无 GUI）已实测通过、无需重复验证的：**

- Agent 侧完整跑通：`termy-agent run --loopback` 免账号免配置打印连接码，真实回环 QUIC 上完成贴码建连、`termy/terminal/1` 帧协议握手、真 shell 会话（echo 回显 / resize 生效 / 多会话并发 / 退出码回传 / 断连杀进程树 / `CONTROLLER_ALREADY_CONNECTED` 单控制端限制）。`cargo test` 64/64。
- `@number0/iroh` 1.1.0（插件将嵌入的 JS binding）在**纯 Node** 下可加载、可建连；假控制端脚本（`03-fake-controller.cjs`）已对真实 Rust agent 走通全链路（QUIC + 帧协议 + 真 bash 回显）。

**只能在你那边（Windows + 真实 Obsidian）回答的，按优先级：**

| # | 问题 | 对应任务 |
| --- | --- | --- |
| 1 | `@number0/iroh` 能否在 **Obsidian 的 Electron 渲染进程**里 require 成功 | A0 核心（步骤 2） |
| 2 | Windows 平台的预编译包 + **Windows 版 agent** 是否同样全链路互通 | 步骤 1、3、4 |
| 3 | esbuild 打包/插件目录布局下模块能否加载 | 步骤 5 |

## 1. 准备

```powershell
git clone <repo> ; cd ReqFirst ; git checkout v2.0
# Rust（rust-toolchain.toml 固定 1.97.1，rustup 会自动装）
cd agent ; cargo build ; cd ..
# 脚本依赖（独立于插件的 package.json，不动 pnpm-lock）
cd scripts/a0-spike ; npm install ; cd ../..
```

## 2. 逐步执行

### 步骤 1：Node 基线（2 分钟）

```powershell
cd scripts/a0-spike
node 01-node-baseline.cjs
```

预期末行 `A0 NODE BASELINE: OK`。失败即说明 win32-x64-msvc 预编译包有问题——后面不用做了，直接把报错发我。

### 步骤 2：Obsidian/Electron 加载（A0 核心，约 5 分钟）

打开 `scripts/a0-spike/02-obsidian-console.js`，按文件头部注释操作（改 `REQUIRE_PATH` → Obsidian 开发者工具 Console 里粘贴整段）。

- `A0 ELECTRON: OK` → **直接嵌入路径成立**，这是 v2.0 网络层的理想路径。
- require 报错（典型：`NODE_MODULE_VERSION` 不匹配、DLL 初始化失败）→ **termy-bridge 兜底路径**。把完整报错发我。

### 步骤 3：Windows agent 测试套件（约 5 分钟）

```powershell
cd agent ; cargo test
```

重点关注（这些在 Linux 全过、但 Windows 路径从未跑过）：

- `pty::` 系列——Job Object 进程树终止（`pty.rs` 里明确注释过 "Untested on this build host"）
- `serve::` 三条集成测试——注意其中 `dropping_the_connection_kills_every_session_process_tree` 是 `#[cfg(unix)]`，Windows 上不会跑；Windows 的等价验证靠步骤 4 手工确认

### 步骤 4：Windows agent × JS 假控制端（端到端，约 5 分钟）

窗口 A：

```powershell
.\agent\target\debug\termy-agent.exe run --loopback
# 记下打印的 endpoint 开头的连接码；默认 shell 是 powershell.exe
```

窗口 B：

```powershell
cd scripts/a0-spike
node 03-fake-controller.cjs <连接码>
```

预期 `FAKE CONTROLLER: OK`。注意：脚本里发的是 `echo <marker>`，PowerShell 下同样可用。顺手验证：

- 窗口 A `Ctrl-C` 能干净退出
- `termy-agent.exe status` 运行中能显示连接码
- 第二个 `run` 实例被单实例锁拒绝

### 步骤 5：打包态加载（A0 的另一半，约 15 分钟）

上面步骤 2 验证的是"Electron 能不能加载"；这一步验证"**插件实际分发布局**下能不能加载"。Obsidian 插件目录没有 node_modules，方案是 external + 随插件目录携带：

1. `esbuild.config.mjs` 的 `external` 数组加一项 `'@number0/iroh'`（先本地改，不用提交）。
2. `pnpm build`（或 `pnpm dev`）+ 你平时的安装方式（`pnpm install:dev`）装进测试 vault。
3. 把 `scripts/a0-spike/node_modules/@number0/`（`iroh` + `iroh-win32-x64-msvc` 两个包）整体拷到 `<vault>/.obsidian/plugins/termy/node_modules/@number0/`。
4. 重启 Obsidian，Console 里执行：`require('<vault绝对路径>/.obsidian/plugins/termy/node_modules/@number0/iroh')` 后重复步骤 2 的绑定/打码片段。

通过 = 分发布局可行（后续我会把这个拷贝动作写进 `scripts/package-plugin.js`）。若 require 在这种布局下失败而步骤 2 成功，也记录报错——那是打包问题不是 Electron 问题，有别的解法（bundle loader + external `.node`）。

## 3. 结果回填

做完把下表填好发我即可（截图/粘贴输出都行），我据此继续：

| 步骤 | 结果 (OK / 报错原文) |
| --- | --- |
| 1 Node 基线 | |
| 2 Electron 加载 | |
| 3 cargo test（含失败用例名） | |
| 4 假控制端 × Windows agent | |
| 5 打包态加载 | |

- 步骤 2+5 都 OK → 我按**直接嵌入**写 `irohClient.ts`（真实 binding 接线）并把拷贝逻辑进打包脚本；A0 正式关闭，更新实现方案 §6.2 与开发计划。
- 步骤 2 失败 → 我按 **termy-bridge** 兜底路径开工（本机回环通道 + 身份校验，实现方案 §6.2 预案），排期 +2~3 天（计划 §6 已预留）。
- 步骤 3/4 的 Windows 特有问题 → 单独修，不阻塞 A0 结论。

## 4. 我这边并行继续的（不依赖上表）

- 控制端 `TerminalTransport` 的 v2.0 实现（对帧协议写，假流单测，A0 出结论后只换底层流对象）
- 设备列表 / 添加设备 UI 接线（`pairedDeviceStore`/`devicePairing` 已就绪）
- V1 遗留 TS 模块清理（`authClient`/`deviceClient`/`relayClient` 等）
