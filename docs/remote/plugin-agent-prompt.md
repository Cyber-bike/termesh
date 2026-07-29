# 插件端实现 — 给 Windows 侧编码助手的启动提示词

把下面 `---` 之间的内容整段贴给 Windows 机器上的 Claude Code。

**贴之前替换两处占位符**：`<RELAY_PASSWORD>` 和 `<VAULT_PATH>`。口令不写进仓库——这是公开仓库，凭据一旦提交就等于泄露，即使随后删除也已进入 git 历史和各种镜像。

---

你要在 Windows + Obsidian 环境下完成 Termy 远程增强的**插件端实现**。后端（协议、云端 Relay、目标端 Agent）已全部完成、测试通过并部署上线；缺的只有插件端里依赖 Obsidian 运行时的那部分。

## 先做这三件事，不要跳过

1. `git clone https://github.com/jiang-zhong-xi/ReqFirst.git` 然后 `cd ReqFirst`
2. **完整读 `docs/remote/plugin-handover.md`**。它是本任务的权威依据，尤其是第 2 节"已经踩过的坑"——那六条全是实测撞出来的，读代码看不出来，按它写能省掉几天返工。
3. 跑一遍现有测试确认基线是绿的：
   ```powershell
   pnpm install
   pnpm test:remote      # 52 项，需要 Node 22
   pnpm test:terminal
   ```
   基线不绿先查环境（Node 版本、pnpm 版本 10.33.4），**不要在红的基线上开工**。

## 已经能用的东西

`src/services/remote/` 下的纯逻辑全部写完并有测试覆盖：帧编解码（`frameCodec.ts`，已与 Rust 两端对过 hex 向量）、路径安全（`pathSafety.ts`）、附件收集（`noteCollector.ts`）、信用窗口（`creditWindow.ts`）、发送流程（`transferSender.ts`）、UI 状态机（`remoteState.ts`）、Transport 接口定义（`transport.ts`）。

**直接用，不要重写。** 它们的行为被 52 项测试锁定，其中好几项锁的是踩过坑之后的正确行为。

## 一个已经在线的真实后端

不需要自己搭环境，云端 Relay 已经部署好了：

```
地址      https://bjev.duckdns.org
用户名    chandler
口令      <RELAY_PASSWORD>
```

证书是 Let's Encrypt 签的，Obsidian 和 Agent 都认。验证它活着：

```powershell
curl https://bjev.duckdns.org/health          # 应返回 ok
```

注意健康检查路径是 `/health`，**没有 `/v1` 前缀**。

控制连接的地址是 `wss://bjev.duckdns.org/v1/control/ws`。

## 硬约束

- **`src/protocol/generated/messages.ts` 不许手改。** 它是从 `protocol/` 生成后同步过来的，CI 会重新生成再 diff，改了必红。要改协议就改 `protocol/schema/` 再 `pnpm sync:protocol`。
- **本地终端零回归是第一优先级。** `terminalInstance.ts` 有 2287 行、15 处以上直接调 `this.ptyClient.*`，这是侵入式重构不是薄封装。先只做 `LocalTerminalTransport` 把本地路径切过去，**完整手测一遍本地终端**（输入输出、复制粘贴、搜索、resize、Ctrl-C、清屏、本地拖拽），确认没坏再碰远程。
- **拖拽分流点在 `terminalView.ts:557` 的 `handleDrop` 里、调用 `buildDroppedInput`（第 570 行）之前。** 方案明确禁止先生成本地绝对路径再判断远程模式。
- **控制 WSS 必须用 `ws` 包，不能用浏览器原生 `WebSocket`**——原生的设不了 `Authorization` 头。`ws@8.20.1` 已经是依赖。
- **`TERMY_AGENT_ALLOW_INSECURE` 之类的明文开关不许进发布版。** 现在有真实 HTTPS 后端了，本来也不需要。
- 改 `pathSafety.ts` 时**别在编辑器里直接敲控制字符**——那个正则被原始控制字节污染过一次，变成取反字符类后拒绝所有路径，而且 grep 会把文件当二进制导致搜索静默无输出，极难排查。

## 实现顺序

每一步都能独立验证，按序做，别并行：

1. **`LocalTerminalTransport` + `TerminalInstance` 重构** → 停下来完整回归本地功能。风险最大的一步，先隔离掉。
2. **`relayClient.ts` 的 HTTPS 部分 + 设置面板**（服务器地址、登录、生成配对码、设备列表）。接口契约见 `protocol/openapi.yaml`。访问令牌 15 分钟过期且**没有 refresh**，过期后重新登录。
3. **用真机 Agent 配对一台设备**，确认设备列表里显示在线。
4. **控制 WSS + `RemoteTerminalTransport`** → 远程终端可用。四条事件通道一条都不能少，少了 `onShellEvent` 会让 cwd 跟踪和 AI CLI 上下文交接全部失效。
5. **拖拽分流 + 文件传输** → 远程传文件可用。
6. 按技术方案 §16 完成定义逐条验收。

## 需要你在真机上确认的假设

这几处是在无桌面 Linux 上写的，**从未在 Obsidian 里跑过**，可能是错的：

- `vaultLinkSource.ts` 用 `metadataCache.getFileCache()` 的 `links`/`embeds`（而非 `resolvedLinks`，后者按解析后路径索引、丢掉了原始链接文本，而判断"未解析的链接是不是附件"必须要原文）。这个假设需要验证。
- `getFileByPath` / `getFirstLinkpathDest` / `getFileCache` 按 Obsidian API 1.12.3 写的，确认签名没变。
- 未解析链接的处理：只有**看起来是附件**的链接解析失败才整批拒绝，指向 Markdown 的未解析链接直接忽略。真实 vault 里验证一下这个判断够不够宽松——Obsidian 里指向尚未创建笔记的 wiki 链接是常态。

## 开发环境

Obsidian vault 在 `<VAULT_PATH>`。`pnpm install:dev` 会把构建产物装进去，`pnpm dev` 起 watch。

`pnpm build:rust` 构建的是 **Termy 自己的本地终端服务端**（`rust-servers/`），**与远程功能的 `termy-agent` 毫无关系**，远程功能不需要跑它。别搞混。

其余构建细节见 `docs/remote/building.md`。

## 报告要求

每完成一步，明确说清：**改了哪些文件、跑了哪些测试、结果是什么、哪些是手测的、哪些还没验证**。不要把"应该能work"说成"已完成"。如果某个假设在真机上被推翻了（很可能发生），直接说，并把 `plugin-handover.md` 对应段落改掉。
