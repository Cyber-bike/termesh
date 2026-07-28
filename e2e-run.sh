#!/usr/bin/env bash
#
# End-to-end: starts a relay and an agent, then drives a real terminal session
# and a real file transfer through the control socket. This is the test that
# caught transfer.fileEnd overtaking the chunks it terminates.
#
# Uses the debug binaries - build them first with a plain `cargo build`.
set -e

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT}"

# Only prepend if rustup lives in the usual place; CI puts cargo on PATH itself.
[ -d "${HOME}/.cargo/bin" ] && export PATH="${HOME}/.cargo/bin:${PATH}"

RELAY_BIN=./relay/target/debug/termy-relay
AGENT_BIN=./agent/target/debug/termy-agent
for bin in "${RELAY_BIN}" "${AGENT_BIN}"; do
  [ -x "${bin}" ] || { echo "missing ${bin}; run: cargo build --manifest-path $(dirname "$(dirname "$(dirname "${bin}")")")/Cargo.toml" >&2; exit 1; }
done

[ -d "${ROOT}/e2e/node_modules/ws" ] || { echo "missing driver deps; run: npm --prefix e2e install" >&2; exit 1; }

WORK=/tmp/e2e
rm -rf "${WORK}"/relay.db* "${WORK}"/recv "${WORK}"/cfg "${WORK}"/*.lock
mkdir -p "${WORK}"/recv

RELAY=""; AGENT=""
cleanup() { kill ${AGENT} ${RELAY} 2>/dev/null || true; wait 2>/dev/null || true; }
trap cleanup EXIT

export TERMY_DB_PATH=${WORK}/relay.db TERMY_PEPPER=$(head -c32 /dev/urandom|base64) TERMY_JWT_SECRET=$(head -c32 /dev/urandom|base64) TERMY_RELAY_URL=wss://relay.example.com/v1/agent/ws TERMY_BIND=127.0.0.1:18090
echo 'hunter2hunter2' | ${RELAY_BIN} useradd alice >/dev/null 2>&1
RUST_LOG=termy_relay=debug ${RELAY_BIN} serve > ${WORK}/relay.log 2>&1 & RELAY=$!
sleep 3
B=http://127.0.0.1:18090
TOK=$(curl -s -X POST $B/v1/auth/login -H 'content-type: application/json' -d '{"login":"alice","password":"hunter2hunter2"}' | python3 -c "import sys,json;print(json.load(sys.stdin)['accessToken'])")
CODE=$(curl -s -X POST $B/v1/devices/pairing-codes -H "authorization: Bearer $TOK" -H 'content-type: application/json' -d '{}' | python3 -c "import sys,json;print(json.load(sys.stdin)['pairingCode'])")
export XDG_CONFIG_HOME=${WORK}/cfg XDG_RUNTIME_DIR=${WORK} TERMY_AGENT_ALLOW_INSECURE=1
${AGENT_BIN} bind --code "$CODE" --relay "$B" --name e2e-box >/dev/null 2>&1
python3 -c "
import json
p='${WORK}/cfg/termy-agent/config.json'
d=json.load(open(p)); d['relayUrl']='ws://127.0.0.1:18090/v1/agent/ws'; d['receiveRoot']='${WORK}/recv'
json.dump(d,open(p,'w'),indent=2); print('DEVICE_ID='+d['deviceId'])" > ${WORK}/dev.env
. ${WORK}/dev.env
RUST_LOG=termy_agent=debug ${AGENT_BIN} run > ${WORK}/agent.log 2>&1 & AGENT=$!
sleep 4
echo "=== status ==="; ${AGENT_BIN} status | head -5
echo "=== 端到端 ==="
set +e
BASE=$B TOKEN=$TOK DEVICE_ID=$DEVICE_ID RECV=${WORK}/recv node "${ROOT}/e2e/driver.js"
DRIVER=$?
cleanup
exit $DRIVER
