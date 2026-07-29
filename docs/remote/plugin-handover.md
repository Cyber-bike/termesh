# 插件端实现交接

写给在 Windows 机器上接手插件改造的人。后端（协议、Relay、Agent）已完成并实测通过；插件端只完成了不依赖 Obsidian 运行时的那一半。

本文记录：**已经做完的**、**还要做什么**、**怎么验证每一步**，以及**已经踩过的坑**——最后一项最重要，有几个是读代码看不出来的。

---

## 1. 现状

### 已完成且已验证

| 位置 | 内容 | 验证方式 |
| --- | --- | --- |
| `protocol/` | 16 条消息 Schema、OpenAPI、38 条 fixtures、12 条帧向量、TS/Rust 类型生成 | `cd protocol && npm test` |
| `relay/` | 6 个 HTTPS 接口 + WSS 网关（路由、背压、心跳、归属校验） | `cargo test --manifest-path relay/Cargo.toml`（71 项） |
| `agent/` | CLI、配置、单实例锁、退避重连、PTY、进程树终止、文件接收 | `cargo test --manifest-path agent/Cargo.toml`（33 项） |
| `src/services/remote/*.ts` | 帧编解码、Relay 客户端、远程 Transport、设备轮询、附件收集与发送、状态机 | `pnpm test:remote`（63 项） |
| 插件端 Obsidian 集成 | Transport 收敛、远程工具栏、拖拽分流、登录/配对/设备设置 | `pnpm test:terminal`、`pnpm lint:obsidian`、`pnpm build` |
| 端到端 | 真实终端命令 + 真实文件传输 | `./e2e-run.sh`（8 项检查） |

`src/protocol/generated/messages.ts` 是从 `protocol/` 生成后同步过来的，**不要手改**；改了 CI 的 `sync-protocol.js --check` 会失败。

### 已完成，待真机验收

1. `TerminalInstance` 已收敛到 Transport 接口，本地终端 133 项回归测试通过
2. `TerminalView` 已接入拖拽分流、远程状态机、设备选择与连接 UI
3. `relayClient.ts` 已实现 HTTPS 登录、设备 API 与控制 WSS 连接管理
4. 设置面板已实现服务器地址、登录、配对码和设备管理

自动化验证已通过：`pnpm test:terminal`（133 项）、`pnpm test:remote`（63 项）、`pnpm lint:obsidian`、`pnpm build`。仍需按第 5 节在真实 Obsidian、Windows 和 Ubuntu 设备上完成运行时验收。

---

## 2. 已经踩过的坑

按被坑的惨烈程度排序。**这些都是实测发现的，不是推演的。**

### 2.1 `cwd` 不需要走协议

我一度以为远程模式下 cwd 跟踪要靠 Agent 上报，还在 `terminal.shellEvent` 里加了 `cwd` 字段。查证后发现是错的：

- Rust 侧的 `osc_scanner.rs` **根本不解析 cwd**，只产生 A/B/C/D 四个命令边界标记；
- 本地模式的 cwd 是**插件在 TypeScript 侧从 xterm 缓冲区解析提示行**得到的（`src/services/terminal/promptCwdParsers.ts` 的 `extractCwdFromPromptLines`）；
- 远程模式下终端输出字节原样透传，**插件照样能解析**。

所以 `cwd` 字段已从协议删掉。`terminal.shellEvent` 的 payload 现在与本地 `ShellEvent`（`src/services/server/types.ts`）**形状完全一致**：

```ts
{ type: ShellEventType; source: ShellEventSource; exitCode: number | null }
```

意味着 `RemoteTerminalTransport.onShellEvent` 收到 payload 可以**直接转交** `TerminalInstance.handleShellEvent`，不需要转换层。

### 2.2 Transport 必须有四条事件通道，不是两条

原方案的接口只有 `onData` / `onExit`。实际读代码发现 `TerminalInstance` 消费**四条**（`terminalInstance.ts:566-591`）：

| PtyClient | 用途 | 少了会怎样 |
| --- | --- | --- |
| `onSessionOutput` | 终端输出 | — |
| `onSessionExit` | shell 退出 | — |
| `onSessionError` | 会话级错误 | 远程错误无处上报 |
| `onSessionShellEvent` | shell 集成 | **cwd 跟踪、AI CLI 上下文交接全部失效** |

`src/services/remote/transport.ts` 已按四通道定义。

### 2.3 改造范围比"薄封装"大得多

`terminalInstance.ts` 是 **2287 行**，有 **15 处以上**直接调用 `this.ptyClient.*`，分布在：

```
566  onSessionOutput      578  onSessionExit       584  onSessionError
590  onSessionShellEvent  614  init                616  destroySession
655  writeBinary          817  write               835  resize
886  disposePtyClientHandlers                      927  同上
935  destroySession       1833 write               1869 isConnected
2206 write (Ctrl-C)       2213 write (clear)       2226/2234 同上
```

`PtyClient` 的每个方法都带 `sessionId`（一个 client 管多个会话），而 Transport 是**每会话一个实例**。所以 `LocalTerminalTransport` 是"绑定了某个 sessionId 的 PtyClient 包装"，`open()` 返回的 `TerminalSessionInfo` 把 sessionId 带回给 `TerminalInstance` 继续用。

**这是对核心类的侵入式重构，排期按此估。** §16.7 要求本地终端零回归，所以先做 `LocalTerminalTransport` 并确认本地功能完好，再接远程。

### 2.4 `buildDroppedInput` 是私有方法，分流点在它之前

方案里写"本地模式 -> 现有 `buildDroppedInput`"，实际它是 `TerminalView` 的私有方法，位置：

```
terminalView.ts:557  private async handleDrop(dataTransfer)   ← 分流点在这里
terminalView.ts:570  private async buildDroppedInput(...)     ← 它负责解析出本地绝对路径
```

方案 §5.4 要求"禁止先生成本地绝对路径再判断远程模式"，落到代码上就是：**在 `handleDrop` 里、调用 `buildDroppedInput` 之前分流**。

### 2.5 未解析链接的处理决定了功能可用性

方案原文写"无法解析的本地链接使整次预检失败"。**照这个实现，绝大多数真实笔记都传不出去**——Obsidian 里指向尚未创建笔记的 wiki 链接是常态，`metadataCache.unresolvedLinks` 通常非空。

已改为：只有**看起来是附件**（带已知附件扩展名）的链接解析失败才整批拒绝；指向 Markdown 或无扩展名的未解析链接直接忽略。逻辑在 `noteCollector.ts` 的 `looksLikeAttachment`，有专门的测试锁定。

### 2.6 控制字符正则被写坏过

`pathSafety.ts` 里检测控制字符的正则一度被原始控制字节污染成 `/[^@-^_]/`——**取反字符类，会拒绝所有路径**。更麻烦的是文件含控制字节后 `grep` 把它当二进制，连续几次搜索都静默无输出，`cat -A` 才看出来。

已加测试 `a control character is caught rather than matching everything` 锁死。**你改这个文件时注意别用编辑器直接敲控制字符。**

---

## 3. 已实现部分与验收要点

### 3.1 `LocalTerminalTransport`（先做这个）

`src/services/remote/localTransport.ts`，包装现有 `PtyClient`：

```ts
export class LocalTerminalTransport implements TerminalTransport {
  private sessionId: string | null = null;
  constructor(private readonly ptyClient: PtyClient, private readonly config: PtyConfig) {}

  async open(options: TerminalOpenOptions): Promise<TerminalSessionInfo> {
    this.sessionId = await this.ptyClient.init({ ...this.config, cols: options.cols, rows: options.rows });
    return { sessionId: this.sessionId, shell: /* 现有逻辑 */ };
  }
  write(data) { this.ptyClient.writeBinary(this.sessionId!, data); }
  resize(c, r) { this.ptyClient.resize(this.sessionId!, c, r); }
  async close() { this.ptyClient.destroySession(this.sessionId!); }
  onData(h) { return toDisposable(this.ptyClient.onSessionOutput(this.sessionId!, h)); }
  onExit(h) { /* onSessionExit -> TerminalExitEvent */ }
  onError(h) { return toDisposable(this.ptyClient.onSessionError(this.sessionId!, h)); }
  onShellEvent(h) { return toDisposable(this.ptyClient.onSessionShellEvent(this.sessionId!, h)); }
}
```

`toDisposable` 和 `DisposableBag` 已在 `transport.ts` 提供。

**验证**：切到这条路径后跑 `pnpm test:terminal`，并在真机上手测本地终端——输入输出、复制粘贴、搜索、resize、Ctrl-C、清屏、本地拖拽。§16.7 的零回归就是这一步。

### 3.2 `relayClient.ts`

`src/services/remote/relayClient.ts`，两件事：

**HTTPS**（接口契约见 `protocol/openapi.yaml`）：`login` / `createPairingCode` / `revokePairingCode` / `listDevices` / `deleteDevice`。注意访问令牌 15 分钟过期且**没有 refresh**，过期后重新登录。

**控制 WSS**：`wss://<host>/v1/control/ws`，必须带

```
Authorization: Bearer <accessToken>
Sec-WebSocket-Protocol: termy.v1
```

浏览器原生 `WebSocket` **设不了 Authorization 头**，所以必须用 `ws`（已是 Termy 依赖，`package.json` devDependencies 里的 `ws@8.20.1`，esbuild 会打包进去）。

连接期内令牌不再校验（§6.2），所以一次会话可以超过 15 分钟；但设备列表轮询会先失效并要求重新登录。

消息分发：`ControlMessageByType`（`src/protocol/generated/messages.ts`）是按 `type` 判别的联合类型，`switch (msg.type)` 可以拿到精确的 payload 类型。

**验证**：本地起后端跑通。用 `./e2e-run.sh` 的做法——`TERMY_AGENT_ALLOW_INSECURE=1` 允许 Agent 连明文本地 relay，插件侧同理需要允许 `ws://`，别把这个开关带进发布版。

### 3.3 `RemoteTerminalTransport`

用 `relayClient` 的控制连接实现四条通道：

| Transport | 线上表现 |
| --- | --- |
| `open` | 发 `terminal.open`，等 `terminal.opened`（15 s 超时），返回其 `sessionId` |
| `write` | 二进制帧 `kind=0x01`，`streamId=sessionId`，offset 自增 |
| `resize` | `terminal.resize` |
| `close` | `terminal.close`，`reason=user` |
| `onData` | 二进制帧 `kind=0x02` |
| `onExit` | `terminal.close`（`reason=shell_exited`，带 exitCode） |
| `onError` | `terminal.error` |
| `onShellEvent` | `terminal.shellEvent`，payload 直接转交 |

编解码用 `frameCodec.ts`，已与另外两端的实现对过 hex 向量。

### 3.4 拖拽分流

改 `terminalView.ts:557` 的 `handleDrop`：

```ts
private async handleDrop(dataTransfer: DataTransfer | null): Promise<void> {
  if (this.remoteMode?.state === 'Connected') {
    await this.handleRemoteDrop(dataTransfer);   // 校验单个 .md TFile
    return;
  }
  if (this.remoteMode && this.remoteMode.state !== 'LocalMode') {
    new Notice(/* 按 capabilities(state) 给原因 */);
    return;
  }
  const input = await this.buildDroppedInput(dataTransfer);  // 现有本地路径，不动
  ...
}
```

远程分支流程：取单个 Markdown `TFile` → `createVaultLinkSource(app, file)` → `collect()` → `checkQuotas()` → 失败弹 `error` 文案 → 成功则 `TransferSender.run()`。

`vaultLinkSource.ts` 已写好但**未验证**——它用 `metadataCache.getFileCache()` 的 `links`/`embeds` 而不是 `resolvedLinks`，因为后者按解析后路径索引，丢掉了原始链接文本和顺序，而判断"未解析的是不是附件"必须要原文。这个假设需要在真机上确认。

### 3.5 UI 状态

`remoteState.ts` 已提供纯 reducer 和能力表，接进 `TerminalView` 即可：`capabilities(state)` 返回 `{ input, drop, deviceSelection }`，直接驱动禁用逻辑。状态图六个状态的可达性有测试保证。

设备在线状态走**轮询**（§4.11），固定 15 s，仅在远程模式且未连接时轮询——Relay 不推送设备状态事件。

---

## 4. 建议顺序

1. `LocalTerminalTransport` + `TerminalInstance` 重构 → **确认本地零回归**（最大的风险在这里，先隔离掉）
2. `relayClient.ts` HTTPS 部分 + 设置面板（登录、生成配对码、设备列表）
3. 用 Agent 实机配对一台机器，确认设备列表显示在线
4. 控制 WSS + `RemoteTerminalTransport` → 远程终端可用
5. 拖拽分流 + 传输 → 远程传文件可用
6. §16 完成定义逐条验收

第 1 步做完就该停下来完整回归一次本地功能。后面每一步都能独立验证。

---

## 5. 本机验证不了、必须在真机做的事

- **Windows 进程树终止**：`agent/src/pty.rs` 的 Job Object 实现从未在 Windows 上编译或运行过。CI 的 `agent-windows` job 会做类型检查，但**证明不了它真能杀掉子孙进程**。手测方法：远程 shell 里起一个后台进程，关闭会话，确认它死了。Linux 侧的等价测试是 `terminating_kills_the_whole_process_group`，已通过。
- **`loginctl enable-linger` 的权限要求**：多数发行版需要一次 root 或 polkit 认证，`install-linux.sh` 里用 `sudo` 处理。目标机器上确认一遍。
- **非同一局域网的双路径验收**：§16.1。
- **Obsidian 宿主假设**：`vaultLinkSource.ts` 用的 `getFileByPath` / `getFirstLinkpathDest` / `getFileCache` 都按 Obsidian API 1.12.3 写，真机确认。

---

## 6. 参考

- 技术方案：`/root/Termy远程增强-MVP技术方案.md`（本仓库外）
- 协议契约：`protocol/README.md`——含落地时做的判断，以及为什么 Rust 侧要在生成类型之外再跑一遍 Schema 校验
- 运维：`docs/remote/operations.md`
- 隐私与已知限制：`docs/remote/privacy-and-limits.md`
