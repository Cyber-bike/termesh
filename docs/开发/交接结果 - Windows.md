# 交接结果：Windows 验证

> 执行日期：2026-07-31 · 分支 `v2.0` · Windows 主机

| 步骤 | 结果 (OK / 报错原文) | 备注 |
| --- | --- | --- |
| 1 Node 基线 | OK（2026-07-31，人工执行） | 按清单要求未重跑 |
| 2 Electron 加载 | OK（2026-07-31，人工执行，25 符号 + 连接码生成成功） | A0 结论=直接嵌入；按清单要求未重跑 |
| 3 cargo test（含失败用例名） | **未完成：测试挂起** | 共 53 个库测试，51 个显示 `ok`；两个 `session_table` 用例超过 60 秒后持续挂起，无最终汇总或退出码。`pty::tests::the_session_slot_admits_one_at_a_time` 通过。未修改代码，终止挂起的测试进程后继续后续验证 |
| 4 假控制端 × Windows agent | **失败：建连和开会话成功，shell 往返超时** | 单实例锁通过；`status` 未显示连接码；Ctrl-C 打印 `shutting down` 后进程仍未退出。详见下方原始输出 |
| 5 打包态加载 | OK（2026-07-31，人工执行） | `external + 插件目录携带 node_modules` 可行；`pnpm build` 和 bundle smoke check 通过；真实 Obsidian 重启后 Console 验证通过 |

## 步骤 3：Windows agent 测试套件

执行命令：

```powershell
cd agent ; cargo test
```

构建完成输出：

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 30m 49s
Running unittests src\lib.rs (target\debug\deps\termy_agent-6fc7194ab4183cc2.exe)

running 53 tests
```

显示 `ok` 的测试共 51 个，其中 Windows PTY 测试原文：

```text
test pty::tests::the_session_slot_admits_one_at_a_time ... ok
```

测试未正常结束。挂起用例及输出原文：

```text
test session_table::tests::a_request_past_max_concurrent_is_rejected_not_queued has been running for over 60 seconds
test session_table::tests::close_all_empties_the_table has been running for over 60 seconds
```

上述输出后持续无新输出，因而没有 `test result` 汇总和进程退出码。按清单约束未尝试修改 `agent/src`，终止挂起的测试进程后继续步骤 4。

## 步骤 4：Windows agent × JS 假控制端

首次按清单启动时，`cargo test` 未生成 CLI 二进制，原文如下：

```text
.\agent\target\debug\termy-agent.exe: The term '.\agent\target\debug\termy-agent.exe' is not recognized as a name of a cmdlet, function, script file, or executable program.
Check the spelling of the name, or if a path was included, verify that the path is correct and try again.
```

执行清单准备章节中的 `cd agent ; cargo build` 后构建成功，再次启动正常打印连接码并等待控制端。

### status

agent 运行期间执行 `termy-agent.exe status`，未显示连接码，输出原文：

```text
identity   <redacted-device-id>
agent      pid 1388 (not running)
code       none (start the agent with `termy-agent run`)
```

### 单实例锁

运行第二个 `run --loopback` 实例被拒，退出码为 1，输出原文：

```text
error: another termy-agent instance already holds C:\Users\<redacted-user>\AppData\Roaming\TermyAgent\agent.lock
```

上述用户名按仓库隐私规则脱敏，除此之外保留输出原文。

### 假控制端

假控制端成功连接并打开 PowerShell 会话，但 marker 未返回，退出码为 1。输出原文：

```text
(node:22592) [DEP0128] DeprecationWarning: Invalid 'main' field in '\\?\<repo>\scripts\a0-spike\node_modules\@number0\iroh\package.json' of 'iroh-js/index.js'. Please either fix that or report it to the module author
(Use `node --trace-deprecation ...` to show where the warning was created)
dialing <redacted-device-id> ...
connected
session opened: 0a0783b5-cda8-4edf-b758-0f7783abbbe2 shell: powershell.exe
FAKE CONTROLLER: TIMED OUT after 30s
```

上述仓库路径和设备 ID 按仓库隐私规则脱敏，除此之外保留输出原文。

agent 侧对应输出原文：

```text
2026-07-31T06:57:05.028698Z  INFO termy_agent::serve: controller connected remote=bdeb0efb56
2026-07-31T06:57:05.048595Z  INFO termy_agent::serve: terminal session opened session=0a0783b5-cda8-4edf-b758-0f7783abbbe2 shell=powershell.exe
```

### Ctrl-C

Ctrl-C 后输出原文：

```text
shutting down
2026-07-31T06:57:56.721495Z  INFO termy_agent::serve: controller connection closed: closed remote=bdeb0efb56
```

随后 PowerShell 提示符未返回；进程检查显示 `termy-agent` PID 1388 仍存活且 `Responding=True`。因此 Ctrl-C 干净退出验证失败，测试终端随后被终止。

## 步骤 5：打包态加载

1. 在 `esbuild.config.mjs` 的 `external` 数组加入 `@number0/iroh`。
2. `pnpm build` 通过，输出 `[verify-build] Bundle smoke check passed`。
3. 使用 `pnpm install:dev '<test-vault>' --no-rust` 安装到测试 vault，构建和安装均成功。
4. 将 `scripts/a0-spike/node_modules/@number0/` 整体复制到 `<test-vault>\.obsidian\plugins\termy\node_modules\@number0\`，确认包含 `iroh` 和 `iroh-win32-x64-msvc`。
5. 人工完全重启 Obsidian，在 Console 中从上述插件目录 `require` 模块并执行 Endpoint loopback 绑定、连接码生成和关闭；人工回填结果为 OK。

结论：插件分发布局采用 `external + 插件目录携带 node_modules` 可行，本次保留 `esbuild.config.mjs` 的 external 改动。