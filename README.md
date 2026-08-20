<div align="center">

# Termesh

<img src="assets/termy-logo-selection.png" width="150" alt="Termesh logo" />

*A device-aware terminal mesh for Obsidian*

Open local and remote terminal sessions from one device home, with reusable workflows, AI CLI context handoff, and a native Rust PTY backend.

[![Version](https://img.shields.io/badge/version-1.4.4-7c3aed?style=for-the-badge)](./manifest.json)
[![Obsidian](https://img.shields.io/badge/Obsidian-Desktop%20Only-8b5cf6?style=for-the-badge)](https://obsidian.md/)
[![Community Plugin](https://img.shields.io/badge/Obsidian-Community%20Plugin-22c55e?style=for-the-badge)](https://obsidian.md/plugins?id=termesh)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue?style=for-the-badge)](./LICENSE)
[![PTY](https://img.shields.io/badge/backend-Rust%20PTY-f97316?style=for-the-badge)](./rust-servers)

English / [简体中文](./README_ZH.md)

[Install](#installation) · [Quick Start](#quick-start) · [Features](#features) · [Screenshots](#visual-tour) · [Report Issues](https://github.com/jiang-zhong-xi/Termy/issues) · [Telegram](https://t.me/+t6oRqhaw8c1jNzE1)

<p align="center">
  <img src="assets/termy-workspace-overview.png" width="980" alt="Termy main workspace preview with Obsidian, Codex CLI, OpenCode, and Claude Code" />
</p>

</div>

---

## Why Termesh?

Termesh is built for people who work across notes, terminals, AI coding sessions, and more than one device. Its device home keeps local and paired remote terminals in one place while preserving the editor context that makes an Obsidian-native terminal useful.

- **Native PTY backend**: Rust keeps the backend lean and avoids extra bridge runtimes.
- **Real terminal UX**: xterm.js frontend with search, copy/paste, prompt navigation, split panes, and multi-session support.
- **Workflow-driven automation**: Run reusable terminal, Obsidian-command, and external-link workflows from the status bar or command palette.
- **File-aware interactions**: Drag text, files, and folders into the terminal and open file references directly from terminal output.
- **AI-aware context handoff**: Claude Code, Codex CLI, and OpenCode integrations can inherit active note context from Obsidian.
- **Desktop-first customization**: Shell selection, tab/split placement rules, theme sync, background images, blur, renderer controls, and Windows input handling.

## Features

### Terminal Workspace

- Run local shells directly inside Obsidian on Windows, macOS, and Linux.
- Use `cmd`, PowerShell, PowerShell Core, WSL, Git Bash, `bash`, `zsh`, or a custom shell path.
- Open terminals in the current tab, a new tab, left/right tab groups, horizontal/vertical splits, or a new window.
- Search terminal output, clear screen or buffer, resize fonts, navigate prompts, and copy/paste normally.
- Keep new terminals near existing terminal tabs, focus them automatically, or pin them on creation.

### Workflows & Launchers

- Create preset workflows with one or more ordered actions.
- Combine terminal commands, Obsidian commands, and external links in a single workflow.
- Launch workflows from the status bar menu, command palette, or built-in workflow commands.
- Decide whether each workflow appears in the status bar, opens a terminal, starts a fresh terminal instance, or renames the target tab.
- Start quickly with built-in launchers for Claude Code, Codex CLI, OpenCode, and Gemini CLI.

### Obsidian Interactions

- Send the current editor selection, full note, or active note path into the active terminal.
- Drag text, files, and folders into the terminal to paste content or resolved paths.
- Click file references printed by tools, agents, scripts, or compilers to reopen matching vault files or external paths.
- Open the bundled changelog from the command palette or settings.

### AI & Coding Integrations

> [!NOTE]
> Claude Code, OpenCode, or Codex sessions started from an external terminal are ordinary CLI processes outside Termesh's Obsidian integration layer, so they cannot automatically know the active note, vault/workspace root, or editor selection.

- Termesh starts AI CLIs inside the current vault context, where the active note, selection, open files, and workspace root can be available to coding tasks.
- Claude Code and OpenCode use Termesh's IDE bridge; Codex uses a vault-local Skill at `.agents/skills/termy-obsidian-context/SKILL.md`.
- The built-in Codex launcher starts `codex` directly, without MCP registration or global CLI configuration changes.

### Privacy and Network Access

- Termesh does not include telemetry or analytics.
- Termesh downloads the matching native PTY server from [GitHub Releases](https://github.com/jiang-zhong-xi/Termy/releases). On first use of remote devices, it downloads the fixed-version platform-specific iroh `.node` runtime from unpkg, with jsDelivr and GitHub Releases as fallbacks. Native downloads are verified against bundled SHA-256 hashes; offline mode disables them and all update checks.
- Terminal sessions run local shell commands and user-configured workflows. Those commands may read files, modify files, or access the network according to the shell command or external CLI being run.
- Termesh starts local WebSocket connections for its PTY backend and optional IDE bridge. These connections are used for local terminal transport and editor-context handoff.
- Context-aware AI launchers can pass the active note path, selection, editor context, and vault/workspace path to local CLI tools. The Codex integration writes a vault-local helper skill under `.agents/skills/termy-obsidian-context/`.
- Optional: when **Check for AI launcher updates** is enabled in settings, Termesh queries `https://registry.npmjs.org` for the latest Claude Code and Codex CLI releases, and `https://api.github.com` for the latest OpenCode release. The setting is **off by default** and offline mode disables it regardless of the toggle.

### Appearance & Ergonomics

- Follow the Obsidian theme or use custom foreground and background colors.
- Choose Canvas or WebGL rendering, with automatic Canvas fallback when background images are enabled.
- Configure background image URL/path, opacity, size, position, blur amount, and text opacity.
- Use localized UI in English, Simplified Chinese, Japanese, Korean, and Russian.
- Tune Windows-friendly input behavior with `win32-input-mode` support for shells that depend on native key events.

## Visual Tour

<details open>
<summary><strong>Workspace preview</strong></summary>
<br />

<p align="center">
  <img src="assets/termy-workspace-overview.png" width="980" alt="Termy full workspace preview" />
</p>

</details>

<details>
<summary><strong>Workflow UI</strong></summary>
<br />

<table>
  <tr>
    <td width="34%" align="center">
      <img src="assets/termy-statusbar-workflows.png" alt="Termy workflow launcher menu from the status bar" />
      <br />
      <sub>Status bar workflow launcher</sub>
    </td>
    <td width="66%" align="center">
      <img src="assets/termy-settings-workflows.png" alt="Termy workflow settings with built-in Claude Code, Codex CLI, OpenCode, and Gemini CLI entries" />
      <br />
      <sub>Workflow configuration, instance behavior, and built-in launchers</sub>
    </td>
  </tr>
</table>

<p align="center">
  <img src="assets/termy-workflow-editor.png" width="900" alt="Termy preset workflow editor with actions, notes, and context-awareness settings" />
  <br />
  <sub>Preset workflow editor with action ordering, notes, and context-awareness controls</sub>
</p>

</details>

<details>
<summary><strong>Theme customization</strong></summary>
<br />

<p align="center">
  <img src="assets/termy-settings-theme.png" width="900" alt="Termy theme settings with background image, blur, and text opacity controls" />
</p>

</details>

## Command Highlights

| Command | What it does |
| --- | --- |
| `Open terminal` | Opens a new terminal using your configured placement rules. |
| `Termesh: show changelog` | Opens the bundled changelog modal. |
| `Terminal: split horizontal / split vertical` | Splits the active terminal. |
| `Terminal: send selection` | Sends the current editor selection to the active terminal. |
| `Terminal: send current note` | Sends the full current note content. |
| `Terminal: send current path` | Sends the active note path. |
| `Terminal: previous prompt / next prompt` | Navigates prompt history. |
| `Terminal: last failed command` | Jumps to the most recent failed command when available. |

## Installation

### Requirements

- Obsidian Desktop
- Windows, macOS, or Linux

> [!WARNING]
> Termesh is desktop-only because it uses a native PTY backend.

### Install from the Obsidian Community Plugins (recommended)

Termesh is now listed in the official Obsidian Community Plugins directory.

1. Open **Settings → Community plugins** and turn off **Restricted mode** if it is enabled.
2. Click **Browse** and search for `Termesh`.
3. Click **Install**, then **Enable**.

### Install with BRAT (early updates)

Use BRAT if you want to track the latest tagged build before it ships to the community directory.

1. Install [BRAT](https://github.com/TfTHacker/obsidian42-brat).
2. Open BRAT settings and choose **Add beta plugin**.
3. Enter `jiang-zhong-xi/Termy`.
4. Install the plugin and enable it in **Settings → Community plugins**.

### Manual install

Community Plugins and BRAT installations automatically download the verified platform-native runtime when remote devices are first used. Manual installation is only needed for offline environments:

1. Download the matching `termesh-<version>-<platform>.zip` complete package from [GitHub Releases](https://github.com/jiang-zhong-xi/Termy/releases), for example `win32-x64` on Windows x64.
2. Extract the release files into `.obsidian/plugins/termesh/` inside your vault.
3. Reload Obsidian.
4. Enable Termesh in **Settings → Community plugins**.

### Install a remote Agent

The plugin is the controller. Each remote computer also needs `termesh-agent`, which runs as the signed-in ordinary user and exposes that user's shell. No account or separate pairing service is required.

Official Agent builds are currently available for **Linux x64** and **Windows x64**. A macOS Agent is not currently published; macOS can still run the Termesh plugin and local terminals.

#### Linux x64 (Ubuntu 22.04/24.04)

Run the installer as the ordinary user who will own the remote shell, not as root:

```bash
curl -fsSL https://raw.githubusercontent.com/jiang-zhong-xi/Termy/main/agent/packaging/install-linux.sh | bash
```

The installer downloads the latest Agent, verifies its SHA-256 checksum, installs it to `~/.local/bin`, and starts a systemd user service. It requests `sudo` once to enable lingering so the Agent keeps running after logout; the Agent and remote shells themselves never run as root.

When installation finishes, copy the printed connection code into **Termesh → Add device**. Show it again or inspect logs with:

```bash
~/.local/bin/termesh-agent status
journalctl --user -u termesh-agent -f
```

#### Windows x64

1. Download `termesh-agent-win32-x64.exe` and its `.sha256` file from the latest [GitHub Release](https://github.com/jiang-zhong-xi/Termy/releases/latest).
2. Verify the checksum with `Get-FileHash .\termesh-agent-win32-x64.exe -Algorithm SHA256`, then compare it with the first value in the `.sha256` file.
3. Rename it to `termesh-agent.exe`, place it in a permanent directory, and run `termesh-agent.exe run`. Double-clicking the executable does the same thing.
4. Copy the printed connection code into **Termesh → Add device**. Keep the Agent running; use Windows Task Scheduler if it should start automatically at sign-in.

See the [Agent deployment and operations guide](docs/%E4%BD%BF%E7%94%A8/operations.md) for configuration, startup, upgrades, removal, and troubleshooting on each platform.

## Quick Start

1. Open Termesh from the ribbon, command palette, or empty-tab action to choose a device.
2. Choose your shell and terminal placement behavior in settings.
3. Try the built-in workflows from the status bar menu.
4. Send your current selection, note, or file path into the terminal.
5. Drag a file or folder into the terminal to paste its resolved path.
6. Click file references printed by tools or agents to jump back into the matching file.

## Development

```bash
pnpm install
pnpm build
pnpm build:rust
pnpm package:zip
```

| Script | Purpose |
| --- | --- |
| `pnpm dev` | Frontend build/watch flow. |
| `pnpm build` | TypeScript check, production bundle, and bundle smoke check. |
| `pnpm build:rust` | Build native PTY backend binaries. |
| `pnpm package:zip` | Create a release zip. |
| `pnpm install:dev <vault-path>` | Build everything and install into a local dev vault. Pass `--no-rust` to skip the native PTY rebuild when only TypeScript changed. |
| `pnpm test:terminal` | Compile and run terminal-layer Node tests. |

See [docs/development.md](docs/development.md) for the full contributor guide: environment setup, building the Rust agent on each platform, testing, coding style, and the Obsidian developer policy limits every change has to respect.

## Architecture

```mermaid
graph LR
  A[Obsidian Plugin UI] --> B[xterm.js Terminal]
  B --> C[Native Rust PTY Server]
  A --> D[Workflow Launcher]
  A --> E[Context Services]
  E --> F[Claude/OpenCode IDE Bridge]
  E --> G[Codex Skill Context]
  D --> H[AI CLI Launchers]
```

- **Frontend**: TypeScript, Obsidian plugin APIs, and xterm.js.
- **Backend**: Native Rust PTY server built on `portable-pty`.
- **AI context**: Claude Code and OpenCode integrate through the IDE bridge; Codex integrates through a vault-local Skill.
- **Packaging**: Generated plugin assets are emitted as `main.js` and `styles.css`; native binaries are copied to `binaries/`.

## License

Termesh is licensed under [GPL-3.0](./LICENSE).

## Credits

- [xterm.js](https://xtermjs.org/)
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty)

---

<div align="center">

**Made with ❤️ for Obsidian power users**

If Termesh helps your workflow, consider starring the project.

</div>
