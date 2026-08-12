# 交接：Windows 验证清单（A0 + Agent 验收）

> 更新：2026-07-31（第 2 版）· 分支 `v2.0` · 配套脚本在 `scripts/a0-spike/`

## 0. 现状：已证明什么、还差什么

**已完成、无需重复验证的：**

- 沙箱（Linux）：Agent 全链路（`run --loopback` 打码、贴码建连、帧协议握手、真 shell 会话、多会话并发、断连杀进程树、单控制端限制），`cargo test` 64/64；插件侧传输层（`terminalStreamTransport` + `irohStreams` 适配器）经真实 `@number0/iroh` 对真实 agent 全栈联通（FULL STACK INTEGRATION: OK）。
- **步骤 1 已过**（2026-07-31，Windows）：`01-node-baseline.cjs` 输出 `A0 NODE BASELINE: OK`——win32-x64-msvc 预编译包可用。
- **步骤 2 已过（A0 核心结论：直接嵌入可行）**（2026-07-31，Windows 真实 Obsidian）：Electron 渲染进程内 require 成功（25 个导出符号），绑定 loopback Endpoint 并生成连接码成功。termy-bridge 兜底不再需要。

**剩余待验证（本清单只剩步骤 3、4、5）：**

| # | 问题 | 对应任务 |
| --- | --- | --- |
| 1 | Windows 版 agent 测试套件是否全绿（尤其 Job Object 进程树终止） | 步骤 3 |
| 2 | Windows agent × JS 假控制端是否端到端互通 | 步骤 4 |
| 3 | esbuild 打包/插件目录布局下模块能否加载（决定打包脚本怎么改） | 步骤 5 |

> 前置：执行者的仓库必须包含 `v2.0` 分支最新提交（含 `agent/src/serve.rs`、`scripts/a0-spike/`）。若远程还没有这些提交，先完成推送再开始。

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

预期末行 `A0 NODE BASELINE: OK`。**（2026-07-31 已通过，无需重跑。）**

### 步骤 2：Obsidian/Electron 加载（A0 核心，约 5 分钟）——已通过

**（2026-07-31 已通过：模块加载 25 个导出符号、Endpoint 绑定与连接码生成均成功，结论=直接嵌入，无需重跑。）** 如需复现：打开 `scripts/a0-spike/02-obsidian-console.js`，按文件头部注释操作。注意从聊天/终端复制代码可能因折行混入换行符导致 `SyntaxError`，长字符串路径要保持单行。

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

执行完把结果写进 `交接结果 - Windows.md`（新建，格式如下表）并提交到 `v2.0` 分支：

| 步骤 | 结果 (OK / 报错原文) | 备注 |
| --- | --- | --- |
| 1 Node 基线 | OK（2026-07-31，人工执行） | |
| 2 Electron 加载 | OK（2026-07-31，人工执行，25 符号 + 连接码生成成功） | A0 结论=直接嵌入 |
| 3 cargo test（含失败用例名） | | |
| 4 假控制端 × Windows agent | | |
| 5 打包态加载 | | |

- 步骤 5 OK → 打包方案定型为 external + 插件目录携带 node_modules，拷贝逻辑进 `scripts/package-plugin.js`。
- 步骤 5 失败而报错非 MODULE_NOT_FOUND → 记录报错原文；备选方案是 bundle loader + 仅 external `.node` 文件。
- 步骤 3/4 的 Windows 特有问题（如 Job Object 用例失败）→ 记录用例名与输出原文，单独修，不阻塞其他工作。

## 4. 并行进行中的（不依赖上表）

- ~~控制端 `TerminalTransport` 的 v2.0 实现~~（已完成：`terminalStreamTransport.ts` + `irohStreams.ts`，全栈联通已验证）
- 设备连接管理（`irohClient`：endpoint 单例、设备连接生命周期）与设备列表 / 添加设备 UI 接线
- V1 遗留 TS 模块清理（`authClient`/`deviceClient`/`relayClient` 等）
