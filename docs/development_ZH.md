# 开发指南

想参与贡献 Termesh 的人从这一份文档看起就够了：各部分是什么、怎么构建和测试、提交前要遵守哪些规则。（English version: [development.md](development.md)。）

## 1. 总览

Termesh 是一个给 Obsidian 内嵌真实终端的插件，还带一个可选的远程终端功能：Windows/桌面版 Obsidian 作为控制端，通过 `iroh`（QUIC）与目标设备（Windows 或无桌面 Ubuntu）上的 `termesh-agent` 直连，不需要账号，也不需要自建服务端。

| 目录 | 内容 |
| --- | --- |
| `src/` | TypeScript 插件本体。`src/services/` 放运行时/集成逻辑（`terminal/`、`server/`、`codexCli/`、`context/`、`remote/`），`src/ui/` 放视图和弹窗，`src/settings/` 放设置模型与渲染器，`src/i18n/` 放多语言，`src/utils/` 放共享工具函数。 |
| `agent/` | Rust 编写的远程终端 Agent（`termesh-agent`）：设备身份、`iroh` Endpoint、连接码配对、多会话 PTY 服务。 |
| `rust-servers/` | 插件通过本地 WebSocket 连接的本机 PTY 后端。 |
| `relay/`、`protocol/` | V1（账号 + 云端 Relay）遗留实现——见[第 7 节](#7-v1-遗留代码)。 |
| `docs/` | 本文档及其引用的截图/素材。 |
| `scripts/` | 构建、打包、发布相关的 Node 脚本。 |
| `e2e/` | 端到端驱动脚本，用真实回环 Agent + 真实 `@number0/iroh` binding 跑一遍。 |

构建产物（仓库根目录的 `main.js`、`styles.css`，以及 `binaries/`）由构建过程生成，不作为源码提交。

## 2. 前置条件

- **Rust**——版本由 `rust-toolchain.toml` 锁定，`rustup` 会自动拉取，不要手动装别的版本。
- **Node.js 22**——插件测试套件依赖 `--experimental-strip-types`。手头只有 Node 18 时的退路见[第 4 节](#4-测试)。
- **pnpm**——版本见 `package.json` 的 `packageManager` 字段。
- **要在 Windows 上构建 Agent 的贡献者**：必须在 Windows 本机构建，见[第 3.3 节](#33-rust-agenttermesh-agent)。

## 3. 构建

### 3.1 首次搭建

```bash
pnpm install
pnpm build                # tsc --noEmit + esbuild + 打包体积自检 -> main.js
```

需要对着真实 vault 迭代开发时：

```bash
pnpm install:dev <vault-path>   # 构建两层并装进该 vault
pnpm dev                        # esbuild watch 模式
```

只改了 TypeScript、没碰原生 PTY 服务端时，给 `install:dev` 加 `--no-rust` 可以跳过那部分重新构建。

### 3.2 插件打包

```bash
pnpm package        # 组装出可分发的 plugin-package/ 目录
pnpm package:zip     # 打成 termesh-<version>.zip
```

`pnpm package` 产出 `main.js` + `manifest.json` + `styles.css` + `node_modules/@number0/`（远程终端功能依赖的原生模块，见下）。打包后确认没有遗漏符号链接：

```bash
find plugin-package/node_modules -type l   # 应该没有任何输出
```

**`@number0/iroh` 原生模块**：这是远程终端功能依赖的 N-API 模块。`esbuild.config.mjs` 把它标为 `external`，因为安装好的 Obsidian 插件目录本身没有 `node_modules`。分发路径有两条：社区市场/BRAT 安装的插件会在首次使用远程设备时，从 unpkg、jsDelivr 或 GitHub Release 下载对应平台的固定版本 `.node` 文件，并用内置的 SHA-256 校验；离线完整包则直接携带这个模块。`scripts/package-plugin.js` 的第 5b 步会通过 `require.resolve()`（而不是硬编码的平台映射表——pnpm 的隔离 store 会把这些包安装成 `node_modules/.pnpm/` 下的符号链接，平台矩阵本身也会变）找出当前平台真正装上的那个原生包，解引用后拷进 `plugin-package/node_modules/@number0/`。两条分发路径都要求在目标操作系统和架构上构建——`pnpm install` 只会拉当前平台对应的原生包。

### 3.3 Rust Agent（`termesh-agent`）

**Linux：**

```bash
cargo build --manifest-path agent/Cargo.toml --release
./agent/packaging/install-linux.sh agent/target/release/termesh-agent
```

安装脚本**拒绝以 root 身份运行**——要以将来实际使用它的那个普通用户身份安装。它会把二进制装到 `~/.local/bin`、把 systemd user unit 装到 `~/.config/systemd/user`，并执行 `loginctl enable-linger`（通常需要一次 root/polkit 认证的那一步）。装完之后不需要额外配对：启动服务，复制打印出来的连接码，粘贴进插件即可。

**Windows：只能在 Windows 本机构建。** 这不是图省事的问题，是几条硬阻塞：

1. `agent/Cargo.toml` 的 `[target.'cfg(windows)'.dependencies]` 依赖 `windows-sys`——进程树终止（Job Object）和存活检测（`OpenProcess`/`GetExitCodeProcess`）都是 Windows 专属实现；
2. `portable-pty` 在 Windows 上走 ConPTY，链接 Windows 系统库；
3. `x86_64-pc-windows-msvc` 需要 MSVC 链接器，Linux 上没有。

```powershell
rustup toolchain install <rust-toolchain.toml 锁定的版本>
cargo build --manifest-path agent\Cargo.toml --release
# 产物：agent\target\release\termesh-agent.exe
```

Windows 侧目前没有开机自启的安装脚本，用任务计划程序或注册为服务；要保持运行的命令是 `termesh-agent.exe run`。

## 4. 测试

```bash
cargo test --manifest-path agent/Cargo.toml     # Rust 单测 + 真实回环 QUIC 集成测试
pnpm test:remote                                # 插件端远程模块，需要 Node 22
pnpm test:terminal                              # 本地终端层回归测试
pnpm lint                                       # 通用 ESLint 配置，可选、起补充作用
```

**`pnpm lint:obsidian` 是强制项，不是可选项**——只要改动涉及 `src/**/*.ts`，就要跑这个命令，修完所有违规、跑到干净为止，才算改完。它用的是 `eslint.obsidian.config.js`，比通用配置多强制了几条规则：`@microsoft/sdl/no-inner-html`、`@typescript-eslint/no-base-to-string`、`@typescript-eslint/no-redundant-type-constituents`、`@typescript-eslint/no-unnecessary-type-assertion`、`@typescript-eslint/require-await`、`obsidianmd/ui/sentence-case-locale-module`。它会忽略 `src/**/*.test.ts`、`scripts/`、`rust-servers/`、`binaries/`、`plugin-package/`、`main.js`、`styles.css`——改动完全落在这些路径内时可以跳过，否则就要跑。

Rust 侧 CI 门槛：

```bash
cargo fmt --manifest-path agent/Cargo.toml --check
cargo clippy --manifest-path agent/Cargo.toml --all-targets -- -D warnings
```

端到端（真实回环 Agent + 真实 `@number0/iroh` binding，验证真实 shell 回显与 resize；**不覆盖文件传输**）：

```bash
pnpm install
cargo build --manifest-path agent/Cargo.toml
./e2e-run.sh
```

**手头只有 Node 18？** `pnpm test:remote`/`pnpm test:terminal` 没法直接跑，因为依赖 `--experimental-strip-types`。可以先用仓库自带的 `tsc` 转译，再拿 Node 18 跑转译产物——转译后跑会有两个已知失败：`relayClient.test.ts`/`remoteService.test.ts`（V1 遗留测试），原因是这条路径下 `ws` 包解析不到，跟 v2.0 代码无关，属于预期内、可以忽略。

## 5. 代码风格

- TypeScript 用 2 空格缩进，Rust 用 4 空格。
- TypeScript 偏 `strict`：保持单引号、分号，能提升可读性的地方写明确类型。
- 类和 UI 类型用 `PascalCase`，函数和变量用 `camelCase`，文件名用有描述性的小驼峰（如 `terminalPathUtils.ts`、`settingsTab.ts`）。
- 代码注释一律用英文，不管你改的是哪个语言的 locale 文件。

## 6. Obsidian 开发者政策——不可触碰的红线

以下条目直接来自 [Obsidian 官方开发者政策](https://docs.obsidian.md/Developer+policies#Not+allowed)，违反任何一条都会导致插件被社区列表拒绝，已上架的会被下架。拿不准时就问自己一句："一个理性的人会不会把这个描述成下面某一条？"——会的话就不要做。

- **不能混淆代码来隐藏其用途。** `main.js` 用 esbuild 的 `minify: true` 压缩体积是允许的，因为 GitHub 上的可读 TypeScript 源码才是上游真相——不要在此之上再加编码字符串、运行时解码的函数体、不透明打包器或基于 eval 的加载器。
- **不能插入广告**，无论动态还是静态，除非它出现在 Termesh 自己的界面里（设置页、自己的弹窗/视图），且确实与 Termesh 本身相关。
- **不能包含客户端遥测。** 不允许分析 SDK、使用情况上报、错误上报端点，或任何"首次运行打个招呼"式的行为。允许的出站网络调用只有：从 GitHub Release 下载对应版本的 `termy-server` 二进制；下载固定版本、对应平台的 iroh 原生运行时（unpkg/jsDelivr/GitHub Release，SHA-256 校验，离线模式下禁用）；到 PTY 后端和 Claude Code IDE bridge 的纯本地 WebSocket 连接；以及默认关闭的 AI 启动器更新检查（同样受离线模式抑制）。这个开关必须保持默认关闭。
- **不能包含插件自更新机制。** 只有 Obsidian 自己的插件更新器能替换 `main.js`/`styles.css`/`manifest.json`。唯一允许的例外是把匹配的原生 `termy-server` 二进制下载到 `<plugin>/binaries/`——它是单独版本化的原生资源，经 SHA-256 校验，受离线模式禁用控制，且绝不会覆盖插件的 JS/CSS/JSON 文件。
- **不能加载网络资源。** 所有字体、图片、图标都要通过 esbuild 的 `loader` 表或 `assets/` 打进包里，样式表里不要出现 `https://...` 的引用。

## 7. V1 遗留代码

`relay/`（云端 Relay 服务端）和 `protocol/`（三端协议契约生成器）是 V1（账号 + 云端 Relay）的实现。当前 Agent 已经不再连接任何 Relay——相关客户端代码已经删除——但插件侧仍有一部分 V1 模块（`relayClient.ts`、`remoteService.ts`、`authClient.ts`、`deviceClient.ts` 等）在被实际使用，并且依赖 `protocol/generated/` 生成的类型。**不要在没有单独、明确的移除任务的前提下删除或大改它们**——它们支撑的功能仍在正常使用。这两个目录仍可独立编译测试，CI 也一直在跑，防止它们不知不觉烂掉：

```bash
cd protocol && npm ci && npm test
cargo test --manifest-path relay/Cargo.toml
```

除非你确实要改这部分代码，否则不需要构建或重新生成它们。

## 8. 测试用例的隐私要求

不要在测试、快照、文档或示例数据里提交真实的个人信息——本地用户名、绝对的用户主目录路径、云存储路径、Vault 名称、私人笔记/课程/考试相关的文件夹名，或任何看起来像凭据、API key 的东西。改用中性的示例代替，比如 `/Users/example/Documents/Notes/Example.md`、`F:\example-vault\notes\demo`、`notes/路径示例.md`、`archive/12345/`。提交带路径或笔记名的测试改动前，扫一遍改动过的文件，确认没有不小心带进去的私人字符串。

## 9. 提交与 PR

- 提交信息开头要是 Conventional Commit 的类型之一：`feat:`、`fix:`、`chore:`、`docs:`、`refactor:`、`test:`、`build:`、`ci:`、`perf:`、`style:`、`revert:`；能让意思更清楚时鼓励加 scope（如 `feat(terminal):`、`fix(settings):`）；前面加个 emoji 也可以，只要必需的类型前缀还在。
- 提交信息保持简短、祈使句、说清楚具体改了什么。
- PR 要说明用户可见的影响、本地做过哪些验证、关联相关 issue；涉及 UI 的改动要附截图或短录屏。
- 只要涉及打包、发布说明或带版本号的行为变化，就要更新 `CHANGELOG.md`，让发布自动化能把改动映射到正确的版本段落。

## 10. 想找更早的设计历史？

原来放在 `docs/需求/` 和 `docs/开发/` 下的早期草案、分阶段实现方案，以及一次性的交接/验收清单，现在已经把仍然有用的部分并进了这份文档，其余随着代码落地已经过时的部分不再保留。需要那个细节层级时，`git log -- docs/` 里还在。
