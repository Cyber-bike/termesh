# Termy 远程增强 — 支持多个并发远程终端

> **历史文档，已被 v2.0 实际实现取代。** 多会话已经落地（`agent/src/session_table.rs`），但走的不是本文档设想的"不设上限 + 每会话流控"，而是`maxConcurrentSessions`（默认 8）软上限、超出即拒绝（`SESSION_LIMIT_REACHED`）不排队、没有单独的流控协议面。当前实际设计见《实现方案 - 开发版 v2.0.md》和 [plugin-handover.md](plugin-handover.md)。保留本文档只作为设计过程记录。

## Context

当前每台设备同时只能开**一个**远程终端。第二个 `terminal.open` 会被 Agent 用 `DEVICE_BUSY`（"A remote terminal is already running"）直接拒掉。

这个限制在真实使用中很快就碰到了：远程机器上跑着 Claude Code 这类长时间交互程序时，没法再开一个终端看日志或跑别的命令。本地终端早就支持多标签（`TerminalService.terminals` 是个 Map，多开靠多个 Obsidian leaf），远程却退化成单个。

目标是两个场景都支持：

1. **同一台设备多个终端** —— 左边跟 Claude Code 对话，右边 `tail -f` 日志
2. **跨设备多个终端** —— 一个连 Ubuntu，一个连另一台 Windows

### 用户已定的两个取舍

- **每会话流控要做。** 不接受"一个刷屏终端拖慢其它终端"。这是唯一需要新增协议面的部分。
- **不设会话数上限。** 靠系统资源兜底，`DEVICE_BUSY` 作为常开拒绝彻底消失。

---

## 评估结论：协议是对的，改动集中在 Agent 和插件 UI

探查三端后的核心发现——**基础多会话能力不需要动协议**：

| 层 | 现状 | 需要做什么 |
| --- | --- | --- |
| **协议** | 信封里 `deviceId` + `sessionId` 全部必填；二进制帧头有 16 字节 `stream_id` | 基础多会话：**零改动**。仅流控需要新增消息 |
| **Relay** | `sessions: HashMap<Uuid, Route>` 已按会话索引，`Route` 带 `device_id`；二进制帧已按 session 查表路由（`control.rs:187-234`、`agent.rs:255-296`）；断连时已经是"每会话发一条 `terminal.close`"的循环 | **跨设备多会话本来就成立**。只需清理死代码 + 流控 |
| **Agent** | 单个 `Option<Session>` 栈变量 + 冗余的 `SessionSlot` | **主要工作量** |
| **插件** | 本地多终端已完备，远程路径全是全局单例 | **UI 改动最多** |

Relay 的注释 `registry.rs:214-215` 写明了设计前提："one control connection per account... This is what lets responses route by userId alone, with no requestId table." 这条限制**不妨碍**本方案——一个 Obsidian 实例共用一条控制连接，本来就是插件侧现在的做法（`remoteService.ts:135-147` 的 `connect()` 在已有连接时直接返回）。

### 单会话假设的确切位置

**Agent（硬限制在这）**

- `agent/src/client.rs:162` —— `let mut session: Option<Session> = None;` 是 `serve()` 里的栈变量，以 `&mut` 传进 `handle_control`（`:314`）和 `handle_binary`（`:564`）
- `agent/src/pty.rs:323-357` —— `SessionSlot`，与上面那个 `Option` 冗余（每连接创建一次，从不跨任务共享）
- `agent/src/client.rs:330-346` —— `DEVICE_BUSY` 拒绝分支
- **最深的一处**：`client.rs:159` 的全局 `pty_tx/pty_rx` 单通道，`PtyEvent`（`:298-306`）**不带 sessionId**。消费端 `:199-241` 把字节归属给"当前那个 session"。两个 pump 同时喂这条通道会导致输出错配、offset 错乱
- `client.rs:397-414` —— `terminal.resize` 和 `terminal.close` **完全忽略信封里的 sessionId**
- `client.rs:673-674` —— pump 线程无法停止只能饿死，EOF 后无条件发 `Exited(0)`。现在靠 `session` 是 `None` 吞掉；多会话下会关错会话
- `agent/src/state.rs:30-31` —— 状态文件是 `session_active: bool`，没有会话列表

**Relay（基本不用改）**

- `registry.rs:253-258` `device_has_session` 是**死代码**，只有自己的单测在调用——单会话假设的化石
- `registry.rs:241-243` `open_session` 是裸 `insert`，没有唯一性检查
- 队头阻塞：`ConnHandle` 每连接两条 mpsc（`registry.rs:40-58`），`CONTROL_CAPACITY=256` 被所有会话共用；`agent.rs:288` 的 `.await` 会**卡住整个 Agent 读循环**；通道满则以 4413 关掉**整条连接**，所有会话陪葬

**插件（UI 改动最多）**

- `settings.ts:171-181` —— 选中设备是**全局**的 `deviceId`，某个标签页换设备会改掉所有标签页
- `remoteService.ts:172-178` —— `createTerminalTransport()` 不接任何参数，读全局 deviceId
- `remoteService.ts:149-159`、`:280-292` —— `disconnect()` / `setRemoteMode(false)` 全局拆连接
- `terminalView.ts:290-305` —— **一个视图关闭会调 `setRemoteMode(false)`，把所有人的远程终端一起断掉**
- `terminalView.ts:727-731` —— `replaceTerminal` 是**替换**唯一实例，不是新增
- `terminalView.ts:174-183` —— 每视图的状态机被**全局** snapshot 驱动，一次失败让所有视图进 Error
- 两处错误串台：`remoteTerminalTransport.ts:207-216`（`sessionId === null` 的错误广播给**每个** transport）和 `:223-226`（解码失败在所有 transport 上报 `PROTOCOL_ERROR`）

### 已经可以直接复用的东西

- **`PtyClient`**（`src/services/server/ptyClient.ts:27,30,170-210`）—— 本地路径里"一条 socket、N 个会话"的标准多路复用器：`Map<sessionId, listeners>` + 按会话订阅的 API。远程侧应照搬这个形状，把 demux 从每个 transport 移进连接层
- **`LocalTerminalTransport`**（`localTransport.ts:12-83`）—— "把一个 sessionId 绑到多路复用客户端上"的适配器，正是 `RemoteTerminalTransport` 该有的样子
- **`RemoteService.senders` / `pendingTransfers`**（`remoteService.ts:58-59,248-272`）—— 远程代码里**已经存在的正确 demux 模式**，按 `transferId` 路由
- `TerminalService.terminals` Map + 引用计数拆服务端（`terminalService.ts:62,329-332`）
- `DisposableBag` / `toDisposable`（`transport.ts:53-84`）
- `releaseTerminalInstance` / `adoptTerminalInstance`（`terminalView.ts:311-340`）

### 已知的测试空白

- **没有任何测试在一条连接上创建两个 transport**，所以上面那两处串台是完全没被覆盖的
- `terminalService.ts` 和 `terminalView.ts` **各自都没有测试文件**
- `relay/tests/gateway.rs` 里没有任何双会话测试；`registry.rs:364-441` 的单测是唯一出现多会话的地方，且只验证过滤

---

## 一个关键简化：不需要改"替换 vs 新增"

`terminalView.ts:727-731` 的 `replaceTerminal` 是替换而非新增，一度看起来是障碍。实际不是——**多终端在 Termy 里本来就等于多个 Obsidian leaf**（`main.ts:617-635`、`terminalLeafRouting.ts:29-131`），每个 leaf 持有一个 `TerminalInstance`。所以"一个标签页要么本地要么远程"的语义可以原样保留，用户开新标签页再切远程即可。

要改的只是**让每个 leaf 的远程终端彼此独立**，不是改视图的实例模型。这砍掉了插件侧相当一部分预想的工作。

---

## 一期：基础多会话（不动协议，不动 relay 数据面）

### Agent —— `agent/src/`

改动集中在 `client.rs::serve()`：

1. **`client.rs:162`** `session: Option<Session>` → `sessions: HashMap<Uuid, Session>`。`handle_control`（`:314`）和 `handle_binary`（`:564`）的 `&mut Option<Session>` 参数跟着改成 `&mut HashMap<..>`。
2. **`PtyEvent`（`:298-306`）增加 `session_id: Uuid` 字段**，`spawn_output_pump`（`:646-676`）接收所属 session id 并给每个事件打标。这是最关键的一处——现在的归属是位置性的。
3. **消费端 `:199-241` 改为按 `event.session_id` 查表**。查不到就丢弃——这同时**顺手修掉了陈旧 pump 的 `Exited(0)` 误伤**（`:673-674`）：那条事件带的是已消失会话的 id，不会再关掉别人。
4. **删除 `SessionSlot`**（`pty.rs:321-357`）、`DEVICE_BUSY` 分支（`client.rs:330-346`）、以及 `pty.rs:363-379` 那个断言"第二个必须被拒"的测试。`release_any()`（`client.rs:288`）改为遍历全部会话。
5. **`terminal.resize` / `terminal.close`（`:397-414`）必须读信封的 `sessionId` 再查表**。现在它们完全忽略这个字段，多会话下会操作错对象。未知 id：resize 忽略，close 幂等返回。
6. **`handle_binary`（`:559-588`）** 把 `active.id != stream_id` 的相等判断换成 `sessions.get_mut(&stream_id)`。
7. **把 teardown 挪出 async 循环**：`kill_process_group`（`pty.rs:161-185`）里有最长 2 秒的 `std::thread::sleep` 轮询，却是从 `handle_control` 在 tokio worker 线程上直接调的（`client.rs:408`）。单会话时只卡自己，多会话下会**卡住其它所有会话的 I/O**。改用 `tokio::task::spawn_blocking`（`PtySession` 的三个成员都是 `Send`）。`serve()` 结束时收集全部 `JoinHandle` 一起 `join_all` 并加总超时，让 N 个会话**并行**在约 2 秒内拆完，而不是串行 2N 秒。

8. **`write_input` 要的是专用写线程，不是 `spawn_blocking`**：`pty.rs:87-91` 的 `write_all` + `flush` 在从进程不读 stdin 且有大段粘贴时会阻塞。但**两个 `spawn_blocking` 任务可能被重排，而这是字节流**——顺序错乱就是数据损坏。正确做法是每个 `PtySession` 在 `spawn` 时起一条专用写线程，持有 `Box<dyn Write + Send>` 按 FIFO 写；`PtySession` 存 `SyncSender<Vec<u8>>`（有界，如 64），`write_input` 变成非阻塞 `try_send`，`Full` 映射成一条警告级错误。teardown 时 drop 掉 sender 即可结束该线程。
9. **状态文件**（`state.rs:30-31`）`session_active: bool` → `sessionCount: u32`，`termy-agent status`（`main.rs:159-166`）的渲染跟着改。状态文件是 Agent 本地的、启动时重写，**不需要迁移**。

**不用动的**：per-session 的 `output_offset`/`input_offset`（`client.rs:136-137`）、per-session 的 `OscScanner`、Windows 的 per-`PtySession` Job Object（`pty.rs:21-22`）、Unix 基于 sid 的 `/proc` 扫描（`pty.rs:200-240`，每个 PTY 有独立 sid，天然正确）、单实例锁。

### Relay —— 数据面不用改，但错误处理必须改

**这一条是评估过程中最重要的更正。** 初看 relay 只需清理死代码，深入之后发现不是：**今天每一个 `Fault` 都会关闭整条 socket**。单会话时这没问题；多会话下，一个过期的 sessionId 或一台离线设备会掐掉用户所有打开的终端。这是多会话可用的前提，不是优化项。

**1. 区分"畸形" 与 "寻址/存活"**（`control.rs`）——前者连接级致命，后者降级为该会话的 `terminal.error`：

| 位置 | 今天 | 改为 |
| --- | --- | --- |
| `control.rs:160` resize/close 遇未知或他人的会话 | 关闭 4403 | `terminal.error{sessionId, DEVICE_FORBIDDEN}` |
| `control.rs:165` resize/close 时设备离线 | 关闭 4403 | `terminal.error{sessionId, DEVICE_OFFLINE}` |
| `control.rs:142` `terminal.open` 的 `authorize_device` 失败 | 关闭 4403 | `terminal.error{requestId, sessionId:null, …}` |
| `control.rs:206,211` 输入帧指向未知会话/离线设备 | 关闭 4403 | `terminal.error{sessionId,…}` + 丢弃该帧 |
| `control.rs:216` 输入通道满 | 关闭 4413，**所有会话陪葬** | `terminal.error{sessionId, BACKPRESSURE_LIMIT}` + `close_session(id)`，影响面收敛到一个会话 |

保持致命的：JSON 非法、schema 校验失败、控制帧超长、方向错误的消息类型、无法解码的二进制帧——这些是对端 bug，没有具体会话可归咎。

**`terminal.error` 的 `sessionId` 本来就可空，这几个错误码也都已在枚举里——这一整块不需要任何 schema 改动。**

**2. 删除死代码** `device_has_session`（`registry.rs:253-258`）及其单测引用。

**3. `open_session`（`registry.rs:241-243`）加唯一性检查**：现在是裸 `insert`，重复 sessionId 会静默覆盖路由、把该会话的输入导向另一台设备。改为返回 `bool`，`agent.rs:229-235` 把 `false` 当协议违规处理。

### 插件 —— `src/`

1. **把 demux 从 transport 移进连接层**，照搬 `PtyClient`（`src/services/server/ptyClient.ts:27,30,170-210`）的形状：`Map<sessionId, listeners>` + 按会话订阅的 API。这样 `RemoteTerminalTransport` 就变成和 `LocalTerminalTransport`（`localTransport.ts:12-83`）同构的薄适配器。现在每个 transport 都收到**全部**消息再自行过滤，能用但脆弱。
2. **修两处串台**：
   - `remoteTerminalTransport.ts:207-216` —— `sessionId === null` 的错误（`DEVICE_BUSY`、`DEVICE_OFFLINE` 正是这种）广播给每个 transport。应按 `requestId` 关联到发起方，或上报到服务级通道。
   - `:223-226` —— 解码失败在所有 transport 上报 `PROTOCOL_ERROR`。同样上移到服务级。
3. **`createTerminalTransport(deviceId)` 显式接设备参数**（现在 `remoteService.ts:172-178` 无参数、读全局）。
4. **每终端记自己的设备**：`TerminalView` 持有自己的 `deviceId`，`settings.remoteConnection.deviceId`（`settings.ts:171-181`）语义降为"新建终端时的默认设备"。设置结构不变，**无需迁移**。工具栏下拉（`terminalView.ts:667-680`）只改当前终端。
5. **连接生命周期与单个视图解耦**：
   - `terminalView.ts:290-305` 的 `onClose` 现在调 `setRemoteMode(false)`，**一个视图关闭会断掉所有人的远程终端**。改成引用计数，照抄 `terminalService.ts:329-332` 拆本地服务端的做法。
   - `remoteService.ts:149-159`、`:280-292` 的全局 `disconnect()` 同理。
6. **状态机改为按会话驱动**：`terminalView.ts:174-183` 现在用全局 snapshot 喂每视图的状态机，一次失败让所有视图进 `Error`。改为订阅本会话的事件。`remoteState.ts` 是纯 reducer，本身不用动。

---

## 二期：每会话流控（需要新增协议消息）

一期交付后，一个刷屏会话仍会拖慢同设备的其它会话，极端情况下 `4413 BACKPRESSURE_LIMIT` 会关掉**整条连接**、所有会话陪葬（`control.rs:216`、`agent.rs:307`）。根因是 `ConnHandle` 每连接只有两条 mpsc（`registry.rs:40-58`），`CONTROL_CAPACITY=256` 被所有会话共用，且 `agent.rs:288` 的 `.await` 会卡住整个 Agent 读循环。

**方向**：镜像已有的文件传输信用窗口。终端输出**不能丢**——虽然协议对终端 offset 跳跃是容忍的（`frame-codec.test.js:171`，插件侧也确实没检查输出 offset），但丢字节意味着用户看到的屏幕内容缺失，对终端不可接受。所以只能背压，不能丢弃或合并。

现成可复用：`CreditWindow`（`src/services/remote/creditWindow.ts`）的累计单调授予语义已经抗乱序和重放，方向反过来即可（插件消费输出后向 Agent 授予信用）。

具体协议形状（新增 `terminal.credit` 消息 vs 拆分每会话通道）在动工前单独定，不阻塞一期。

---

## 验证

每一步都能独立验证，按序推进：

```bash
# Agent 改造后
cargo test --manifest-path agent/Cargo.toml

# Relay 清理后
cargo test --manifest-path relay/Cargo.toml

# 协议未改动，但确认没被意外触碰
cd protocol && npm test && cd .. && node scripts/sync-protocol.js --check

# 插件（需 Node 22；Node 18 的转译变通见 docs/开发/building.md §6）
pnpm test:remote

# 端到端
npm --prefix e2e ci && ./e2e-run.sh
```

**必须补的测试**（当前完全空白）：

- **Agent**：两个并发会话，输出不串台、offset 各自独立；`resize`/`close` 按信封 sessionId 生效；陈旧 pump 的 `Exited` 不会关掉别的会话。
- **Relay**：`relay/tests/gateway.rs` 现在**没有任何双会话测试**。补同设备双会话、以及跨设备双会话的路由。
- **插件**：**没有任何测试在一条连接上创建两个 transport**——上面那两处串台正因如此从未被发现。补双 transport 无串台、以及 session-less 错误只到发起方。
- **`e2e-run.sh`**：扩成开两个会话，各自跑命令，验证输出归属正确。

**真机验收**（本机验不了）：Windows 上多个 Job Object 各自终止进程树；同设备两个会话互不干扰地退出。

---

## 工作量估计

| 层 | 一期 | 风险 |
| --- | --- | --- |
| 协议 | **零** | — |
| Relay | 小（错误降级 + 删死代码 + 唯一性检查 + 补测试，约 200 行） | 低–中：错误降级触及每条错误路径，且要重写 `gateway.rs` 里现有的 close code 断言。机械但面广 |
| Agent | **大**，`client.rs::serve()` 重构 + `pty.rs` 写线程 | **中–高**：pump 阻塞、写线程在 teardown 时卡在 `write_all`、`sent` 与 `output_offset` 不一致会静默挂死会话 |
| 插件 | 中–大（新 hub 约 250 行 + transport 改写 + 服务/视图接线） | **中**：视图生命周期是**完全没有测试**的部分 |
| 端到端 | 中（`e2e/driver.js` 加双会话场景） | 低：多会话真正被证明的地方就在这 |

**最该担心的两处**：

1. **Agent 的 pump / teardown 交互。** 失败模式是线程泄漏和静默挂死，不是崩溃——最难查。大部分可以单测覆盖。
2. **插件视图生命周期。** `terminalService.ts` 和 `terminalView.ts` **各自都没有测试文件**,引用计数改错只能靠手测发现。建议把引用计数抽成一个纯模块（放在 `remoteState.ts` 旁边），这样能进 `pnpm test:remote`。

**明确不做**（免得有人以为包含在内）：不设会话数上限；不加空闲/会话超时（会话仍然只在 `terminal.close`、shell 退出、WS 断开时结束）；不做跨重连的会话恢复。

一期不动协议，是这次评估最好的消息——协议当初按会话寻址设计对了。但 relay 的错误处理必须一起改，否则多会话反而比单会话更脆弱。
