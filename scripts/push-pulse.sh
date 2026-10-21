#!/usr/bin/env bash
# push-pulse.sh — commit and push the real Hannover-Box pulse to GitHub.
# Run on the box after the listener has written status/last_pulse.json.
# Requires a Git remote with push access (PAT or SSH key).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ ! -f status/last_pulse.json ]]; then
  echo "No status/last_pulse.json found. Abort." >&2
  exit 1
fi

NODE_ID=$(jq -r '.node_id // "unknown"' status/last_pulse.json)
TS=$(jq -r '.ts // "unknown"' status/last_pulse.json)

# Ensure envelope exists and matches
mkdir -p status
cat > status/last_mesh_envelope.json << EOF
{
  "type": "AgentHeartbeat",
  "payload": {
    "agent": "onyx-node",
    "node_id": "$NODE_ID",
    "status": "alive",
    "ts": "$TS"
  }
}
EOF

git config user.name "${GIT_USER_NAME:-Onyx Hannover}"
git config user.email "${GIT_USER_EMAIL:-onyx-hannover@local}"
git add status/last_pulse.json status/last_mesh_envelope.json

if git diff --staged --quiet; then
  echo "Pulse unchanged. Nothing to push."
  exit 0
fi

git commit -m "chore(pulse): real $NODE_ID alive @ $TS"
git push

echo "Pushed real pulse for $NODE_ID ($TS)"
