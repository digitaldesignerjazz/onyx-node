#!/usr/bin/env bash
# Start the main onyx-node listener in the background (no duplicate check —
# use scripts/watchdog.sh for idempotent start/restart).
# Extra listen args (first match wins):
#   1. env ONYX_LISTEN_ARGS="--peers /ip4/x/tcp/4710"
#   2. state/listen.args (gitignored, one line; persists across watchdog restarts)
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"   # rustup stable first (libp2p deps need rustc >= 1.88)
mkdir -p status/peers
if [[ -z "${ONYX_LISTEN_ARGS:-}" && -s state/listen.args ]]; then
  ONYX_LISTEN_ARGS="$(grep -v '^[[:space:]]*#' state/listen.args | tr '\n' ' ')"
fi
# shellcheck disable=SC2086
nohup cargo run --bin onyx-node -- listen --topic nexus/mesh/v0 --interval 30 ${ONYX_LISTEN_ARGS:-} \
  >> status/listen.log 2>&1 &
echo $! > status/onyx-listen.pid
echo "started onyx-node listen pid=$(cat status/onyx-listen.pid)"
