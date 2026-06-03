#!/bin/bash
# Claude Code hook script for updating claude-poke pet state
# Usage: set-status.sh <state> [message]
#
# Sends state via HTTP POST to the claude-poke daemon.
# Falls back silently if the daemon is not running.

PORT="${CLAUDE_POKE_PORT:-9527}"
TIMESTAMP=$(date +%s%N | cut -c1-13)
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
STATE="${1:-Sleeping}"
MESSAGE="${2:-}"

STATE_JSON="\"$STATE\""
if [ "$STATE" = "Notify" ] && [ -n "$MESSAGE" ]; then
    STATE_JSON="{\"Notify\": \"$MESSAGE\"}"
fi

JSON="{\"state\":$STATE_JSON,\"timestamp\":$TIMESTAMP,\"session_id\":\"$SESSION_ID\",\"message\":\"$MESSAGE\"}"

curl -s -X POST "http://127.0.0.1:${PORT}/status" \
    -H "Content-Type: application/json" \
    -d "$JSON" \
    > /dev/null 2>&1 &
