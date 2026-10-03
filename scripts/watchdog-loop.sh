#!/usr/bin/env bash
# Local 2-minute watchdog loop: runs scripts/watchdog.sh every 120 s.
cd "$(dirname "$0")/.." || exit 1
echo $$ > status/watchdog-loop.pid
while true; do
  bash scripts/watchdog.sh >/dev/null 2>&1
  sleep 120
done
