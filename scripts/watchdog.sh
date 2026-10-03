#!/usr/bin/env bash
# Idempotent watchdog for the main onyx-node listener.
# Healthy -> exit 0 silently. Dead -> restart via scripts/start-listen.sh and
# log one line to status/watchdog.log. flock prevents concurrent runs.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p status
# 0) Ygg-Route-Waechter sicherstellen (idempotent, eigenes Pidfile).
#    Steht vor flock: der Listener erbt fd 9 und haelt den Lock, solange er lebt.
GUARD=/workspace/scripts/ygg-route-guard.sh
GPID=/workspace/lumina-state/ygg-route-guard.pid
if [[ -x $GUARD ]] && ! { [[ -s $GPID ]] && kill -0 "$(cat "$GPID")" 2>/dev/null; }; then
  setsid nohup "$GUARD" >/dev/null 2>&1 < /dev/null &
  echo "$(date '+%Y-%m-%dT%H:%M:%S%z') ygg-route-guard gestartet" >> status/watchdog.log
fi
exec 9>status/.watchdog.lock
flock -n 9 || exit 0

PIDFILE=status/onyx-listen.pid
LOG=status/watchdog.log
ts() { date '+%Y-%m-%dT%H:%M:%S%z'; }


is_main_listener() {  # $1 = pid; matches argv exactly, not substrings of shells
  local -a argv
  mapfile -d '' -t argv < "/proc/$1/cmdline" 2>/dev/null || return 1
  (( ${#argv[@]} > 1 )) || return 1
  local exe=${argv[0]##*/} a has_listen=0
  for a in "${argv[@]:1}"; do
    [[ $a == --test-peer ]] && return 1
    [[ $a == listen ]] && has_listen=1
  done
  (( has_listen )) || return 1
  [[ $exe == onyx-node ]] && return 0
  [[ $exe == cargo && " ${argv[*]} " == *" run "*" onyx-node "* ]] && return 0
  return 1
}

# 1) recorded PID alive and really the listener?
if [[ -s $PIDFILE ]]; then
  pid=$(cat "$PIDFILE")
  if [[ "$pid" =~ ^[0-9]+$ ]] && kill -0 "$pid" 2>/dev/null && is_main_listener "$pid"; then
    exit 0
  fi
fi

# 2) listener running under another PID (e.g. started by hand)? adopt it.
for p in $(pgrep -f 'onyx-node' || true); do
  [[ $p == $$ ]] && continue
  if is_main_listener "$p"; then
    echo "$p" > "$PIDFILE"
    echo "$(ts) adopted running listener pid=$p (pidfile was stale)" >> "$LOG"
    exit 0
  fi
done

# 3) dead -> restart
old=$(cat "$PIDFILE" 2>/dev/null || echo none)
"$ROOT/scripts/start-listen.sh" >/dev/null
echo "$(ts) restart: listener not running (old pid=$old) -> new pid=$(cat "$PIDFILE")" >> "$LOG"
