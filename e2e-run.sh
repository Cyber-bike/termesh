#!/usr/bin/env bash
#
# End-to-end (v2.0): starts a real `termesh-agent run --loopback` process and
# drives a real terminal session against it over iroh QUIC, using the same
# @number0/iroh binding the plugin embeds directly (doc A0 verdict).
#
# There is no relay and no `agent bind` any more - v2.0's agent has no relay
# client at all (client.rs was deleted), identity is a local keypair, and
# pairing is "copy the connection code `run` prints". This also does not
# cover file transfer: `termy/transfer/1` is Phase C and not built, so
# agent/src/serve.rs closes that ALPN with PROTOCOL_ERROR.
#
# Uses the debug agent binary - build it first with:
#   cargo build --manifest-path agent/Cargo.toml
# and make sure `pnpm install` has run, so @number0/iroh is under
# node_modules/ at the repo root.
set -e

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT}"

# Only prepend if rustup lives in the usual place; CI puts cargo on PATH itself.
[ -d "${HOME}/.cargo/bin" ] && export PATH="${HOME}/.cargo/bin:${PATH}"

AGENT_BIN=./agent/target/debug/termesh-agent
[ -x "${AGENT_BIN}" ] || { echo "missing ${AGENT_BIN}; run: cargo build --manifest-path agent/Cargo.toml" >&2; exit 1; }

[ -d "${ROOT}/node_modules/@number0/iroh" ] || { echo "missing @number0/iroh; run: pnpm install" >&2; exit 1; }

WORK=${E2E_WORK:-/tmp/e2e}
rm -rf "${WORK}"
mkdir -p "${WORK}/cfg"

AGENT=""
cleanup() { kill ${AGENT} 2>/dev/null || true; wait 2>/dev/null || true; }
trap cleanup EXIT

# XDG_CONFIG_HOME/XDG_RUNTIME_DIR fully isolate config, identity, state and
# the single-instance lock (config.rs::config_dir, lock.rs::lock_path) - no
# other state on the runner is touched.
export XDG_CONFIG_HOME=${WORK}/cfg XDG_RUNTIME_DIR=${WORK}

RUST_LOG=termesh_agent=debug ${AGENT_BIN} run --loopback > ${WORK}/agent.log 2>&1 & AGENT=$!

CODE=""
for _ in $(seq 1 50); do
  CODE=$(grep -oE '^  endpoint[a-z0-9]+' "${WORK}/agent.log" | tr -d ' ' || true)
  [ -n "${CODE}" ] && break
  sleep 0.2
done
[ -n "${CODE}" ] || { echo "agent never printed a connection code within 10s" >&2; cat "${WORK}/agent.log" >&2; exit 1; }

echo "=== status ==="; ${AGENT_BIN} status | head -5
echo "=== 端到端（iroh 回环终端会话） ==="
set +e
node "${ROOT}/e2e/loopback-driver.cjs" "${CODE}"
DRIVER=$?
cleanup
exit $DRIVER
