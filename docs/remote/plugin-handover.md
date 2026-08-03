# 插件端 v2.0 UI 接线交接

写给接手"把已经写好的 v2.0 远程终端后端模块接进真实 Obsidian UI"这件事的人。

> 本文件的上一版内容是 V1（账号 + 云端 Relay）插件端改造的交接材料，那份工作**已经完成并合入仓库**（`remoteService.ts`/`relayClient.ts`/`terminalService.ts` 等现在就是那次工作的产物）。本文是它的 v2.0 续篇，场景不同：这次不是"从零实现协议接入"，而是"把已经写完、测过的纯逻辑模块，接到 Obsidian 的设置面板和命令面板上"。

---

## 1. 现状

### 已完成且已验证（不要重写，直接用）

后端（`agent/`）和插件端纯逻辑（`src/services/remote/`）已经全部写完，覆盖真实 iroh QUIC 通信，不是占位实现：

| 位置 | 内容 | 验证方式 |
| --- | --- | --- |
| `agent/src/p2p.rs` | iroh Endpoint 绑定、连接码生成/解析、`ControllerGate`（单控制端占用） | `cargo test --manifest-path agent/Cargo.toml`，含真实回环 QUIC 集成测试 |
| `agent/src/serve.rs` | 连接分发、doc 8.2 帧握手、多会话 PTY 服务循环 | 同上，含真实 shell 会话、resize、并发会话、连接断开清理进程树的集成测试 |
| `agent/src/session_table.rs` | 多会话表，`maxConcurrentSessions` 软上限 | 同上 |
| `src/services/remote/connectionCode.ts` | `checkConnectionCode()` 连接码格式预校验 | `pnpm test:remote` |
| `src/services/remote/pairedDeviceStore.ts` | 已配对设备列表管理，已接入 `TerminalSettings.pairedDevices` 持久化 | 同上，已在 `main.ts` 的 `loadSettings`/`saveSettings` 里接好 |
| `src/services/remote/devicePairing.ts` | `pairDevice()`：粘贴 -> 预校验 -> 权威解析 -> 入库 | 同上 |
| `src/services/remote/irohStreams.ts` | `IrohModule` 类型镜像（真实 `@number0/iroh` 1.1.0 API 的结构化类型，不 import 包本身）、`byteStreamFromBi`、`terminalStreamFactory` | 同上 |
| `src/services/remote/terminalStreamTransport.ts` | `TerminalStreamTransport`，实现 `transport.ts` 的 `TerminalTransport` 四通道接口 | 同上 |
| `src/services/remote/deviceConnections.ts` | `DeviceConnectionManager`：endpoint 单例、控制端身份生命周期、`connect`/`disconnect`/`status`/`onDidChange`/`createTerminalTransport` | 同上 |
| 端到端（agent 回环 + 真实 iroh 客户端） | `./e2e-run.sh`，真实 shell 回显 + resize | 见 [building.md](building.md) §4 |

`src/main.ts` 目前**只做到**：`pairedDevices` 的持久化读写。**`deviceConnections.ts`/`terminalStreamTransport.ts`/`devicePairing.ts` 一行都没有被 `main.ts` 或 `settingsTab.ts` 引用过**——这是本次任务要补的全部内容。

### 完全没开始

1. `loadIroh(): Promise<IrohModule>` 的真实实现（从插件自己的 `node_modules` 加载 `@number0/iroh`）
2. 控制端身份种子（`identitySeed`）的持久化
3. `DeviceConnectionManager` 在 `main.ts` 里的生命周期挂载
4. 设置面板：设备列表 + 添加设备 UI
5. 一种打开真实远程终端的方式（命令面板命令或设备列表按钮）

---

## 2. 已经踩过的坑

### 2.1 A0：Obsidian Electron 渲染进程可以直接 `require` 原生 iroh 绑定

`@number0/iroh` 是 N-API 原生模块。曾经担心 Obsidian 的 Electron 渲染进程加载不了，需要一个叫 `termy-bridge` 的兜底本机伴生进程。**2026-07-31 在真实 Obsidian 里实测确认：直接 `require()` 可行**（25 个导出符号，`Endpoint` 绑定+连接码生成都成功）。所以**不需要 `termy-bridge`**，`loadIroh()` 就是一句 `require(路径)`。

### 2.2 `@number0/iroh` 必须从插件自己的目录加载，不是从仓库

esbuild 把 `@number0/iroh` 标为 `external`（见 `esbuild.config.mjs`），运行时 `main.js` 会执行一句裸的 `require('@number0/iroh')`。**这在源码开发模式下会 resolve 到仓库根的 `node_modules`，但真实分发给用户的插件包里没有仓库，只有插件自己的目录**——`scripts/package-plugin.js` 的第 5b 步会把 `@number0/iroh` 连同当前平台的原生子包一起拷进 `plugin-package/node_modules/@number0/`（详见 [building.md](building.md) §3.2）。

`loadIroh()` 的实现**不能**写死 `require('@number0/iroh')` 让 Node 走默认解析——那样在开发模式下能跑、打包分发后就会找不到模块（除非插件目录恰好也在 Node 的模块解析路径上，不能假设）。正确做法是**用插件自己的绝对目录拼路径去 require**,`main.ts` 里已经有拿插件绝对路径的先例，搜 `manifest.dir` 和 `getBasePath`（约第 2840-2847 行）：

```ts
if (!(adapter instanceof FileSystemAdapter)) { throw new Error('FileSystemAdapter is not available'); }
const vaultPath = normalizePath(adapter.getBasePath());
const manifestDir = this.manifest.dir ? normalizePath(this.manifest.dir) : normalizePath(`${configDir}/plugins/${this.manifest.id}`);
```

拼出插件绝对目录后，`require(\`${pluginAbsoluteDir}/node_modules/@number0/iroh\`)`——require 一个绝对路径不受 esbuild 的 `external` 标记影响，也不依赖调用者所在目录。

**错误处理要给清楚的中文提示**：如果用户是从源码走 `pnpm install:dev` 装的插件（没走 `pnpm package` 打包流程），插件目录下根本没有 `node_modules/@number0`，这个 require 会失败——不要让它变成一个吞掉的 Promise rejection，UI 上要能看到"缺少远程功能所需的原生模块，请使用打包版本"之类的提示。

### 2.3 Windows 上发现的三个真 bug，已在 `agent/` 修完，跟插件端无关但交接时容易被问起

2026-07-31 的 Windows 真机验收发现并修复了三个 Agent 端 bug（PTY 收尾在 Windows ConPTY 下卡住、`status` 在 Windows 上永远显示未运行、Ctrl-C 之后进程不退出），全部已修（`agent/` 提交 `f1de258`）。这些不需要插件端做任何事，只是如果真机测试时观察到"agent 好像卡住了"之类的现象，先确认 `agent/` 是不是新版本，而不要怀疑是插件端连接逻辑的问题。

---

## 3. 要做的事

### 3.1 `loadIroh` 的真实实现

建议新建 `src/services/remote/irohRuntime.ts`：

```ts
export function createIrohLoader(pluginDir: string): () => Promise<IrohModule> {
  return async () => {
    try {
      return require(`${pluginDir}/node_modules/@number0/iroh`) as IrohModule;
    } catch (err) {
      throw new Error(`远程功能所需的原生模块缺失或加载失败（${pluginDir}/node_modules/@number0/iroh）。如果你是从源码直接安装的插件，请改用 pnpm package 打包后的版本。原始错误：${err}`);
    }
  };
}
```

具体错误文案和抛出方式按插件现有的错误处理约定来，上面只是示意。

### 3.2 控制端身份持久化

`DeviceConnectionManager` 首次绑定时通过 `onIdentityCreated(seed: number[])` 报告新生成的身份种子，需要持久化，否则每次重启插件都会换一个身份（导致已配对设备全部失效）。参照 `pairedDevices` 已经接好的模式：

- `src/settings/settings.ts` 的 `TerminalSettings` 加字段 `controllerIdentitySeed: number[] | null`，默认 `null`
- 归一化时校验：要么 `null`，要么长度正好 32 的 number 数组，否则视为 `null`（防止手改坏的 `data.json` 让绑定直接崩）
- `main.ts` 构造 `DeviceConnectionManager` 时把这个字段传给 `identitySeed`；`onIdentityCreated` 回调里存回 `settings.controllerIdentitySeed` 并 `saveSettings()`

### 3.3 `main.ts` 生命周期挂载

参照现有 `getRemoteService()`（懒加载单例、`onunload` 里 dispose）的写法，加 `getDeviceConnectionManager()`。`dispose()` 是异步的，`onunload` 本身是同步的——注意不要产生未处理的 rejection（`void this._deviceConnections?.dispose()` 或参照插件里其它地方怎么处理异步清理）。

### 3.4 设置面板：设备列表 + 添加设备

参照 `settingsTab.ts` 现有 `renderRemoteSettings()`（V1 的账号/配对码 UI，用 `Setting` API 和 `t()` 做 i18n）的写法风格，新增 v2.0 的 UI 区块：

- 文本输入框 + 按钮：贴连接码 + 填设备名 -> `pairDevice()`（`parser` 参数用 `loadIroh()` 加载出的模块构造一个真正的 `ConnectionCodeParser`，用 `EndpointTicket.fromString(code).endpointAddr()` 拿 `id().toString()` 作为 nodeId，具体签名看 `irohStreams.ts` 里 `IrohEndpointTicket`/`IrohEndpointAddr`）。校验失败要把 `checkConnectionCode`/`pairDevice` 返回的 `problem`/`code` 转成中文提示，不要直接甩英文错误码
- 设备列表：遍历 `PairedDeviceStore.list()`，显示名称、在线状态（`DeviceConnectionManager.status(nodeId)` + `onDidChange` 订阅做实时刷新）、`lastConnectedAt`；每条给"连接"/"断开"和"移除"
- 具体交互细节（单独开 Modal 还是塞进设置页、连接中/失败状态怎么展示）是开放的 UI 设计问题，跟这个插件现有的视觉风格和交互习惯保持一致就行（参考 `renderRemoteSettings()`、`PresetScriptModal`）
- i18n：`src/i18n/types.ts` 的 `remote` 段加新 key，`locales/en.ts` 和 `locales/zh-CN.ts` 都要填

### 3.5 打开一个真远程终端

`TerminalService`/`TerminalInstance` 现在走的是 `RemoteService.createTerminalTransport()`（零参数，V1 单设备模型），调用点在 `src/services/terminal/terminalService.ts` 第 247 行：

```ts
await terminal.initializeWithTransport(this.remoteService.createTerminalTransport());
```

v2.0 是多设备，需要用户先选一个已连接的设备。至少要有一种触发方式（命令面板命令，或设备列表每行的"新建终端"按钮）：拿到某个 `nodeId` -> 确认 `DeviceConnectionManager.status(nodeId)` 是 connected（没连上先 `connect()`）-> `createTerminalTransport(nodeId)` -> 喂给 `TerminalInstance.initializeWithTransport`（可能需要在 `TerminalService` 上加一个新方法，多接一个 transport 参数，不要改坏 V1 那条零参数调用路径）。

---

## 4. 建议顺序

1. `loadIroh` 真实实现 + 身份持久化 → 能跑通"插件启动后拿到一个可用的 iroh 模块和稳定身份"这条链路，写个不依赖 Obsidian API 的单测覆盖归一化逻辑
2. `DeviceConnectionManager` 在 `main.ts` 里的生命周期挂载
3. 添加设备 UI（贴连接码 -> 配对成功，设备出现在列表里）→ 找一台跑着 `termy-agent run` 的真机手测
4. 设备列表 + 连接状态展示
5. 打开真远程终端 → 端到端手测：输入输出、resize、断开重连
6. 每一步做完就手测一遍，不要攒到最后一起测

---

## 5. 硬性约束

- **不要动 `agent/` 目录**，这次是插件端任务
- **不要删除或大改** `remoteService.ts`、`relayClient.ts`、`authClient.ts`、`deviceClient.ts` 等 V1 文件——它们还在被 `main.ts`/`terminalService.ts` 用着，V1 的账号+云端配对功能要继续能用，这次是新增 v2.0 路径，不是替换
- 所有新增/改动的 TS 代码 `pnpm build`（含 tsc 类型检查）和 `pnpm lint` 必须干净
- 纯逻辑代码（比如 `irohRuntime.ts` 不依赖 Obsidian API 的部分）尽量补 `.test.ts`，参照 `src/services/remote/*.test.ts` 的写法（`node:test` + `node:assert/strict`）
- UI 相关的东西没法自动测，改完要在真实 Obsidian 里手动点一遍完整流程：加设备、看列表、连接、开终端、输入输出、断开、移除，不能只凭编译通过就算完成
- 提交信息中文 conventional 格式，按步骤拆成合理的几个提交，不要一坨全塞一个 commit

---

## 6. 真机验证不了、必须在真实 Obsidian 里确认的事

- `loadIroh()` 在**打包后的插件**（`pnpm package` 产物）里能不能真的加载成功，不只是开发模式下的 `pnpm install:dev`
- `manifest.dir`/`getBasePath()` 拼出来的路径在 Windows 上是不是真的指向插件目录（斜杠方向、盘符）
- 添加设备 -> 打开终端 -> 输入输出的完整链路在真实网络环境下（不是 `--loopback`）能不能跑通

---

## 7. 参考

- 实现方案：`Docs/实现方案 - 开发版 v2.0.md`
- 开发计划与风险登记：`Docs/开发计划 v2.0.md`
- 运维/CLI 参考：[operations.md](operations.md)
- 隐私与已知限制：[privacy-and-limits.md](privacy-and-limits.md)
- Windows 真机验收记录：`Docs/交接结果 - Windows.md`
