# Termy 远程增强

从 Windows 上的 Obsidian 操作另一台机器的终端。

技术方案见《实现方案 - 开发版 v2.0.md》与《开发计划 v2.0.md》（同目录）；本目录是实现侧的文档。

> **v2.0 换了架构**：不再是"插件 ⇄ 云端 Relay ⇄ Agent"的账号+中继模型（那是 V1，见下方"V1 遗留"）。v2.0 免账号、免自建服务端，插件与 Agent 通过 `iroh`（QUIC）直连，配对靠"复制一段连接码"，不是登录+配对码。

---

## 组成

```
Obsidian 插件 ──iroh(QUIC)──> Termy Agent
（控制端，任意支持 Obsidian 桌面版的平台）    （目标端，Windows / Ubuntu Server）
```

控制端与目标端优先建立点对点直连（NAT 打洞），协调阶段默认经 iroh 官方托管中继完成，直连打通后中继不再承载数据；无论走哪条路径，QUIC/TLS 1.3 都保证端到端加密，中继不具备解密能力（详见 [privacy-and-limits.md](../使用/privacy-and-limits.md)）。

| 目录 | 内容 |
| --- | --- |
| `agent/` | 目标端 Agent：设备身份（Ed25519）、iroh Endpoint、多会话 PTY、单控制端占用 |
| `src/services/remote/` | 插件端远程逻辑：连接码校验/解析、设备配对与持久化、终端流帧协议、`DeviceConnectionManager` |

---

## 文档

| 文档 | 给谁看 |
| --- | --- |
| [building.md](building.md) | 从源码构建 Agent 和插件 |
| [operations.md](../使用/operations.md) | 安装 Agent、CLI 参考、排障 |
| [privacy-and-limits.md](../使用/privacy-and-limits.md) | 使用者。隐私披露和已知限制 |
| [plugin-handover.md](plugin-handover.md) | 接手插件端 UI 接线的人 |
| [plugin-agent-prompt.md](plugin-agent-prompt.md) | 给 Windows 侧编码助手的启动提示词 |

---

## 开发

```bash
# Agent（Rust）
cargo test --manifest-path agent/Cargo.toml

# 插件端远程模块（需要 Node 22）
pnpm test:remote

# 端到端：起一个真实 --loopback agent，用插件同款 @number0/iroh binding 连它、
# 跑一次真实 shell 会话
pnpm install && cargo build --manifest-path agent/Cargo.toml && ./e2e-run.sh
```

出 release 产物见 [building.md](building.md)。

---

## 当前状态

| | |
| --- | --- |
| A0（原生 iroh 绑定能否在 Obsidian Electron 渲染进程里直接加载） | ✅ 已实测确认可行（2026-07-31），不需要 `termy-bridge` 兜底进程 |
| Agent：设备身份、iroh Endpoint、连接码、单控制端占用、多会话 PTY | ✅ 已实现，测试通过（含真实回环 QUIC 集成测试） |
| 插件端远程逻辑（连接码解析、设备配对与持久化、帧协议、`DeviceConnectionManager`） | ✅ 已实现，测试通过，**尚未接入任何 UI/命令面板** |
| 插件端 Obsidian 集成（设备列表、添加设备 UI、打开远程终端） | ❌ 未开始——见 [plugin-handover.md](plugin-handover.md) |
| 端到端（agent 回环 + 真实 iroh 客户端） | ✅ `./e2e-run.sh`，真实 shell 回显 + resize |
| 文件传输（笔记发送到设备，doc §8.4/8.6/10） | ✅ 已实现，测试通过（含真实回环 QUIC 集成测试）；协议改为 `transferManifest` 等新帧搭载在 `termy/terminal/1` 连接上，而非独立的 `ALPN_TRANSFER`（见下一条的同一约束）与文档 §8.3 设想的 `iroh-blobs` 方案——理由见 [目录树候选需求对应的技术方案](../需求/需求文档%20-%20候选%20-%20目录树与双向文件传输.md) 及其技术方案文档的实现状态说明 |
| 目录树列出/监听、"复制到 Vault"回传（候选需求，非 v2.0 原始范围） | ✅ Agent 侧与插件侧均已实现并测试；同样搭载在 `termy/terminal/1` 连接上而非独立 ALPN——`ControllerGate` 只 admit 一个*连接*不是一个*peer*，这也是上一行放弃独立 `ALPN_TRANSFER` 的原因；插件侧尚未接入 UI，见下一条 |
| 真机验收（Windows 进程树终止、非同一局域网双路径） | 🟡 部分完成，见《交接结果 - Windows.md》（同目录）；仍有未验证项 |

**没有在真实硬件上验证过的部分**：非同一局域网的双路径闭环（直连打洞 vs. 降级中继）、iroh 全局节点发现在真实换网场景下的重连。这些在 [privacy-and-limits.md](../使用/privacy-and-limits.md) 里有对应披露。

---

## V1 遗留

`relay/`、`protocol/`、`src/protocol/generated/`、`src/services/remote/relayClient.ts` 等是 V1（账号 + 云端 Relay）的实现，**代码仍在仓库里且仍可编译测试**，但 v2.0 的 `agent/` 已经不再连接它——`agent` 的 relay 客户端（原 `client.rs`）已被删除，`agent bind` 子命令已不存在。是否/何时清理这批 V1 代码尚未排期，见开发计划的"移除/标记废弃 V1 遗留模块"一项。
