#!/usr/bin/env bash
# LOCAL LOOPBACK TEST PEER — not a real mesh peer.
# Starts a second listener (node-id onyx-probe) on 127.0.0.1:4711 that dials
# the main listener on 127.0.0.1:4710. Its pulses carry
# extra.test_peer = "local loopback test peer — not a real mesh peer".
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
mkdir -p status-probe
cargo build --bin onyx-node -q
nohup ./target/debug/onyx-node listen --node-id onyx-probe --topic nexus/mesh/v0 --interval 30 \
  --identity state-probe/identity.json --runtime state-probe/runtime.json \
  --mesh-key state-probe/mesh.key --status-dir status-probe \
  --listen-addr /ip4/127.0.0.1/tcp/4711 --peers /ip4/127.0.0.1/tcp/4710 \
  --peers-file '' --no-mdns --test-peer >> status-probe/listen.log 2>&1 &
echo $! > status-probe/onyx-probe.pid
echo "started onyx-probe (local test peer) pid=$(cat status-probe/onyx-probe.pid)"
