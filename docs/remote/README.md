# Termy 远程增强

从 Windows 上的 Obsidian 操作另一台机器的终端，并把笔记连同附件推送过去。

技术方案见仓库外的《Termy 远程增强 MVP 技术方案》；本目录是实现侧的文档。

---

## 组成

```
Obsidian 插件 ──HTTPS/WSS──> 云端 Relay <──WSS── Termy Agent
（控制端，Windows）           （单节点）        （Windows / Ubuntu Server）
```

Agent 主动外连，**不监听公网端口**。Relay 只转发，不持久化任何正文。

| 目录 | 内容 |
| --- | --- |
| `protocol/` | 三端唯一真相源：JSON Schema、OpenAPI、fixtures、帧向量、类型生成 |
| `relay/` | 云端 Relay：HTTPS API + WSS 网关 |
| `agent/` | 目标端 Agent：PTY、文件接收、退避重连 |
| `src/services/remote/` | 插件端远程逻辑 |
| `src/protocol/generated/` | 从 `protocol/` 同步过来的类型，**不要手改** |

---

## 文档

| 文档 | 给谁看 |
| --- | --- |
| [building.md](building.md) | 从源码构建四个产物。含哪个产物必须在哪台机器上编 |
| [operations.md](operations.md) | 部署 Relay、安装 Agent、CLI 参考、排障 |
| [privacy-and-limits.md](privacy-and-limits.md) | 使用者。隐私披露和已知限制 |
| [plugin-handover.md](plugin-handover.md) | 接手插件端改造的人。含已踩过的坑 |
| [plugin-agent-prompt.md](plugin-agent-prompt.md) | 给 Windows 侧编码助手的启动提示词，贴之前替换占位符 |
| [multi-session-plan.md](multi-session-plan.md) | 支持多个并发远程终端的实现方案（尚未动工） |
| [`protocol/README.md`](../../protocol/README.md) | 协议契约，以及落地时做的判断 |

---

## 开发

```bash
# 协议：Schema、OpenAPI、fixtures、帧向量、类型生成
cd protocol && npm ci && npm test

# Rust 两端
cargo test --manifest-path relay/Cargo.toml
cargo test --manifest-path agent/Cargo.toml

# 插件端远程模块（需要 Node 22）
pnpm test:remote

# 端到端：起 relay + agent，跑真实终端命令和真实文件传输
./e2e-run.sh
```

工具链由 `rust-toolchain.toml` 锁定。协议类型是生成的并已提交，CI 会重新生成并 diff——改了 Schema 记得跑 `cd protocol && npm run generate && node scripts/sync-protocol.js`。

出 release 产物、以及 Windows Agent 与插件怎么构建，见 [building.md](building.md)。

---

## 当前状态

| | |
| --- | --- |
| 协议契约 | ✅ 三端一致性由共享 fixtures 与帧向量保证 |
| Relay | ✅ 6 个接口 + WSS 网关，71 项测试 |
| Agent | ✅ 全部功能，33 项测试 |
| 插件端远程逻辑 | ✅ 63 项测试 |
| 插件端 Obsidian 集成 | ✅ 已实现，待真机验收；见 [plugin-handover.md](plugin-handover.md) |
| 端到端（Linux 本机） | ✅ 8 项检查 |
| 真机验收（§16） | ❌ 需要 Windows 与 Ubuntu 实机 |

**没有在真实硬件上验证过的部分**：Windows 的 Job Object 进程树终止（CI 只做类型检查）、`loginctl enable-linger` 的实际权限要求、非同一局域网的双路径闭环、Dockerfile 的实际构建。这些在对应文档里都单独标注了。
