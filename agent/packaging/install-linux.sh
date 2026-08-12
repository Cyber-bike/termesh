#!/usr/bin/env bash
#
# Installs termy-agent as a user service on Ubuntu (doc 7.4).
#
# Everything except one step runs unprivileged. `loginctl enable-linger` needs
# root or a polkit prompt on most distributions - that is the documented
# one-off exception in doc 7.4: install with sudo once, run as an ordinary user
# forever after. Without lingering the agent is killed when the SSH session
# ends, which is exactly what MVP completion item 2 forbids.

set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
UNIT_DIR="${HOME}/.config/systemd/user"
UNIT_NAME="termy-agent.service"
SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

say() { printf '\033[1m==>\033[0m %s\n' "$1"; }
die() { printf '\033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }

[[ "$(id -u)" -ne 0 ]] || die "run this as the ordinary user that will own the agent, not as root"

BINARY="${1:-}"
if [[ -z "${BINARY}" ]]; then
  for candidate in \
    "${SOURCE_DIR}/../target/release/termy-agent" \
    "${SOURCE_DIR}/../target/debug/termy-agent" \
    "${SOURCE_DIR}/termy-agent"; do
    [[ -x "${candidate}" ]] && BINARY="${candidate}" && break
  done
fi
[[ -n "${BINARY}" && -x "${BINARY}" ]] || die "pass the path to the termy-agent binary as the first argument"

say "installing the binary into ${BIN_DIR}"
mkdir -p "${BIN_DIR}"
install -m 0755 "${BINARY}" "${BIN_DIR}/termy-agent"

say "installing the user unit into ${UNIT_DIR}"
mkdir -p "${UNIT_DIR}"
install -m 0644 "${SOURCE_DIR}/${UNIT_NAME}" "${UNIT_DIR}/${UNIT_NAME}"

say "enabling lingering so the agent survives logout"
if loginctl show-user "$(id -un)" --property=Linger 2>/dev/null | grep -q 'Linger=yes'; then
  echo "    already enabled"
else
  echo "    this is the one step that needs elevation (doc 7.4)"
  sudo loginctl enable-linger "$(id -un)"
fi

say "reloading the user manager"
systemctl --user daemon-reload

cat <<INSTRUCTIONS

Installed. Next:

  1. Start it:

       systemctl --user enable --now ${UNIT_NAME}

  2. Get the connection code (no account, no pairing service - the code is
     printed by the running agent and also readable afterwards):

       termy-agent status
       journalctl --user -u ${UNIT_NAME} -f

  3. Paste that code into Termy's "添加设备" in Obsidian.

The agent and the remote shell run as $(id -un). Nothing here runs as root
after installation.
INSTRUCTIONS
