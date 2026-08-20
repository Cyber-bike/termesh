# Development Guide

This is the single entry point for anyone setting up a local environment to work on Termesh: what the pieces are, how to build and test each of them, and the rules a contribution has to follow. (中文版见 [development_ZH.md](development_ZH.md)。)

## 1. Overview

Termesh is an Obsidian plugin that embeds a real terminal in a note-taking app, plus an optional remote-terminal feature: a Windows/desktop Obsidian control device drives a `termesh-agent` process on a target device (Windows or headless Ubuntu) over a direct `iroh` (QUIC) connection, no account or self-hosted server required.

| Path | Content |
| --- | --- |
| `src/` | The TypeScript Obsidian plugin. `src/services/` for runtime/integration logic (`terminal/`, `server/`, `codexCli/`, `context/`, `remote/`), `src/ui/` for views and modals, `src/settings/` for settings models/renderers, `src/i18n/` for locales, `src/utils/` for shared helpers. |
| `agent/` | The Rust remote-terminal agent (`termesh-agent`): device identity, `iroh` endpoint, connection-code pairing, multi-session PTY service. |
| `rust-servers/` | The native local PTY backend the plugin talks to over a local WebSocket. |
| `relay/`, `protocol/` | V1 (account + cloud relay) legacy implementation — see [§7](#7-legacy-v1-code). |
| `docs/` | This guide, plus screenshots/assets referenced from it. |
| `scripts/` | Build, packaging, and release tooling (Node scripts). |
| `e2e/` | End-to-end driver for a real loopback agent + real `@number0/iroh` binding. |

Generated build artifacts (`main.js`, `styles.css` at the repo root, `binaries/`) are produced by the build, not checked in as source.

## 2. Prerequisites

- **Rust** — version is pinned by `rust-toolchain.toml`; `rustup` picks it up automatically, don't install a different version by hand.
- **Node.js 22** — the plugin's test suites need `--experimental-strip-types`. See the Node 18 fallback note in [§4](#4-testing) if that's all you have.
- **pnpm** — version pinned in `package.json`'s `packageManager` field.
- **Windows contributors building the agent**: you must build directly on Windows. See [§3.3](#33-rust-agent-termesh-agent).

## 3. Building

### 3.1 First-time setup

```bash
pnpm install
pnpm build                # tsc --noEmit + esbuild + bundle smoke check -> main.js
```

For iterative development against a real vault:

```bash
pnpm install:dev <vault-path>   # builds both layers and installs into the vault
pnpm dev                        # esbuild watch mode
```

Pass `--no-rust` to `install:dev` to skip rebuilding the native PTY server when only TypeScript changed.

### 3.2 Plugin packaging

```bash
pnpm package        # assembles a distributable plugin-package/ directory
pnpm package:zip     # zips it as termesh-<version>.zip
```

`pnpm package` produces `main.js` + `manifest.json` + `styles.css` + `node_modules/@number0/` (the native remote-terminal dependency, see below). After packaging, verify nothing is a symlink:

```bash
find plugin-package/node_modules -type l   # should print nothing
```

**The `@number0/iroh` native module**: this N-API module backs the remote-terminal feature. `esbuild.config.mjs` marks it `external` because an installed Obsidian plugin directory has no `node_modules` of its own. Two distribution paths exist: Community Plugins/BRAT installs download the matching platform `.node` file on first use of a remote device (from unpkg, jsDelivr, or GitHub Releases, verified against a bundled SHA-256); offline packages bundle it directly. `scripts/package-plugin.js` step 5b resolves the platform package actually installed (via `require.resolve()`, not a hardcoded platform map — pnpm's isolated store symlinks these under `node_modules/.pnpm/`, and the platform matrix itself changes over time) and copies it, dereferenced, into `plugin-package/node_modules/@number0/`. Both distribution paths require building on the target OS/architecture — `pnpm install` only fetches the native package for the current platform.

### 3.3 Rust agent (`termesh-agent`)

**Linux:**

```bash
cargo build --manifest-path agent/Cargo.toml --release
./agent/packaging/install-linux.sh agent/target/release/termesh-agent
```

The install script **refuses to run as root** — install it as the normal user that will run the agent. It installs the binary to `~/.local/bin`, installs a systemd user unit under `~/.config/systemd/user`, and runs `loginctl enable-linger` (the one step that typically needs a root/polkit prompt). No pairing step is needed afterward — start the service, copy the printed connection code, paste it into the plugin.

**Windows: build on Windows only.** Cross-compiling from Linux is a hard blocker, not a convenience issue:

1. `agent/Cargo.toml`'s `[target.'cfg(windows)'.dependencies]` pulls in `windows-sys` for Job-Object-based process-tree termination and `OpenProcess`/`GetExitCodeProcess`-based liveness checks — both Windows-only implementations.
2. `portable-pty` uses ConPTY on Windows, which links Windows system libraries.
3. `x86_64-pc-windows-msvc` needs the MSVC linker, which doesn't exist on Linux.

```powershell
rustup toolchain install <version pinned in rust-toolchain.toml>
cargo build --manifest-path agent\Cargo.toml --release
# artifact: agent\target\release\termesh-agent.exe
```

There's no autostart install script for Windows yet — register it with Task Scheduler or as a service; `termesh-agent.exe run` is the command to keep running.

## 4. Testing

```bash
cargo test --manifest-path agent/Cargo.toml     # Rust unit + real-loopback QUIC integration tests
pnpm test:remote                                # plugin remote module, needs Node 22
pnpm test:terminal                              # local terminal-layer regression
pnpm lint                                       # general ESLint config, optional/complementary
```

**`pnpm lint:obsidian` is mandatory**, not optional, on every change that touches `src/**/*.ts` — run it and fix violations until it's clean before you consider the change done. It uses `eslint.obsidian.config.js`, which enforces rules the general config doesn't: `@microsoft/sdl/no-inner-html`, `@typescript-eslint/no-base-to-string`, `@typescript-eslint/no-redundant-type-constituents`, `@typescript-eslint/no-unnecessary-type-assertion`, `@typescript-eslint/require-await`, `obsidianmd/ui/sentence-case-locale-module`. It ignores `src/**/*.test.ts`, `scripts/`, `rust-servers/`, `binaries/`, `plugin-package/`, `main.js`, `styles.css` — if a change is entirely inside those paths you can skip it, otherwise run it.

Rust CI gates:

```bash
cargo fmt --manifest-path agent/Cargo.toml --check
cargo clippy --manifest-path agent/Cargo.toml --all-targets -- -D warnings
```

End-to-end (real loopback agent, real `@number0/iroh` binding, real shell echo + resize; does **not** cover file transfer):

```bash
pnpm install
cargo build --manifest-path agent/Cargo.toml
./e2e-run.sh
```

**Only have Node 18?** `pnpm test:remote`/`pnpm test:terminal` won't run directly since they need `--experimental-strip-types`. Transpile with the repo's `tsc` first and run the output under Node 18 — two pre-existing V1 test files (`relayClient.test.ts`, `remoteService.test.ts`) will fail there for unrelated `ws`-resolution reasons specific to that path; that's expected and unrelated to v2.0 code.

## 5. Coding style

- 2-space indentation in TypeScript, 4-space in Rust.
- TypeScript is strict-oriented: keep single quotes, semicolons, and explicit types where they improve clarity.
- `PascalCase` for classes and UI types, `camelCase` for functions and variables, descriptive lower-camel-case filenames (`terminalPathUtils.ts`, `settingsTab.ts`).
- Code comments are always in English, regardless of which locale file you're editing.

## 6. Obsidian developer policy — non-negotiable

These come straight from [Obsidian's developer policy](https://docs.obsidian.md/Developer+policies#Not+allowed). Violating any of them gets a plugin rejected from the community list and, if already shipped, removed. When in doubt, ask: "could a reasonable person describe this as one of the items below?" If yes, don't ship it.

- **No obfuscation.** `main.js` is minified by esbuild for size, which is fine because the readable TypeScript source on GitHub is the upstream truth — don't add encoded strings, runtime-decoded bodies, opaque packers, or eval-based loaders on top of that.
- **No ads**, dynamic or static, anywhere outside Termesh's own UI surfaces (settings tab, its own modals/views), and only when genuinely related to Termesh itself.
- **No client-side telemetry.** No analytics SDKs, usage pings, error-reporting endpoints, or "phone home" behavior. The only outbound network calls allowed: downloading the matching `termy-server` binary from GitHub Releases; downloading the fixed-version platform `.node` iroh runtime (unpkg/jsDelivr/GitHub Releases, SHA-256 verified, disabled by offline mode); the local-only WebSocket connections to the PTY backend and the Claude Code IDE bridge; and the optional, off-by-default AI launcher update check (also suppressed by offline mode). That setting must stay off by default.
- **No plugin self-update mechanism.** Obsidian's own updater is the only acceptable path for `main.js`/`styles.css`/`manifest.json`. The one allowed exception is downloading the matching native `termy-server` binary into `<plugin>/binaries/` — a separately versioned native asset, SHA-256 verified, disabled by offline mode, and it must never overwrite plugin JS/CSS/JSON.
- **No network-loaded assets.** Bundle every font, image, or icon via esbuild's `loader` table or `assets/`; don't reference `https://...` URLs from stylesheets.

## 7. Legacy V1 code

`relay/` (the cloud relay server) and `protocol/` (the three-party protocol contract generator) implement V1 (account + cloud relay). The current agent no longer connects to any relay — its relay client code has been removed — but some plugin-side V1 modules (`relayClient.ts`, `remoteService.ts`, `authClient.ts`, `deviceClient.ts`, etc.) are still in active use and consume types generated from `protocol/generated/`. **Don't delete or substantially rewrite them** without a separate, explicit removal task — they support functionality that's still shipping. They build and test independently, and CI keeps exercising them so they don't silently rot:

```bash
cd protocol && npm ci && npm test
cargo test --manifest-path relay/Cargo.toml
```

You don't need to build or regenerate them unless you're specifically working on that code.

## 8. Test fixture privacy

Never commit real personal data in tests, snapshots, docs, or sample payloads — local usernames, absolute home-directory paths, cloud-storage paths, vault names, private note/course/exam folder names, or anything that looks like a credential or API key. Use neutral fixtures instead: `/Users/example/Documents/Notes/Example.md`, `F:\example-vault\notes\demo`, `notes/path-example.md`, `archive/12345/`. Before finalizing test changes with paths or note-like names in them, scan the changed files for anything that slipped in by accident.

## 9. Commits and pull requests

- Subjects start with a Conventional Commit type: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`, `build:`, `ci:`, `perf:`, `style:`, `revert:`. Scopes are encouraged when they add clarity (`feat(terminal):`, `fix(settings):`). A leading emoji is fine as long as the required prefix is still there.
- Keep subjects short, imperative, and specific.
- PRs should summarize user-visible impact, list local verification steps, link related issues, and include screenshots or a short recording for UI changes.
- Update `CHANGELOG.md` whenever packaging, release notes, or versioned behavior changes, so release automation maps the change to the right version section.

## 10. Looking for the design history?

Earlier drafts, staged implementation plans, and one-off handover/verification checklists that used to live under `docs/需求/` and `docs/开发/` were folded into this guide where still relevant, or retired once superseded by shipped code. If you need that level of detail, `git log -- docs/` still has it.
