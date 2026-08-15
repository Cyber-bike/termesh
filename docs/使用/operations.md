# 部署与运维

覆盖 Agent 的安装与 CLI 参考。

v2.0 **不需要部署任何云端服务**——没有账号、没有云端 Relay 需要你运维；点对点连接的协调路径默认用 iroh 官方托管中继，不需要自建（自建列入 v2.1 待确认事项）。这与 V1 的"必须自己起一个 Relay"完全不同，如果你在找云端 Relay 部署步骤，那是 V1 的内容，已经不适用了。

---

## 1. Agent

### 1.1 Ubuntu

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

**没有配对步骤。** 安装脚本跑完直接打印连接码，把它贴进 Termy 插件的"添加设备"即可——不需要登录、不需要先在服务端生成配对码。装完之后随时可以用 `termy-agent status` 再看一次连接码，或 `journalctl --user -u termy-agent -f` 看日志。

### 1.2 Windows

`termy-agent.exe` 子命令与 Linux 完全一致，**除了不带子命令时的行为**：双击 `termy-agent.exe`（不带任何参数）等价于 `termy-agent.exe run`——会打开一个控制台窗口并打印连接码，窗口保持打开直到 Ctrl-C 或出错；如果启动就失败（比如身份锁已被占用），窗口也会停住等按 Enter 再关，不会一闪而过。命令行里显式敲 `termy-agent.exe run` 行为不变。配置在 `%APPDATA%\TermyAgent\config.json`。开机自启可以用任务计划程序或注册为服务，MVP 未提供安装脚本。

### 1.3 CLI 参考

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

### 1.4 配置与状态文件

| 平台 | 配置 | 状态 |
| --- | --- | --- |
| Ubuntu | `$XDG_CONFIG_HOME/termy-agent/config.json`（默认 `~/.config/termy-agent/`） | 同目录 `agent.state.json` |
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

### 1.5 常见问题

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

---

## 2. V1 遗留：云端 Relay

`relay/` 目录（Axum + SQLite 的账号/配对/WSS 网关服务）是 V1 的实现，代码仍在仓库里、仍可独立构建测试（`cargo build --manifest-path relay/Cargo.toml`），但 **v2.0 的 Agent 已经不会连接任何 Relay**——它的 relay 客户端代码已被删除。除非你在维护 V1 遗留的插件端代码路径（`relayClient.ts` 等），否则不需要部署它。是否/何时整体移除尚未排期。
