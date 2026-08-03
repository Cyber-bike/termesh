# 插件端 v2.0 UI 接线 — 给 Windows 侧编码助手的启动提示词

把下面 `---` 之间的内容整段贴给 Windows 机器上的编码助手。**不需要替换任何占位符**——v2.0 没有账号、没有云端 Relay 密码，不涉及需要脱敏的凭据。

---

你在仓库 D:\project\ReqFirst（Obsidian 插件 Termy + Rust agent，分支 `v2.0`）上工作。任务：把已经写好、测过、但完全没接进界面的 v2.0 远程终端后端模块，装到真实 Obsidian UI 里，让用户能添加设备、看到设备列表、打开一个真实的远程终端。

## 先做这三件事，不要跳过

1. `git fetch && git pull`，确认基线是最新的。
2. **完整读 `docs/开发/plugin-handover.md`**——它是本任务的权威依据，尤其是第 2 节"已经踩过的坑"，那几条都是实测撞出来的，读代码看不出来。
3. 跑一遍现有测试确认基线是绿的：
   ```powershell
   pnpm install
   pnpm test:remote      # 需要 Node 22
   pnpm test:terminal
   pnpm build
   ```
   基线不绿先查环境（Node 版本、pnpm 版本，见 `package.json` 的 `packageManager` 字段），**不要在红的基线上开工**。

## 已经能用的东西

`agent/`（Rust，iroh Endpoint、多会话 PTY 服务）和 `src/services/remote/`（连接码校验、设备配对与持久化、终端流帧协议、`DeviceConnectionManager`）**全部写完并有测试覆盖，覆盖真实 iroh QUIC 通信，不是占位实现**。直接用，不要重写——具体清单见 `plugin-handover.md` 第 1 节。

`src/main.ts` 和 `src/settings/settingsTab.ts` 目前**完全没有引用**上述任何一个 v2.0 模块，这就是本次任务的全部范围。

## 硬性约束

- **不要动 `agent/` 目录**，这次是插件端任务
- **不要删除或大改** V1 的 `remoteService.ts`、`relayClient.ts` 等文件——它们还在被用着，这次是新增 v2.0 路径，不是替换
- 每一步做完手测一遍，不要攒到最后——UI 部分我这边看不到，没法帮你远程调
- 遇到我记录得和实际代码对不上的地方（`plugin-handover.md` 里的方法名/行号可能会随代码演进漂移），以你读到的真实代码为准，不要卡住

## 报告要求

每完成一步，明确说清：**改了哪些文件、跑了哪些测试、结果是什么、哪些是手测的、哪些还没验证**。不要把"应该能 work"说成"已完成"。如果 `plugin-handover.md` 里某个假设被真机验证推翻了，直接说，并把对应段落改掉再提交。
