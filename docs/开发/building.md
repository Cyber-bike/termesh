# 构建指南

从源码构建两个产物：`termy-agent`（目标端）和插件 `main.js`（控制端）。

v2.0 不再需要构建/部署云端 Relay——见下方"V1 遗留"，那部分只有你要碰 V1 遗留代码才用得上。

---

## 0. 哪个产物在哪台机器上构建

| 产物 | 构建机 | 备注 |
| --- | --- | --- |
| `termy-agent`（Linux） | Linux | — |
| `termy-agent.exe`（Windows） | **必须在 Windows 本机** | 见 §2.2，不能交叉编译 |
| 插件 `main.js` | Node 22 + pnpm | 打包分发还需要 `@number0/iroh` 的平台原生模块，见 §3.2 |

---

## 1. 前置

**Rust**：版本由 `rust-toolchain.toml` 锁定，rustup 会自动拉取，不要手动指定别的版本。

**Node**：插件端的测试要 **22**（依赖 `--experimental-strip-types`）；插件打包要 **pnpm**（版本见 `package.json` 的 `packageManager` 字段，当前 `10.33.4`）。

---

## 2. Agent

### 2.1 Linux

```bash
cargo build --manifest-path agent/Cargo.toml --release
./agent/packaging/install-linux.sh agent/target/release/termy-agent
```

安装脚本**拒绝以 root 运行**——Agent 要以将来使用它的那个普通用户身份安装。它做三件事：装二进制到 `~/.local/bin`、装 user unit 到 `~/.config/systemd/user`、`loginctl enable-linger`（唯一需要 sudo 的一步，多数发行版需要一次 root 或 polkit 认证）。

装完之后**不需要任何配对步骤**——v2.0 免账号、免配置。启动服务、读连接码、贴进 Obsidian 即可，具体见 [operations.md](../使用/operations.md) §1。

### 2.2 Windows：只能在 Windows 本机构建

```powershell
rustup toolchain install <rust-toolchain.toml 里锁定的版本>
cargo build --manifest-path agent\Cargo.toml --release
# 产物 agent\target\release\termy-agent.exe
```

**不要试图从 Linux 交叉编译**，几条理由都是硬阻塞：

1. `agent/Cargo.toml` 在 `[target.'cfg(windows)'.dependencies]` 里依赖 `windows-sys`——进程树终止（Job Object）和 `status` 的存活检测（`OpenProcess`/`GetExitCodeProcess`）都是 Windows 专属实现；
2. `portable-pty` 在 Windows 走 ConPTY，链接 Windows 系统库；
3. `x86_64-pc-windows-msvc` 需要 MSVC 链接器，Linux 上没有。

Windows 侧的开机自启 MVP 没提供安装脚本（`agent/packaging/install-linux.sh` 是 Linux 专属），用任务计划程序或注册为服务，`termy-agent.exe run` 是要跑起来的命令。

---

## 3. 插件

```bash
pnpm install
pnpm build                # tsc --noEmit + esbuild + verify-build → main.js
pnpm package               # 组装成可分发的 plugin-package/ 目录
pnpm package:zip           # 打成 termy-<version>.zip
```

`pnpm build` 产出 `main.js`；`pnpm package` 之后分发目录是 `main.js` + `manifest.json` + `styles.css` + `node_modules/@number0/`（见下）。

### 3.1 本地开发装进 vault

```bash
pnpm install:dev
pnpm dev        # esbuild watch 模式
```

### 3.2 `@number0/iroh` 原生模块分发

`@number0/iroh` 是 v2.0 远程终端功能依赖的原生 N-API 模块（A0 判定：Obsidian Electron 渲染进程可以直接 `require` 它，见 README 的"当前状态"）。esbuild 把它标为 `external`（`esbuild.config.mjs`），因此使用两条分发路径：社区市场或 BRAT 安装在首次使用远程设备时，从同版本 GitHub Release 下载平台 `.node` 文件并严格校验 SHA-256；离线完整包则直接携带整个模块。插件直接加载 N-API 文件，不下载或执行 JavaScript 代码。

`scripts/package-plugin.js` 的第 5b 步会自动做这件事：从已安装的 `@number0/iroh` 读它自己声明的 `optionalDependencies`，用 `require.resolve()` 找出当前平台真正装上的那个原生包（不是硬编码的平台名映射表——pnpm 的隔离 store 会把这些包安装成 `node_modules/.pnpm/` 下的符号链接，而且平台矩阵本身也会变，之前 `darwin-x64` 就消失过一次），把 `@number0/iroh` 和 `@number0/iroh-<platform>` 都**解引用后**（不是符号链接）拷进 `plugin-package/node_modules/@number0/`。

两条路径都要求在目标操作系统和架构上构建：`pnpm install` 只安装当前平台对应的原生包。Release workflow 在各平台 runner 上生成完整包，同时提取 `iroh-runtime-<platform>.node` 及校验文件供在线安装按需下载。

`pnpm package` 之后可以验证一下没有漏东西：

```bash
find plugin-package/node_modules -type l   # 应为空——都应该是真实文件，不是符号链接
```

---

## 4. 验证

```bash
cargo test --manifest-path agent/Cargo.toml     # Agent，含真实回环 QUIC 集成测试
pnpm test:remote                                # 插件端远程模块，需要 Node 22
pnpm test:terminal                              # 本地终端回归（Transport 接口零回归)
pnpm lint
```

手头只有 Node 18、跑不了 `pnpm test:remote`（需要 `--experimental-strip-types`）时，可以用 `tsc` 转译后拿 Node 18 跑（实测 132 项，130 过，2 个因为依赖 `ws` 包在隔离目录里解析不到而失败——那两个是 V1 的 `relayClient.test.ts`/`remoteService.test.ts`，跟 v2.0 代码无关，忽略即可）：

```bash
W=$(mktemp -d)
./node_modules/.bin/tsc src/services/remote/*.ts --outDir "$W" \
  --module ES2022 --target ES2022 --moduleResolution bundler \
  --allowImportingTsExtensions --rewriteRelativeImportExtensions \
  --skipLibCheck --types node
# tsc 只重写显式写了 .ts 后缀的相对导入；没写后缀的（v2.0 新文件的风格）还要补一刀，
# 但不能碰已经是 .js/.json 结尾的，否则会变成 .js.js：
find "$W" -name '*.js' -exec sed -i -E \
  "/from '\.[^']*\.(js|json)'/! s/from '(\.[^']+)'/from '\1.js'/g" {} +
echo '{"type":"module"}' > "$W/package.json"
node --test "$W/services/remote"/*.test.js   # 注意：在仓库根下执行，且要先 pnpm install
```

端到端：

```bash
pnpm install
cargo build --manifest-path agent/Cargo.toml
./e2e-run.sh
```

起一个真实的 `termy-agent run --loopback`，用插件同款 `@number0/iroh` binding（`e2e/loopback-driver.cjs`）以 QUIC 直连、走 doc 8.2 握手、验证真实 shell 回显与 resize。**不覆盖文件传输**——`termy/transfer/1`（Phase C）还没实现，`agent/src/serve.rs` 目前直接用 `PROTOCOL_ERROR` 关掉这条 ALPN。

CI 跑的门槛还包括：

```bash
cargo fmt --manifest-path agent/Cargo.toml --check
cargo clippy --manifest-path agent/Cargo.toml --all-targets -- -D warnings
```

---

## V1 遗留：`relay/` 与 `protocol/`

`relay/`（云端 Relay 服务端）和 `protocol/`（三端协议契约生成器）是 V1 的实现，**仍在仓库里、仍可独立编译测试**，但 v2.0 的 `agent/` 已经不再连接任何 Relay——`agent` 的 relay 客户端代码已被删除。插件端还有一部分 V1 代码（`relayClient.ts`、`remoteService.ts` 等）在用 `protocol/generated/` 里生成的类型，所以这两个目录暂时不能删。

除非你要改这部分 V1 代码，否则**不需要构建 `relay/` 或重新生成 `protocol/` 的类型**——v2.0 的 Agent 构建、插件构建、`e2e-run.sh` 都不依赖它们。CI 仍然单独跑它们的测试和生成物一致性检查（`.github/workflows/remote-mvp.yml` 的 `protocol`/`rust`（Relay 部分）/`generated-rust` job），保证这批遗留代码不会不知不觉地烂掉。

```bash
cd protocol && npm ci && npm test
cargo test --manifest-path relay/Cargo.toml
```

是否/何时整体移除这批 V1 代码尚未排期，见开发计划的相应条目。
