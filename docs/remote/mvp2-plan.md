# Termy 远程增强 v2 —— 免账号配对 + 多设备多终端 + 断线保活

三个分开提出的需求（去账号改配对码、多设备多终端、重连不丢会话）改的是同一批文件
（`relay/src/gateway/control.rs`、`registry.rs`、`agent/src/client.rs`），必须合成一次架构
改动，不能分三轮改。本文档是合并后的技术方案。P2P/WebRTC 直连评估过，本轮不做，见 §7。

读者：接手这次改造的人。建议和 [operations.md](operations.md)、
[privacy-and-limits.md](privacy-and-limits.md)、[multi-session-plan.md](multi-session-plan.md)
对照着看——本方案吸收了 multi-session-plan.md 的大部分结论，但因为设备模型变了，relay 侧
的落地方式不一样，agent 侧又叠加了断线保活，所以单列一份而不是改那份。

---

## 1. 核心变化一览

| | 现状 | v2 |
| --- | --- | --- |
| 顶层实体 | `users`（登录名+密码）持有 `devices` | `devices` 自己就是顶层实体，没有账号 |
| 配对方式 | 登录后调 API 生成一次性配对码，agent 用码换 token | agent 启动时自己生成配对码+token，一次性都注册好 |
| 控制连接 | 一条连接对应一个账号，可挂多台设备 | 一条连接对应一台设备；多设备 = 多条并行连接 |
| 每设备终端数 | 1个，第二个 `terminal.open` 被拒 `DEVICE_BUSY` | 不设上限（系统资源兜底），预留熔断软上限 |
| 断线行为 | WS 一断，PTY 立即被杀 | Agent 进程存活期间，PTY 不因网络断开而死，缓冲输出，重连后补发 |

---

## 2. 数据模型（relay，SQLite）

```
-- 删除
DROP TABLE users;
DROP TABLE pairing_codes;

-- devices 表改造
ALTER TABLE devices DROP COLUMN user_id;
ALTER TABLE devices ADD COLUMN code_digest BLOB NOT NULL;   -- 配对码摘要，和 token_digest 同级
-- token_digest、name、platform、agent_version、last_seen_at 不变
```

两把密钥并列，语义不同：
- `token_digest`：agent ↔ relay，`/v1/agent/ws` 用，agent 自己持有明文，插件永远看不到。
- `code_digest`：插件 ↔ relay，`/v1/control/ws` 用，**长期有效、可复用**（不再是"一次性配对
  码"，改名字面意义也变了，但复用 `crypto.rs` 里同样的 HMAC-pepper 摘要方案）。

两者都在 `agent/src/config.rs` 的 `Config` 里以明文形式保存（0600 文件），因为都需要能被
人类重新查看/使用——`termy-agent show-code` 就是新增出来干这个的。

---

## 3. relay 改动

### 3.1 `relay/src/crypto.rs`、`db.rs`
- `crypto.rs` 不用大改：`generate_pairing_code`/`digest_secret` 照用。删掉 `hash_password`/
  `verify_password`/`DUMMY_PHC` 相关（Argon2 依赖可以整个从 `Cargo.toml` 移除）。
- `db.rs`：删掉 `create_user`/`find_user_by_login`/`set_password_digest` 和全部 pairing_codes
  相关方法。`find_device_by_token_digest` 旁边加一个 `find_device_by_code_digest`，签名对称。
  新增 `create_device_self(name, platform, agent_version, token_digest, code_digest) -> Uuid`，
  一步到位创建设备行，不再有"消费配对码"这个事务步骤。

### 3.2 `relay/src/auth.rs`
- 删除 `Claims`、`issue_access_token`、`verify_access_token`、`AuthUser`（连同 `jsonwebtoken`
  依赖）。`TERMY_JWT_SECRET` 从 `config.rs` 的必需环境变量里去掉。
- 新增一个共享的设备头解析函数，供 `agent.rs` 和 `control.rs` 共用（现在 `agent.rs:54-82` 的
  `authenticate()` 已经写了一遍 `Device <id>.<secret>` 的解析+摘要比对逻辑，直接抽出来，
  按传入的查表函数区分是查 `token_digest` 还是 `code_digest`）：

  ```rust
  pub async fn authenticate_device(
      headers: &HeaderMap,
      lookup: impl Fn(&[u8]) -> BoxFuture<'_, Result<Option<Device>, AppError>>,
      pepper: &[u8],
  ) -> Result<Uuid, AppError> { /* 原 agent.rs:54-82 逻辑原样搬过来 */ }
  ```

### 3.3 `relay/src/api/`
- `auth.rs` 整个文件删除，`api/mod.rs` 去掉 `/v1/auth/login` 路由。
- `devices.rs`：
  - 删除 `create_pairing_code`/`revoke_pairing_code`（不再需要登录态才能生成码）。
  - 删除 `list_devices`/`delete_device`（没有账号维度的"我的设备列表"这回事了；agent 自己
    知道自己的 id，插件在配对时就地记住，不需要服务端提供列表）。
  - `register_device` 改造成免认证的自注册：请求体从 `{pairingCode, deviceName, platform,
    agentVersion}` 简化为 `{deviceName, platform, agentVersion}`；处理逻辑不再"消费码"，而是
    `crypto::generate_pairing_code()` 生成新码、连同 `deviceToken` 一起入库、一起返回：
    ```rust
    pub struct RegisterResponse {
        pub device_id: String,
        pub device_token: String,
        pub pairing_code: String,   // 新增字段
        pub relay_url: String,
    }
    ```
    限流沿用现有 `limits::REGISTER_BY_IP`；因为不再有 `MAX_DEVICES_PER_USER` 这道闸（没有
    account 了），改成一个全局的 `TERMY_MAX_DEVICES`（配置项，默认给个宽松值比如 64）防止
    单个 relay 被注册请求刷爆磁盘。
  - 新增 `POST /v1/devices/self/rotate-code`，走 `Device <id>.<token>` 认证（agent 自己发起，
    不需要旧配对码），生成新码、更新 `code_digest`、返回明文新码，供 `termy-agent
    rotate-code` 使用。

### 3.4 `relay/src/gateway/registry.rs`
- `Route` 去掉 `user_id`，只剩 `device_id`（一条控制连接已经绑死一台设备，`user_id` 字段
  纯冗余）。
- `register_control`/`unregister_control_if_current`/`control_handle` 的键从 `user_id` 换成
  `device_id`。
- `device_has_session`（目前是死代码，`registry.rs:253-258`）启用起来，用作 §5 提到的会话数
  熔断检查点。
- `sessions: HashMap<Uuid, Route>` 保留，但语义从"路由查找表"降级为"合法性校验表"——因为
  device↔control 已经 1:1，relay 不再需要靠它去找"另一端是谁"，只需要用它确认某个
  `sessionId` 当前确实挂在这台设备上（防止过期/伪造的 session id 被转发）。

### 3.5 `relay/src/gateway/control.rs`（改动最集中的文件）
- 认证从 `AuthUser`（JWT）换成 `Device <deviceId>.<pairingCode>`，直接复用 §3.2 的共享函数，
  查 `code_digest`。
- `authorize_device`（`control.rs:238-260`）整个删除——`terminal.open`/`transfer.start` 里
  客户端声明的 `deviceId` 必须等于连接自身的认证身份，不等就是协议错误，不需要查库确认归属。
- **错误处理降级**（直接采纳 multi-session-plan.md §"Relay" 的结论，原样照做）：
  | 位置 | 现状 | 改为 |
  | --- | --- | --- |
  | resize/close 遇未知/他人会话 | 关闭连接 4403 | `terminal.error{sessionId, DEVICE_FORBIDDEN}` |
  | resize/close 时设备离线 | 关闭连接 4403 | `terminal.error{sessionId, DEVICE_OFFLINE}` |
  | 输入帧指向未知会话/离线设备 | 关闭连接 4403 | `terminal.error{sessionId,…}` + 丢帧 |
  | 输入通道满 | 关闭 4413，全部会话陪葬 | `terminal.error{sessionId, BACKPRESSURE_LIMIT}` + 只关这一个 session |
- **`cleanup()`（`control.rs:317-353`）是本轮新增的关键改动**：目前插件掉线会主动给 agent
  发 `terminal.close` 把所有 session 一起关掉。改为**不再发送**——只把这条控制连接从
  `registry` 摘除，`sessions`/routes 原样保留。会话是否终结完全交给 agent 自己判断（agent
  进程还活着就应该允许重新附着），relay 在这一步唯一要做的是"不要主动帮倒忙"。
  同理 `transfer` 的清理逻辑不变（传输本来就没有"断线保活"的需求，网络断了直接失败重传更简单）。

### 3.6 `relay/src/gateway/agent.rs`
- 认证部分（`authenticate()`）改成调用 §3.2 的共享函数，查 `token_digest`，其余不变。
- `AGENT_OFFLINE_TIMEOUT`（50s，agent↔relay 心跳判定）**和"控制连接断线后的会话宽限期"是
  两回事**，不要混用：前者是 relay 判断 agent 是否掉线，后者是 agent 自己决定 PTY 缓冲区
  留多久（见 §4.2）。这条不需要 relay 改代码，只是要在实现时不要把两个超时搞混。

---

## 4. agent 改动（工作量最大）

### 4.1 `config.rs` / `main.rs`：自注册取代 bind
- `Config` 增加 `pairing_code: String` 字段，和 `device_token` 同等对待（0600 文件里的明文）。
- `main.rs`：`run` 检测到没有配置文件时，自动调用新的自注册端点（不再需要用户先跑
  `termy-agent bind --code ...`），拿到 `deviceId`/`deviceToken`/`pairingCode` 后写配置、
  在终端**醒目打印一次** `pairingCode`（这是用户唯一能看到明文码的时刻之一）。
- 新增子命令：
  - `termy-agent show-code`——读配置，重新打印配对码（第二台 Obsidian 想连同一台机器时用）。
  - `termy-agent rotate-code`——调 §3.3 的 rotate 端点，旧码立即失效。
- `bind` 子命令保留但降级为"手动指定 relay 地址+设备名"的等价物，不再需要外部传入配对码
  参数。

### 4.2 `client.rs`：会话生命周期与重连解耦（合并了 multi-session-plan.md 一期 + 断线保活）

这是这次改造真正的核心。目标数据结构：

```rust
struct Session {
    id: Uuid,
    pty: PtySession,
    output_offset: u64,
    input_offset: u64,
    ring: OutputRingBuffer,      // 新增：断线期间的输出缓冲
    attached: bool,              // 新增：当前是否有存活的 relay 连接在消费它
}

struct SessionManager {
    sessions: HashMap<Uuid, Session>,
    idle_since_orphaned: HashMap<Uuid, Instant>, // 新增：孤儿会话计时
}
```

`SessionManager` 的生命周期要**提升到 `run()` 这一层**（跨越 `connect_and_serve` 的多次调用），
不能再是 `serve()` 里的栈变量——这是和 multi-session-plan.md 原方案最大的不同点，原方案假设
`Option<Session>`/`HashMap<Session>` 随连接创建销毁即可，断线保活要求它**比连接活得更久**。

具体改动，按 multi-session-plan.md 已经定位好的位置，逐条叠加断线保活的部分：

1. **`PtyEvent` 加 `session_id` 字段**（原方案已提出），输出通过 `spawn_output_pump` 打标后，
   先写入该 session 的 `ring`，再尝试通过 `out_tx` 发给 relay——**这一步的顺序很重要**：先落
   缓冲区、再尝试发送，这样即使 `out_tx` 发送失败（连接已断），字节也不会丢，下次重连直接从
   缓冲区里补发。
2. **`terminal.resize`/`terminal.close` 按信封 `sessionId` 查表**（原方案已提出）。
3. **`OutputRingBuffer`**：有界环形缓冲，建议默认 2 MiB/session（配置项），存原始字节，不做
   VT 解析——前端 xterm.js 本来就能正确重放任意 ANSI 序列，agent 不需要自己再实现一个终端
   模拟器。超出容量后从头部丢弃最旧的字节。
4. **重连后的补发协议**（新增协议消息，见 §4.3）：连接建立后，agent **不主动推送**任何存量
   会话列表——由插件对它记得的每个 `sessionId` 主动发 `terminal.reattach{sessionId,
   fromOffset}`，agent 查 `sessions` 表：
   - 找不到该 id → 回 `terminal.error{sessionId, SESSION_NOT_FOUND}`，插件按"这个终端已经
     没了"处理（等价于今天开一个新终端）。
   - 找到但 `fromOffset` 早于缓冲区最旧的字节 → 回 `terminal.reattached{resumedFromOffset:
     <缓冲区最旧偏移>, gapped: true}`，随后把整个缓冲区内容重放。
   - 找到且 `fromOffset` 在缓冲区范围内 → 回 `terminal.reattached{resumedFromOffset:
     fromOffset, gapped: false}`，重放 `fromOffset` 之后的部分。
   - 收到 `terminal.reattach` 后该 session 标记 `attached = true`，正常双向收发恢复。
5. **孤儿会话的存活时长**：控制连接消失后（relay 不再主动通知，见 §3.5），agent 把对应
   session 标记 `attached = false`、记下 `idle_since_orphaned`。加一个后台 tick：孤儿状态
   超过可配置阈值（建议默认 30 分钟）就正常终止该 PTY，释放资源——这不是"断线重连"的一部分，
   是防止"忘记关的终端"无限攒着吃内存/进程数的兜底，数值给成配置项而不是写死。
6. **写线程模型**（原方案已提出，直接采纳）：每个 `PtySession` 起一条专用写线程处理
   `write_input`，避免 `spawn_blocking` 乱序导致字节流损坏；teardown 走
   `tokio::task::spawn_blocking` 让多个 session 的 `kill_process_group` 并行退出，而不是
   串行等 2N 秒。
7. **会话数熔断**：不设面向用户的硬上限，但加一个非常宽松的软上限（建议 64，配置项），
   超过时 `terminal.open` 回 `terminal.error{TOO_MANY_SESSIONS}`。这是 §（安全讨论）里提到
   的"配对码泄露后无限开 PTY"的兜底，正常使用永远碰不到。

### 4.3 协议新增（`protocol/schema/messages/`、`protocol/generated/`）

只需要两条新消息，其余复用现有信封结构（`deviceId`+`sessionId`+`requestId` 已经够用）：

```jsonc
// plugin -> agent（走 control 连接的 control 通道，不是 file 通道）
{ "type": "terminal.reattach", "sessionId": "<uuid>", "requestId": "<uuid>",
  "payload": { "fromOffset": 12345 } }

// agent -> plugin
{ "type": "terminal.reattached", "sessionId": "<uuid>", "requestId": "<echo>",
  "payload": { "resumedFromOffset": 12345, "gapped": false } }
```

`terminal.error` 复用现有 schema，新增两个 `code` 枚举值：`SESSION_NOT_FOUND`、
`TOO_MANY_SESSIONS`。改完在 `protocol/` 跑一遍 `npm run generate && node
scripts/sync-protocol.js` 同步到 Rust/TS 两端生成代码。

二期的每会话流控（`terminal.credit`，方向仿照 `creditWindow.ts` 反过来）本方案不展开，
接口层面唯一要求是：`Session` 的输出路径已经是"per-session 状态 + 独立打标"，以后插入
credit 检查点是局部改动，不需要再动数据结构。

---

## 5. 插件改动（`src/`）

### 5.1 配对与连接（吸收需求 1）
- `relayClient.ts`：删除 `login`/`createPairingCode`/`revokePairingCode`/`listDevices`/
  `deleteDevice`。`connectControl()` 改为接收 `{deviceId, pairingCode}`，用
  `Authorization: Device <deviceId>.<pairingCode>` 开 WS，不再有 JWT/过期概念。
- `settingsTab.ts`：登录名/密码/设备列表/创建配对码按钮全部删除，换成"粘贴配对码 + 设备
  别名"的一个输入区。粘贴后调一次 `POST /v1/control/resolve {pairingCode}`（新增的小接口，
  免认证，仅返回 `{deviceId, deviceName, platform}` 用于展示确认，不建立连接）确认码有效，
  本地记住 `{deviceId, pairingCode, label}` 列表——这就是插件侧"我的设备"的全部存储，不再
  依赖服务端提供列表。

### 5.2 多设备（吸收需求 3 的跨设备部分）
- 插件本地维护 `Map<deviceId, RelayControlConnection>`，每台要用的设备各开一条独立连接，
  互不共享 `ConnHandle`/mpsc 队列——这是设备级配对模型的自然结果，不需要额外设计：一台设备
  的输出风暴物理上碰不到另一台设备的连接。
- `remoteService.ts` 的单例 `client`/`connection` 字段要拆成按 `deviceId` 索引的集合，
  `disconnect()`/`setRemoteMode()` 等全局操作要按 device 粒度重做（这部分和
  multi-session-plan.md §"插件"里点出的"一个视图关闭牵连所有人"是同一类问题，解法一致：
  引用计数，照抄 `terminalService.ts:329-332` 拆本地服务端的做法）。

### 5.3 同设备多终端 + 重连补发（吸收需求 3 的同设备部分 + 需求 4）
直接采纳 multi-session-plan.md 已经写清楚的落地方式，不重复展开，关键点摘录：
- demux 从 transport 移进连接层，照抄 `ptyClient.ts` 的 `Map<sessionId, listeners>` 形状。
- `createTerminalTransport(deviceId, sessionId?)` 显式接参数；`TerminalView` 自己持有
  `deviceId`+`sessionId`，不再读全局配置。
- 新增：每个 `RemoteTerminalTransport` 记住自己收到的最大 `output offset`；连接断开重连后，
  主动发 `terminal.reattach{sessionId, fromOffset: 本地记的offset}`，收到
  `terminal.reattached{gapped:true}` 时在终端里插入一条"部分输出可能丢失"的提示行，
  `gapped:false` 则无感恢复。
- 两处串台修复（`remoteTerminalTransport.ts` 的 sessionId 为空错误广播、解码失败广播）按
  原方案处理。

---

## 6. 上线顺序建议

破坏性变更（schema 变了，线上唯一那台设备的绑定数据作废），建议顺序：

1. `protocol/` 加两条新消息 + 两个 error code，生成代码，`npm test` 过。
2. relay：数据库迁移 + §3 全部改动，`cargo test --manifest-path relay/Cargo.toml` 过，重点
   补 multi-session-plan.md 提到的"双会话路由""连接断开不牵连其它会话"这两类此前完全空白
   的测试。
3. agent：§4 全部改动，`cargo test --manifest-path agent/Cargo.toml` 过，重点补"断线重连后
   offset 续传正确""孤儿会话超时清理"这两类新场景的测试。
4. 插件：§5 全部改动，`pnpm test:remote` 过。
5. `./e2e-run.sh` 扩出三个新场景：免账号配对全流程、同设备两个并发会话互不干扰、拔网线
   重连后旧终端内容补齐。
6. 线上 relay 清库重新部署，Ubuntu 机器重新走一次新的自注册流程，Obsidian 端重新粘贴配对码。

---

## 7. 明确不做 / 延后

- **P2P/WebRTC 直连（RTCDataChannel）**：评估过，终端场景 NAT 穿透不保证成功，仍然需要信令
  +TURN 兜底，工程量远大于收益。现有 relay 转发的延迟增量对交互式终端可忽略。以后若真要减少
  服务器依赖，`iroh`（直连优先+relay兜底+短ticket配对）比手撸 WebRTC 划算，但那是传输层的
  整体替换，单独立项评估，不和本次改动混在一起。
- **Agent 进程崩溃/主机重启后的会话持久化**：已确认范围是"网络抖动/relay 重启不丢会话"
  （Agent 进程本身存活），不含"Agent 进程本身被杀/重启也要接回同一会话"。后者需要把 PTY
  持有者和网络客户端拆成两个独立生命周期的进程（本质是重新实现一个 tmux/dtach），Ubuntu 上
  可以考虑用 `tmux new-session -A` 顶一层来白嫖这个能力，但 Windows 没有对等方案，本轮不做。
- **每会话流控（credit window）的具体协议形状**：接口层面已经预留（§4.3 末尾），协议消息
  本身留到动工前再定，不阻塞本方案其余部分。
- **会话数硬上限**：不面向用户设限，只加一个远高于实际用量的软熔断（默认 64，配置项）。
