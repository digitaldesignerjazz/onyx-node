#!/usr/bin/env bash
# Idempotent watchdog for the main onyx-node listener.
# Healthy -> exit 0 silently. Dead -> restart via scripts/start-listen.sh and
# log one line to status/watchdog.log. flock prevents concurrent runs.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p status
exec 9>status/.watchdog.lock
flock -n 9 || exit 0

PIDFILE=status/onyx-listen.pid
LOG=status/watchdog.log
ts() { date '+%Y-%m-%dT%H:%M:%S%z'; }

is_main_listener() {  # $1 = pid
  local cmd
  cmd=$(tr '\0' ' ' < "/proc/$1/cmdline" 2>/dev/null) || return 1
  [[ "$cmd" == *onyx-node*listen* && "$cmd" != *--test-peer* ]]
}

# 1) recorded PID alive and really the listener?
if [[ -s $PIDFILE ]]; then
  pid=$(cat "$PIDFILE")
  if [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" 2>/dev/null && is_main_listener "$pid"; then
    exit 0
  fi
fi

# 2) listener running under another PID (e.g. started by hand)? adopt it.
for p in $(pgrep -f 'onyx-node.*listen' || true); do
  if is_main_listener "$p"; then
    # prefer the cargo parent if present
    echo "$p" > "$PIDFILE"
    echo "$(ts) adopted running listener pid=$p (pidfile was stale)" >> "$LOG"
    exit 0
  fi
done

# 3) dead -> restart
old=$(cat "$PIDFILE" 2>/dev/null || echo none)
"$ROOT/scripts/start-listen.sh" >/dev/null
echo "$(ts) restart: listener not running (old pid=$old) -> new pid=$(cat "$PIDFILE")" >> "$LOG"
