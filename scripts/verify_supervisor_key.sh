#!/usr/bin/env bash
# verify_supervisor_key.sh — check ML-KEM-1024 supervisor public key fingerprint
# against the committed reference in keys/supervisor_kyber1024_metadata.json
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
META="$REPO_ROOT/keys/supervisor_kyber1024_metadata.json"
PUB="$REPO_ROOT/keys/supervisor_kyber1024_public.txt"

if [[ ! -f "$META" || ! -f "$PUB" ]]; then
  echo "ERROR: supervisor key files missing in $REPO_ROOT/keys"
  exit 2
fi

EXPECTED=$(python3 -c "import json; print(json.load(open('$META'))['fingerprint'])")
B64=$(python3 -c "
import base64,re
text=open('$PUB').read()
m=re.search(r'PUBLIC KEY \(Base64\):\s*(\S+)', text)
print(m.group(1) if m else '')
")

if [[ -z "$B64" ]]; then
  echo "ERROR: could not parse Base64 public key from $PUB"
  exit 3
fi

RAW=$(printf '%s' "$B64" | base64 -d | xxd -p -c 256 | tr -d '\n')
ACTUAL=$(printf '%s' "$RAW" | python3 -c "
import hashlib,sys
data=bytes.fromhex(sys.stdin.read())
print(hashlib.sha3_256(data).hexdigest()[:16].upper())
")

echo "expected: $EXPECTED"
echo "actual:   $ACTUAL"

if [[ "$ACTUAL" == "$EXPECTED" ]]; then
  echo "OK: supervisor key fingerprint matches"
  exit 0
else
  echo "MISMATCH: supervisor public key may have been tampered with"
  exit 1
fi
