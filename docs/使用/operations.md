# 部署与运维

覆盖 Agent 的安装与 CLI 参考。

v2.0 **不需要部署任何云端服务**——没有账号、没有云端 Relay 需要你运维；点对点连接的协调路径默认用 iroh 官方托管中继，不需要自建（自建列入 v2.1 待确认事项）。这与 V1 的"必须自己起一个 Relay"完全不同，如果你在找云端 Relay 部署步骤，那是 V1 的内容，已经不适用了。

---

## 1. Agent

Agent 安装在要被远程控制的目标电脑上，Termesh 插件安装在运行 Obsidian 的控制端。Agent 以启动它的普通用户身份创建 shell，不会提权。当前 Release 正式提供以下产物：

| 目标平台 | Release 产物 | 安装方式 | 支持状态 |
| --- | --- | --- | --- |
| Ubuntu 22.04/24.04 x64 | `termy-agent-linux-x64` | 一键脚本 + systemd user service | 正式支持 |
| Windows 10/11 x64 | `termy-agent-win32-x64.exe` | 手动下载；可选任务计划程序 | 正式支持 |
| macOS | 无 | 无 | 暂未发布 Agent；可作为插件控制端使用 |
| Linux ARM64 / 其他发行版 | 无 | 可自行从源码构建 | 未提供预编译产物，不属于当前正式验证矩阵 |

### 1.1 Linux x64（Ubuntu 22.04/24.04）

#### 安装

一条命令装好并启动，直接打印连接码，不需要 GitHub 仓库checkout：

```bash
curl -fsSL https://raw.githubusercontent.com/jiang-zhong-xi/Termy/main/agent/packaging/install-linux.sh | bash
```

脚本自动完成：没传本地二进制路径时从最新 Release 下载 `termy-agent-linux-x64` 并校验 sha256、把二进制装到 `~/.local/bin`、把 user unit 装到 `~/.config/systemd/user`、启用 lingering、`systemctl --user enable --now`、等待 Agent 上线后直接打印连接码。全程只有一步需要交互（见下）。

从本地构建安装（跳过下载）：

```bash
./agent/packaging/install-linux.sh /path/to/termy-agent
```

**关于权限**：`loginctl enable-linger` 在多数发行版需要一次 root 或 polkit 认证，脚本用 `sudo` 处理这一步。口径是"**安装时一次性 sudo，运行时 Agent 与远程 shell 全程非 root**"。不启用 lingering 的话，SSH 一断 Agent 就被杀。

**没有配对步骤。** 安装脚本跑完直接打印连接码，把它贴进 Termy 插件的"添加设备"即可——不需要登录、不需要先在服务端生成配对码。装完之后随时可以用 `~/.local/bin/termy-agent status` 再看一次连接码，或 `journalctl --user -u termy-agent -f` 看日志。重新登录后如果 `~/.local/bin` 已进入 `PATH`，后续命令可以简写为 `termy-agent`。

安装前需要 `curl`、`sha256sum`、`systemd`、`loginctl` 和 `sudo`。脚本只下载官方 Release 中的 `linux-x64` 产物；其他架构会停止并提示传入本地编译的二进制。

#### 日常管理

```bash
# 查看进程状态、身份指纹和当前连接码
~/.local/bin/termy-agent status

# 查看实时日志
journalctl --user -u termy-agent -f

# 停止、启动或重启
systemctl --user stop termy-agent
systemctl --user start termy-agent
systemctl --user restart termy-agent

# 查看 systemd 状态
systemctl --user status termy-agent
```

修改设备名或 shell 后应重启服务，新配置只影响之后创建的远程会话：

```bash
~/.local/bin/termy-agent config set-name "My Linux server"
~/.local/bin/termy-agent config set-shell /bin/bash -- -l
systemctl --user restart termy-agent
```

#### 升级

重新执行安装命令会下载、校验并覆盖为最新 Release。覆盖二进制后要重启已经运行的服务，才能切换到新版本：

```bash
curl -fsSL https://raw.githubusercontent.com/jiang-zhong-xi/Termy/main/agent/packaging/install-linux.sh | bash
systemctl --user restart termy-agent
~/.local/bin/termy-agent status
```

Agent 重启后连接码可能变化，请在插件中使用 `status` 输出的当前连接码。

#### 卸载

```bash
systemctl --user disable --now termy-agent
rm -f ~/.config/systemd/user/termy-agent.service
systemctl --user daemon-reload
rm -f ~/.local/bin/termy-agent
```

以上命令保留 `~/.config/termy-agent/` 中的配置和设备身份，以便重新安装后仍使用同一身份。确认不再需要时再删除：

```bash
rm -rf ~/.config/termy-agent
```

安装器启用的 lingering 可能也被其他用户服务使用，因此卸载时不会自动关闭。确认该用户不需要任何退出登录后继续运行的 user service 时，可执行 `sudo loginctl disable-linger "$(id -un)"`。

### 1.2 Windows x64

#### 下载与校验

当前没有 Windows 自动安装器。请从 [最新 GitHub Release](https://github.com/jiang-zhong-xi/Termy/releases/latest) 下载以下两个文件：

- `termy-agent-win32-x64.exe`
- `termy-agent-win32-x64.exe.sha256`

在下载目录打开 PowerShell 并校验文件；两个值必须完全一致：

```powershell
$expected = ((Get-Content .\termy-agent-win32-x64.exe.sha256 -Raw) -split '\s+')[0]
$actual = (Get-FileHash .\termy-agent-win32-x64.exe -Algorithm SHA256).Hash.ToLowerInvariant()
$expected
$actual
if ($actual -ne $expected.ToLowerInvariant()) { throw 'SHA-256 mismatch' }
```

校验成功后，把程序放到不会被清理的固定目录：

```powershell
$installDir = Join-Path $env:LOCALAPPDATA 'TermyAgent'
New-Item -ItemType Directory -Force $installDir | Out-Null
Move-Item .\termy-agent-win32-x64.exe (Join-Path $installDir 'termy-agent.exe')
```

#### 启动和添加设备

```powershell
& "$env:LOCALAPPDATA\TermyAgent\termy-agent.exe" run
```

终端会打印连接码并保持运行。把连接码粘贴到 Obsidian 的 **Termesh → 添加设备**。关闭窗口或按 Ctrl-C 会停止 Agent；需要远程连接时该进程必须保持运行。

`termy-agent.exe` 子命令与 Linux 完全一致，**除了不带子命令时的行为**：双击 `termy-agent.exe`（不带任何参数）等价于 `termy-agent.exe run`——会打开一个控制台窗口并打印连接码，窗口保持打开直到 Ctrl-C 或出错；如果启动就失败（比如身份锁已被占用），窗口也会停住等按 Enter 再关，不会一闪而过。命令行里显式敲 `termy-agent.exe run` 行为不变。配置在 `%APPDATA%\TermyAgent\config.json`。开机自启可以用任务计划程序或注册为服务，MVP 未提供安装脚本。

在另一个 PowerShell 窗口中可以查看状态和当前连接码：

```powershell
& "$env:LOCALAPPDATA\TermyAgent\termy-agent.exe" status
```

#### 登录后自动启动

当前版本未提供 Windows service 安装器。可按以下步骤使用任务计划程序：

1. 按 `Win+R`，输入 `taskschd.msc`。
2. 选择“创建任务”，名称填写 `Termy Agent`。
3. 在“触发器”中新增“登录时”。
4. 在“操作”中新增“启动程序”，程序填写 `%LOCALAPPDATA%\TermyAgent\termy-agent.exe`，参数填写 `run`。
5. 在“设置”中启用任务失败后重新启动，并将“如果任务已在运行”设为“不启动新实例”。
6. 保存后右键该任务并选择“运行”，再用 `status` 检查。

Agent 仍以登录用户运行。若目标机器在用户登录前就必须接受连接，需要由管理员按本机安全策略把它注册为该普通用户的服务；当前项目不提供或维护该部署方式。

#### 配置、升级与卸载

```powershell
# 修改显示名称或 shell；重启 Agent 后生效
& "$env:LOCALAPPDATA\TermyAgent\termy-agent.exe" config set-name "My Windows PC"
& "$env:LOCALAPPDATA\TermyAgent\termy-agent.exe" config set-shell powershell.exe

# 停止手动运行的 Agent
# 在运行 Agent 的窗口按 Ctrl-C
```

升级时先停止 Agent 或任务计划任务，下载并校验新的 `termy-agent-win32-x64.exe`，再覆盖 `%LOCALAPPDATA%\TermyAgent\termy-agent.exe` 并重新启动。不要在进程运行时覆盖 Windows 可执行文件。

卸载时先停止 Agent，删除任务计划程序中的 `Termy Agent`，再删除 `%LOCALAPPDATA%\TermyAgent\termy-agent.exe`。默认保留 `%APPDATA%\TermyAgent` 中的配置和身份；确认不再需要该身份时，可以手动删除整个配置目录。

### 1.3 macOS 与其他平台

当前 Release **没有发布 macOS、Linux ARM64 或 Windows ARM64 Agent**。macOS、Linux ARM64 仍可运行支持该平台的 Termesh 插件控制端和本地终端，但不能按本文方式把该设备作为正式支持的远程 Agent。

Linux ARM64 或其他 Unix 平台的开发者可以尝试从源码构建：

```bash
cargo build --manifest-path agent/Cargo.toml --release
./agent/packaging/install-linux.sh agent/target/release/termy-agent
```

这条路径需要 Rust 工具链，也不代表该平台已进入 Release 验证矩阵。Windows Agent 必须在 Windows 本机构建，详见 [构建指南](../开发/building.md)。

### 1.4 CLI 参考

| 命令 | 说明 |
| --- | --- |
| `run [--loopback]` | 打印身份指纹与连接码，接受控制端连接。持单实例锁（同一身份不能跑两个实例）。`--loopback` 只绑 `127.0.0.1`、禁用中继与地址发布，仅用于本机开发/测试，连接码里不会有能从外部访问的地址 |
| `status` | 读状态文件打印身份指纹、pid 是否存活、连接码；**不与运行中的进程通信**，纯读文件 |
| `config show` | 打印当前配置（设备名、身份文件路径、接收目录、最大并发会话数、shell）。没有配置文件时打印全部默认值 |
| `config set-name <名字>` | 改设备显示名 |
| `config set-receive-root <绝对路径>` | 改接收目录（v2.0 文件传输尚未实现，这个字段目前不生效，为 Phase C 预留） |
| `config set-shell <程序> [参数...]` | 改远程 shell。**以 `-` 开头的参数要用 `--` 隔开**，否则会被当成 CLI 自己的选项：`config set-shell /bin/bash -- -l` |
| `rotate-identity [--yes]` | 重新生成设备身份（Ed25519 keypair）。**旧连接码立刻失效**，所有控制端都要用新连接码重新配对。默认交互确认，`--yes` 跳过 |

**没有 `bind` 命令。** v2.0 免账号、免配对服务，Agent 的身份就是本地生成的密钥对，不需要向任何服务端注册。

### 1.5 配置与状态文件

| 平台 | 配置 | 状态 |
| --- | --- | --- |
| Linux | `$XDG_CONFIG_HOME/termy-agent/config.json`（默认 `~/.config/termy-agent/`） | 同目录 `agent.state.json` |
| Windows | `%APPDATA%\TermyAgent\config.json` | 同目录 `agent.state.json` |

同目录下还有身份文件 `identity.json`（默认路径，可用 `identityKeyPath` 覆盖）。Unix 上目录 `0700`、文件 `0600`。

**配置文件默认为空即代表全部用默认值**——v2.0 "免配置"的意思是这个文件甚至可以不存在，`run` 照样能跑。状态文件不含任何机密（不含身份私钥、不含配对凭据），连接码本身也只在运行期间有效（内嵌当前已知的网络地址），可以随手贴给别人看，但记住"谁拿到连接码谁就能连上"，见 [privacy-and-limits.md](privacy-and-limits.md)。

默认值：

| 字段 | 默认值 |
| --- | --- |
| `deviceName` | 系统主机名 |
| `receiveRoot` | `~/TermyReceive` |
| `maxConcurrentSessions` | 8（合法范围 1–256） |
| `shell`（Unix） | `/bin/bash -l`（登录 shell，见下方"常见问题") |
| `shell`（Windows） | `powershell.exe` |

### 1.6 使用流程与常见问题

完整使用流程如下：

1. 在目标电脑启动 Agent，并保持进程运行。
2. 执行 `termy-agent status`（Windows 使用完整 exe 路径）取得当前连接码。
3. 在控制端打开 Obsidian → Termesh → **添加设备**，粘贴连接码。
4. 从设备首页选择该设备并创建远程终端。
5. 不再需要时可在插件中移除设备；这只删除控制端记录，不会停止目标电脑上的 Agent。

连接码包含 Agent 身份与当前网络地址。拿到连接码的人可以尝试连接该 Agent，因此应按访问凭据对待，不要公开发布。需要让所有旧连接码失效时执行 `rotate-identity`，然后重启 Agent 并在所有控制端重新添加设备。

**`status` 显示 "not running"，但进程明明在跑**——检查 `XDG_RUNTIME_DIR`/`XDG_CONFIG_HOME` 或 `%APPDATA%` 是否和启动 Agent 时用的是同一份环境；`status` 是纯读文件，环境不一致就是在读两个不同的状态文件。

**本地开发连不上/想完全隔离**——用 `termy-agent run --loopback`，只绑 `127.0.0.1`，禁用中继和公共发现网络，连接码只能从本机连接。

**换了身份码之后，之前配对的设备连不上了**——`rotate-identity` 会让所有旧连接码失效，这是设计如此（不是 bug）。重新 `run` 拿新连接码，在插件里重新添加设备。

**远程终端里某个命令 command not found，但 SSH 进去却能用**——远程 shell 不是登录 shell（如果被 `config set-shell` 改过）。默认值已经是登录 shell（`/bin/bash -l`），如果你手动改成非 `-l` 的配置，装在 `~/.profile` 里加进 PATH 的东西（pipx、cargo、npm 全局、Claude Code）就会找不到。改回来：

```bash
termy-agent config set-shell /bin/bash -- -l   # 注意 -- ，否则 -l 会被当成 CLI 自己的选项
systemctl --user restart termy-agent
```

改的是**新会话**的 shell，已经开着的远程终端要关掉重开。

**目标机一直连不上（`DEVICE_UNREACHABLE`）**——检查 Agent 是否在跑（`status`）、连接码是不是最新的（重新生成过身份或者 Agent 重启过，连接码会变）。`journalctl --user -u termy-agent -f`（Ubuntu）能看到 iroh 连接建立失败的日志。非同一局域网场景依赖直连打洞或降级到公共中继，见 [privacy-and-limits.md](privacy-and-limits.md)。

**第二个控制端连不上，报 `CONTROLLER_ALREADY_CONNECTED`**——v2.0 同一时刻只允许一个控制端占用一台 Agent（doc §7.7），这是设计如此。断开第一个连接后再试。

**退出 SSH 后 Agent 就停了**——lingering 没启用。见 §1.1。

**`systemctl --user` 报 `Failed to connect to bus: No medium found`**——当前 SSH 会话没有继承 systemd user bus 环境。最新版安装脚本会自动修复；已经安装到一半时可以执行：

```bash
uid="$(id -u)"
sudo systemctl start "user@${uid}.service"
export XDG_RUNTIME_DIR="/run/user/${uid}"
export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
systemctl --user daemon-reload
systemctl --user enable --now termy-agent
```

这些 `export` 只影响当前 shell。新的 SSH 会话如果仍缺少环境变量，可以再次设置；安装脚本启动服务后，Agent 本身不依赖当前 SSH 会话保持连接。

---

## 2. V1 遗留：云端 Relay

`relay/` 目录（Axum + SQLite 的账号/配对/WSS 网关服务）是 V1 的实现，代码仍在仓库里、仍可独立构建测试（`cargo build --manifest-path relay/Cargo.toml`），但 **v2.0 的 Agent 已经不会连接任何 Relay**——它的 relay 客户端代码已被删除。除非你在维护 V1 遗留的插件端代码路径（`relayClient.ts` 等），否则不需要部署它。是否/何时整体移除尚未排期。
