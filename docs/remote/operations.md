# 部署与运维

覆盖云端 Relay 的部署、Agent 的安装，以及两者的 CLI 参考。

---

## 1. 云端 Relay

### 1.1 前提

- 一个**公网可信证书**和对应域名。Agent 用 `webpki-roots` 校验证书链，**自签证书连不上**。
- 一个反向代理终止 TLS。Relay 自己只说明文 HTTP，按方案 §6.1 它必须与代理处于同一受控主机网络。
- 一个**持久卷**。SQLite 数据文件不落到卷上，容器重启就会丢掉全部账号、配对码和设备。

### 1.2 必需的环境变量

| 变量 | 说明 |
| --- | --- |
| `TERMY_PEPPER` | 配对码与设备令牌摘要的服务器 pepper，≥32 字节，base64。**轮换会使全部配对码和设备令牌失效** |
| `TERMY_JWT_SECRET` | 访问令牌签名密钥，≥32 字节，base64。轮换会让所有人重新登录 |
| `TERMY_RELAY_URL` | 告诉 Agent 回连哪里，必须是公网名，如 `wss://relay.example.com/v1/agent/ws` |
| `TERMY_DB_PATH` | 默认 `/var/lib/termy-relay/relay.db` |
| `TERMY_BIND` | 默认 `0.0.0.0:8080` |

生成密钥：

```bash
head -c32 /dev/urandom | base64
```

三个必需变量缺任何一个，进程都拒绝启动——这是有意的，没有安全的默认值。

### 1.3 启动

```bash
export TERMY_PEPPER=... TERMY_JWT_SECRET=... TERMY_RELAY_URL=wss://relay.example.com/v1/agent/ws
docker compose -f relay/compose.yaml up -d
```

> `relay/Dockerfile` 与 `relay/compose.yaml` 在开发机上**没有实际构建过**（环境里没有 Docker 守护进程），首次使用前请自行构建验证一次。

**小机器上别用 Docker。** 镜像内会重跑一遍 `cargo build --release`（单核约 14 分钟、峰值 633 MB，外加 `rust:bookworm` 基础镜像约 1.5 GB 磁盘），而产物与 `relay/target/release/termy-relay` 完全一致。Docker daemon 本身还要常驻 100–200 MB，relay 进程实测只占 9 MB。compose 提供的持久卷和重启策略 systemd 都有等价物：

```ini
# /etc/systemd/system/termy-relay.service 关键行
User=termy
EnvironmentFile=/etc/termy-relay/relay.env   # 0600 root，systemd 以 root 读取后再降权
ExecStart=/usr/local/bin/termy-relay serve
Restart=always
RestartSec=5
ProtectSystem=strict
ReadWritePaths=/var/lib/termy-relay          # 唯一可写路径，即数据库目录
```

CLI 子命令也要读同一份环境：

```bash
set -a; . /etc/termy-relay/relay.env; set +a
sudo -u termy -E termy-relay useradd alice
```

### 1.4 反向代理

最低要求：

- 透传 WebSocket upgrade（`Upgrade` / `Connection` 头）
- 关闭响应缓冲
- `proxy_read_timeout` ≥ 60 s（大于 Agent 的 50 s 离线判定窗口）

nginx 片段。`$connection_upgrade` **必须先在 `http` 块里定义 map**，少了这段 nginx 直接起不来（unknown variable）——放 `/etc/nginx/conf.d/upgrade.conf` 即可：

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}
```

然后在 server 块里：

```nginx
location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_buffering off;
    proxy_read_timeout 300s;
}
```

`X-Forwarded-For` 必须设置——限流按它取源地址。**注意**：Relay 之所以信任这个头，前提是它的端口只有代理能访问。如果哪天把 Relay 端口直接暴露到公网，这个信任必须撤销，否则攻击者可以伪造无限个源地址绕过限流。

### 1.5 开户

MVP **没有注册接口**，账号只能在服务端创建：

```bash
docker compose exec relay termy-relay useradd alice
# 交互式会两次提示输入；非交互式可以管道传入：
echo 'a-strong-password' | docker compose exec -T relay termy-relay useradd alice
```

口令走 stdin，不进 argv。

### 1.6 排障

```bash
termy-relay device-list alice          # 某账号绑了哪些设备
termy-relay passwd alice               # 改口令
termy-relay healthcheck                # 探活，不需要任何环境变量
```

日志是结构化的，且**不会记录配对码、用户令牌或设备令牌**（§13.2）。`RUST_LOG=termy_relay=debug` 可以看到被拒绝的协议消息的原因。

---

## 2. Agent

### 2.1 Ubuntu

```bash
./agent/packaging/install-linux.sh /path/to/termy-agent
```

脚本做三件事：把二进制装到 `~/.local/bin`、把 user unit 装到 `~/.config/systemd/user`、启用 lingering。

**关于权限**：`loginctl enable-linger` 在多数发行版需要一次 root 或 polkit 认证，脚本用 `sudo` 处理这一步。口径是"**安装时一次性 sudo，运行时 Agent 与远程 shell 全程非 root**"。不启用 lingering 的话，SSH 一断 Agent 就被杀，方案 §16.2 的验收过不了。

绑定并启动：

```bash
termy-agent bind --code <配对码> --relay https://relay.example.com
systemctl --user enable --now termy-agent.service
termy-agent status
journalctl --user -u termy-agent -f
```

### 2.2 Windows

`termy-agent.exe` 子命令与 Linux 完全一致。配置在 `%APPDATA%\TermyAgent\config.json`。开机自启可以用任务计划程序或注册为服务，MVP 未提供安装脚本。

### 2.3 CLI 参考

| 命令 | 说明 |
| --- | --- |
| `bind --code <码> --relay <https 地址> [--name <名字>]` | 消费配对码换取设备令牌并写配置。`--name` 默认取主机名 |
| `config show` | 打印配置。**不打印设备令牌** |
| `config set-name <名字>` | 改设备显示名 |
| `config set-receive-root <绝对路径>` | 改接收目录 |
| `config set-shell <程序> [参数...]` | 改远程 shell |
| `run` | 连接 Relay 并提供服务。持单实例锁 |
| `status` | 读状态文件打印连接情况，不与运行中的进程通信 |

### 2.4 配置与状态文件

| 平台 | 配置 | 状态 |
| --- | --- | --- |
| Ubuntu | `$XDG_CONFIG_HOME/termy-agent/config.json`（默认 `~/.config/termy-agent/`） | 同目录 `agent.state.json` |
| Windows | `%APPDATA%\TermyAgent\config.json` | 同目录 `agent.state.json` |

Unix 上目录 `0700`、文件 `0600`。配置文件含设备令牌；**状态文件不含任何机密**，可以随手贴给别人看。

### 2.5 常见问题

**`status` 显示 needs rebind**——Relay 拒绝了设备令牌（设备被解绑，或数据库被重建）。重新生成配对码并 `bind`。

**Agent 反复重连**——检查 `journalctl` 里的关闭码。`4409` 表示同一设备有另一个连接（是不是跑了两个实例？单实例锁只在同一台机器上生效），`4401` 表示令牌无效。

**本地开发连不上**——`bind` 只收 `https://`、配置只收 `wss://`。本地 relay 没有 TLS 时设 `TERMY_AGENT_ALLOW_INSECURE=1` 放行明文，会打印警告。**不要在生产用**。

**传输失败**——检查接收目录是否可写（`termy-agent config show` 看 `receiveRoot`）。失败可能留下部分文件，见隐私与限制文档。
