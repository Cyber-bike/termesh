set -e
cd /root/termy-remote
export PATH=/root/.cargo/bin:$PATH
rm -rf /tmp/e2e/relay.db* /tmp/e2e/recv /tmp/e2e/cfg /tmp/e2e/*.lock
mkdir -p /tmp/e2e/recv
export TERMY_DB_PATH=/tmp/e2e/relay.db TERMY_PEPPER=$(head -c32 /dev/urandom|base64) TERMY_JWT_SECRET=$(head -c32 /dev/urandom|base64) TERMY_RELAY_URL=wss://relay.example.com/v1/agent/ws TERMY_BIND=127.0.0.1:18090
echo 'hunter2hunter2' | ./relay/target/debug/termy-relay useradd alice >/dev/null 2>&1
RUST_LOG=termy_relay=debug ./relay/target/debug/termy-relay serve > /tmp/e2e/relay.log 2>&1 & RELAY=$!
sleep 3
B=http://127.0.0.1:18090
TOK=$(curl -s -X POST $B/v1/auth/login -H 'content-type: application/json' -d '{"login":"alice","password":"hunter2hunter2"}' | python3 -c "import sys,json;print(json.load(sys.stdin)['accessToken'])")
CODE=$(curl -s -X POST $B/v1/devices/pairing-codes -H "authorization: Bearer $TOK" -H 'content-type: application/json' -d '{}' | python3 -c "import sys,json;print(json.load(sys.stdin)['pairingCode'])")
export XDG_CONFIG_HOME=/tmp/e2e/cfg XDG_RUNTIME_DIR=/tmp/e2e TERMY_AGENT_ALLOW_INSECURE=1
./agent/target/debug/termy-agent bind --code "$CODE" --relay "$B" --name e2e-box >/dev/null 2>&1
python3 -c "
import json
p='/tmp/e2e/cfg/termy-agent/config.json'
d=json.load(open(p)); d['relayUrl']='ws://127.0.0.1:18090/v1/agent/ws'; d['receiveRoot']='/tmp/e2e/recv'
json.dump(d,open(p,'w'),indent=2); print('DEVICE_ID='+d['deviceId'])" > /tmp/e2e/dev.env
. /tmp/e2e/dev.env
RUST_LOG=termy_agent=debug ./agent/target/debug/termy-agent run > /tmp/e2e/agent.log 2>&1 & AGENT=$!
sleep 4
echo "=== status ==="; ./agent/target/debug/termy-agent status | head -5
echo "=== 端到端 ==="
set +e
BASE=$B TOKEN=$TOK DEVICE_ID=$DEVICE_ID RECV=/tmp/e2e/recv node /tmp/e2e/driver/drive.js
DRIVER=$?
kill $AGENT $RELAY 2>/dev/null; wait 2>/dev/null
exit $DRIVER
