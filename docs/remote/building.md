# 构建指南

从源码构建四个产物：协议类型、Relay、Agent、插件。

---

## 0. 哪个产物在哪台机器上构建

| 产物 | 构建机 | 是否在开发机实测过 |
| --- | --- | --- |
| 协议类型（TS + Rust） | 任意（Node 18+） | ✅ |
| `termy-relay`（Linux） | Linux | ✅ 13m58s / 633 MB 峰值 |
| Relay 容器镜像 | 有 Docker 守护进程的机器 | ❌ 本机无 Docker |
| `termy-agent`（Linux） | Linux | ✅ 10m14s / 530 MB 峰值 |
| `termy-agent.exe`（Windows） | **必须在 Windows 本机** | ❌ 见 §4.2 |
| 插件 `main.js` | Node 22 + pnpm | ❌ 本机无 pnpm、Node 仅 18 |

---

## 1. 前置

**Rust**：版本由 `rust-toolchain.toml` 锁定在 1.97.1，rustup 会自动拉取，不要手动指定别的版本。Relay 的传递依赖有硬性下限（`time@0.3.54` 要 1.88、`icu_*@2.2.0` 要 1.86），低于 1.88 直接编不过。

**内存**：release profile 是 `lto = "thin"` + `codegen-units = 1`，比 debug 吃得多。实测峰值 Relay 633 MB。**开发机 961 MB 内存必须先有 swap**，否则链接阶段会被 OOM killer 杀掉：

```bash
sudo fallocate -l 8G /swapfile && sudo chmod 600 /swapfile
sudo mkswap /swapfile && sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

**Node**：`protocol/` 用 18 就够；插件端的测试要 **22**（依赖 `--experimental-strip-types`）；插件打包要 **pnpm**。

---

## 2. 协议包（先构建，Relay / Agent / 插件都依赖它）

```bash
cd protocol
npm ci
npm run generate          # TS 类型 + Rust 类型
cd .. && node scripts/sync-protocol.js
```

三端的依赖关系：

- `relay/` 和 `agent/` 对 `protocol/generated/rust` 是 **path 依赖**，该目录必须存在且已生成；
- 插件消费 `src/protocol/generated/messages.ts`，由 `sync-protocol.js` 从 `protocol/generated/typescript/` 拷过去。

生成物**已提交进仓库**，所以只想编 Rust 的话可以跳过这一节。但**改了 `protocol/schema/` 就必须重新生成并提交**——CI 会用 `npm run generate:check` 和 `node scripts/sync-protocol.js --check` 重新生成再 diff，不一致就红。

`generate:rust` 需要 `cargo install cargo-typify --locked`（装一次要几分钟，CI 里因此单开了一个 job）。只改 TS 侧可以只跑 `node tools/generate-typescript.js`。

---

## 3. Relay

### 3.1 裸机

```bash
cargo build --manifest-path relay/Cargo.toml --release
```

产物 `relay/target/release/termy-relay`，约 11 MB（profile 里 `strip = true`，无需再 strip）。

单核开发机实测 **13m58s**，峰值 633 MB。多核机器会快很多，但 `codegen-units = 1` 限制了并行度——这是有意的取舍，换更小更快的二进制。急着迭代就别加 `--release`。

### 3.2 容器

```bash
docker build -f relay/Dockerfile -t termy-relay .
```

**build context 必须是仓库根目录**，不能是 `relay/`——Dockerfile 要 `COPY protocol/generated/rust`（path 依赖）。

镜像内用 `cargo build --release --locked`，所以 `relay/Cargo.lock` 必须是提交状态且与 `Cargo.toml` 一致，否则构建直接失败（这是想要的行为：镜像不该悄悄升级依赖）。

> ⚠️ Dockerfile 和 compose.yaml 在开发机上**从未实际构建过**（无 Docker 守护进程），首次使用前请自行验证一次。

---

## 4. Agent

### 4.1 Linux

```bash
cargo build --manifest-path agent/Cargo.toml --release
./agent/packaging/install-linux.sh agent/target/release/termy-agent
```

产物 `agent/target/release/termy-agent`，约 8.5 MB。单核实测 10m10s，峰值 530 MB。

安装脚本**拒绝以 root 运行**——Agent 要以将来使用它的那个普通用户身份安装。它做三件事：装二进制到 `~/.local/bin`、装 user unit 到 `~/.config/systemd/user`、`loginctl enable-linger`（唯一需要 sudo 的一步）。

### 4.2 Windows：只能在 Windows 本机构建

```powershell
rustup toolchain install 1.97.1
cargo build --manifest-path agent\Cargo.toml --release
# 产物 agent\target\release\termy-agent.exe
```

**不要试图从 Linux 交叉编译**，三条理由，每条都是硬阻塞：

1. `agent/Cargo.toml` 在 `[target.'cfg(windows)'.dependencies]` 里依赖 `windows-sys` 的 `Win32_System_JobObjects`——进程树终止靠 Job Object，这是 Windows 侧的核心功能；
2. `portable-pty` 在 Windows 走 ConPTY，链接 Windows 系统库；
3. `x86_64-pc-windows-msvc` 需要 MSVC 链接器，Linux 上没有。换 `-gnu` target 要装 mingw-w64，且 ConPTY 与 Job Object 在 mingw 下的可用性未经验证——不值得为省一次构建去趟这个坑。

开发机当前只装了 `x86_64-unknown-linux-gnu` 一个 target，也没有 mingw-w64。

Windows 侧的开机自启 MVP 没提供脚本，用任务计划程序或注册为服务，见 `operations.md` §2.2。

---

## 5. 插件

```bash
pnpm install
pnpm sync:protocol        # 同步协议类型，改过 schema 才需要
pnpm build                # tsc --noEmit + esbuild + verify-build → main.js
pnpm package:zip          # 打成 termy-<version>.zip
```

`pnpm build` 产出 `main.js`；分发包是 `main.js` + `manifest.json` + `styles.css` 三个文件。

**最容易搞混的一点**：`pnpm build:rust` 构建的是 **Termy 自己的本地终端服务端**（`rust-servers/`，产物 `termy-server-{platform}-{arch}`），**与本次新增的 `termy-agent` 毫无关系**。远程功能不需要跑它，本地终端功能才需要。`pnpm release` = `build` + `build:rust` + `package:zip`，走的是完整的本地功能链路。

> 插件端的 Obsidian 集成尚未实现，见 [plugin-handover.md](plugin-handover.md)。当前 `pnpm build` 构建出来的是不含远程功能的 Termy 本体。

---

## 6. 验证

```bash
cd protocol && npm test                                   # 4 项：schema + openapi + 帧向量 + 类型检查
cargo test --manifest-path protocol/generated/rust/Cargo.toml   # 14
cargo test --manifest-path relay/Cargo.toml               # 71
cargo test --manifest-path agent/Cargo.toml               # 33
pnpm test:remote                                          # 52，需要 Node 22
npm --prefix e2e ci && ./e2e-run.sh                       # 8 项，起真 relay + 真 agent
```

CI 跑的门槛还包括：

```bash
cargo fmt --manifest-path relay/Cargo.toml --check
cargo clippy --manifest-path relay/Cargo.toml --all-targets -- -D warnings   # agent 同理
```

**`e2e-run.sh` 用的是 debug 产物**（`target/debug/`），跑它之前需要先有一次不带 `--release` 的构建；缺二进制或缺驱动依赖时脚本会直接告诉你该跑哪条命令。路径全部相对脚本自身，在任意 cwd 下调用都可以。

驱动的 `ws` 依赖装在 `e2e/` 而不是仓库根——根上装不了：`@xterm/addon-canvas@0.7.0` 声明的 peer 是 `@xterm/xterm@^5`，而根依赖是 `^6`，npm 即使 `--no-save` 也要解析整棵树，必然 ERESOLVE 失败。

---

## 7. 实测数据（1 核 / 961 MB + 8 GB swap）

| 构建 | 耗时 | 内存峰值 | 产物大小 |
| --- | --- | --- | --- |
| relay debug | 4m58s | ~411 MB | — |
| relay release | 13m58s | 633 MB | 11.0 MB |
| agent debug | 2m54s | ~400 MB | — |
| agent release | 10m10s | 530 MB | 8.5 MB |

这些数字只用来判断量级和内存下限，别当性能基准。
