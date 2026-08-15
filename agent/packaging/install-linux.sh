#!/usr/bin/env bash
#
# Installs termy-agent as a user service on Ubuntu (doc 7.4).
#
# One-liner remote install (downloads the latest release, no local checkout
# needed):
#
#   curl -fsSL https://raw.githubusercontent.com/jiang-zhong-xi/Termy/main/agent/packaging/install-linux.sh | bash
#
# Or, from a local checkout/build, pass the binary path explicitly:
#
#   ./agent/packaging/install-linux.sh /path/to/termy-agent
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
RELEASE_REPO="jiang-zhong-xi/Termy"
RELEASE_ASSET="termy-agent-linux-x64"
# Only set when this script is run from a real file (a local checkout), not
# piped through `curl | bash`, where BASH_SOURCE[0] is empty - falling back to
# the current directory there would risk matching an unrelated binary that
# happens to sit at a candidate path below.
SOURCE_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
  SOURCE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
fi

say() { printf '\033[1m==>\033[0m %s\n' "$1"; }
die() { printf '\033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }

[[ "$(id -u)" -ne 0 ]] || die "run this as the ordinary user that will own the agent, not as root"

TMP_DOWNLOAD_DIR=""
cleanup() { [[ -z "${TMP_DOWNLOAD_DIR}" ]] || rm -rf "${TMP_DOWNLOAD_DIR}"; }
trap cleanup EXIT

BINARY="${1:-}"
if [[ -z "${BINARY}" && -n "${SOURCE_DIR}" ]]; then
  for candidate in \
    "${SOURCE_DIR}/../target/release/termy-agent" \
    "${SOURCE_DIR}/../target/debug/termy-agent" \
    "${SOURCE_DIR}/termy-agent"; do
    [[ -x "${candidate}" ]] && BINARY="${candidate}" && break
  done
fi

if [[ -z "${BINARY}" ]]; then
  command -v curl >/dev/null || die "no local termy-agent binary found and curl is not installed to fetch one; pass a binary path as the first argument"
  ARCH="$(uname -m)"
  [[ "${ARCH}" == "x86_64" ]] || die "no prebuilt agent for architecture '${ARCH}' (only x86_64 Linux builds are published); pass a local binary path as the first argument"

  say "no local binary found; downloading the latest release from GitHub"
  TMP_DOWNLOAD_DIR="$(mktemp -d)"
  RELEASE_BASE="https://github.com/${RELEASE_REPO}/releases/latest/download"
  curl -fsSL "${RELEASE_BASE}/${RELEASE_ASSET}" -o "${TMP_DOWNLOAD_DIR}/${RELEASE_ASSET}" \
    || die "download failed: ${RELEASE_BASE}/${RELEASE_ASSET}"
  curl -fsSL "${RELEASE_BASE}/${RELEASE_ASSET}.sha256" -o "${TMP_DOWNLOAD_DIR}/${RELEASE_ASSET}.sha256" \
    || die "download failed: ${RELEASE_BASE}/${RELEASE_ASSET}.sha256"
  (cd "${TMP_DOWNLOAD_DIR}" && sha256sum -c "${RELEASE_ASSET}.sha256") \
    || die "checksum verification failed for the downloaded binary"
  chmod +x "${TMP_DOWNLOAD_DIR}/${RELEASE_ASSET}"
  BINARY="${TMP_DOWNLOAD_DIR}/${RELEASE_ASSET}"
fi
[[ -n "${BINARY}" && -x "${BINARY}" ]] || die "pass the path to the termy-agent binary as the first argument"

say "installing the binary into ${BIN_DIR}"
mkdir -p "${BIN_DIR}"
install -m 0755 "${BINARY}" "${BIN_DIR}/termy-agent"

say "installing the user unit into ${UNIT_DIR}"
mkdir -p "${UNIT_DIR}"
if [[ -n "${SOURCE_DIR}" && -f "${SOURCE_DIR}/${UNIT_NAME}" ]]; then
  install -m 0644 "${SOURCE_DIR}/${UNIT_NAME}" "${UNIT_DIR}/${UNIT_NAME}"
else
  # curl | bash has no checkout to read the unit file from - embed it so the
  # installer stays a single self-contained script.
  UNIT_TMP="$(mktemp)"
  trap 'rm -f "${UNIT_TMP}"; cleanup' EXIT
  cat > "${UNIT_TMP}" <<'UNIT'
[Unit]
Description=Termy remote agent
Documentation=https://github.com/jiang-zhong-xi/Termy
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=%h/.local/bin/termy-agent run
Restart=always
RestartSec=5
Environment=RUST_LOG=termy_agent=info
NoNewPrivileges=true
StandardOutput=journal
StandardError=journal
SyslogIdentifier=termy-agent

[Install]
WantedBy=default.target
UNIT
  install -m 0644 "${UNIT_TMP}" "${UNIT_DIR}/${UNIT_NAME}"
fi

say "enabling lingering so the agent survives logout"
if loginctl show-user "$(id -un)" --property=Linger 2>/dev/null | grep -q 'Linger=yes'; then
  echo "    already enabled"
else
  echo "    this is the one step that needs elevation (doc 7.4)"
  sudo loginctl enable-linger "$(id -un)"
fi

say "reloading the user manager"
systemctl --user daemon-reload

say "starting the service"
systemctl --user enable --now "${UNIT_NAME}"

say "waiting for the agent to publish a connection code"
CODE_LINE=""
for _ in $(seq 1 20); do
  LINE="$("${BIN_DIR}/termy-agent" status 2>/dev/null | grep '^code' || true)"
  if [[ -n "${LINE}" && "${LINE}" != *"none"* && "${LINE}" != *"unavailable"* ]]; then
    CODE_LINE="${LINE}"
    break
  fi
  sleep 1
done

echo
echo "Installed and running as $(id -un). Nothing here runs as root after installation."
echo
if [[ -n "${CODE_LINE}" ]]; then
  echo "  ${CODE_LINE}"
  echo
  echo 'Paste that code into Termy'"'"'s "添加设备" in Obsidian.'
else
  echo "Couldn't read the connection code yet (still reaching a relay). Check it with:"
  echo
  echo "  termy-agent status"
fi
echo
echo "Useful commands:"
echo "  termy-agent status                          show the connection code again"
echo "  journalctl --user -u ${UNIT_NAME} -f        tail the agent's logs"
