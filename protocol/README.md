# Termy 远程协议包

三端唯一真相源。对应《Termy 远程增强 MVP 技术方案》第 8 章，协议 `protocolVersion = 1`，Schema 包 `1.0.0`。

```
schema/common.schema.json          共享 $defs（UUID、Semver、ErrorCode、SafeRelativePath、FileEntry…）
schema/control-envelope.schema.json 信封基类 + type 枚举
schema/messages/*.schema.json      16 条控制消息，各自 pin 死 type/requestId/sessionId
fixtures/valid/                    16 条正例，每种消息至少一条
fixtures/invalid/                  22 条反例，每条对应一条具体规则
fixtures/frames/                   12 条二进制帧 hex 向量
tools/frame-codec.js               38 字节帧头的参考编解码器 + 偏移计数域
tools/semantic.js                  JSON Schema 表达不了的跨字段规则
tools/validate.js                  编译全部 Schema + 跑 fixtures
```

```bash
npm install && npm test
```

## 落地时做的判断

方案第 8.4 节以表格形式冻结了字段，但有几处留白，这里按 8.3 的通则推导后固化。**这些是需要复核的判断，不是文档原文。**

### `requestId` / `sessionId` 逐条取值

依据 8.3「请求及其直接响应相同；事件为 `null`」：

| 消息 | `requestId` | `sessionId` | 理由 |
| --- | --- | --- | --- |
| `agent.hello` / `agent.helloAck` | UUID，成对 | `null` | hello 有直接响应 helloAck |
| `agent.heartbeat` | `null` | `null` | 8.4 已明确 |
| `terminal.open` / `terminal.opened` | UUID，成对 | `null` / UUID | opened 携带 Agent 新建的 sessionId |
| `terminal.resize` / `terminal.close` | `null` | UUID | 无直接响应，属会话内事件 |
| `terminal.error` | UUID 或 `null` | UUID 或 `null` | 会话建立前失败时两者都为 `null` |
| `terminal.shellEvent` | `null` | UUID | 纯事件 |
| `transfer.start` / `transfer.accepted` | UUID，成对 | `null` | 传输独立于终端会话 |
| `transfer.credit` / `fileEnd` / `complete` / `abort` / `result` | `null` | `null` | 均按 transferId 关联，非直接响应 |

其中 **`transfer.result` 的 `requestId` 取 `null`** 值得注意：它虽然是整次传输的结局，但 `transfer.start` 的直接响应是 `transfer.accepted`，两者之间可能隔很久，因此按 transferId 关联而非 requestId。

### `additionalProperties: false` 的实现方式

方案 8.1 要求「MVP 的 Schema 一律 `additionalProperties: false`」。在 Draft 2020-12 里，`additionalProperties` 看不见 `$ref` 引入的属性，直接照写会把信封的六个字段全判为多余。因此采用等价且正确的写法：

- **叶子对象**（各 `payload`）用 `additionalProperties: false`；
- **消息顶层**用 `unevaluatedProperties: false`，它能看见 `$ref` 已求值的属性。

约束强度与原文一致：未知字段一律拒绝。`fixtures/invalid/envelope-extra-property.json` 和 `payload-extra-property.json` 分别锁住这两层。

### 由代码而非 Schema 强制的规则

`tools/semantic.js`。JSON Schema 表达不了跨字段关系，但它们同样是契约的一部分，三端都必须实现：

- `entries[i].index === i`（连续无缺口）
- `rootNote === entries[0].relativePath`
- `relativePath` 在整批内唯一
- 累计 `size <= 256 MiB`
- `relativePath` 的 **UTF-8 字节数** `<= 1024`（Schema 的 `maxLength` 数的是 UTF-16 码元，只能作上界）
- `transfer.result`：`code === null` 当且仅当 `success === true`
- `terminal.close`：`reason=shell_exited` 时 `exitCode` 不得为 `null`
- 控制帧编码后 `<= 65536` 字节

### 路径 pattern 覆盖到的

`SafeRelativePath` 的正则挡住：绝对路径、`C:` 盘符、反斜杠、`..` 段、`.` 段、空段（`//`）、结尾 `/`、NUL。**Windows 保留设备名（`CON`、`NUL`、`AUX` 等）、尾随点和空格、大小写冲突不在此处**——它们依赖目标平台，按方案 10.3 属于 Agent 落盘前的最终校验。

### 帧向量

`fixtures/frames/*.hex` 每个文件两行注释加一行 hex，`index.json` 汇总期望。Rust 侧解码器必须对同一组向量给出相同的接受/拒绝结论。`OffsetTracker` 实现 8.5 的分级校验：文件帧偏移不连续为致命错误，终端帧只报告。
