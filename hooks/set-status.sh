#!/bin/bash
# Claude Code hook script for updating claude-poke pet state
# Usage: set-status.sh <state> [message]

STATE_FILE="/tmp/claude-pet-status.json"
# Get timestamp in milliseconds (truncate nanoseconds to 13 digits)
TIMESTAMP=$(date +%s%N | cut -c1-13)
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
STATE="${1:-Sleeping}"
MESSAGE="${2:-}"

STATE_JSON="\"$STATE\""
if [ "$STATE" = "Notify" ] && [ -n "$MESSAGE" ]; then
    STATE_JSON="{\"Notify\": \"$MESSAGE\"}"
fi

cat > "$STATE_FILE.tmp" << EOF
{
    "state": $STATE_JSON,
    "timestamp": $TIMESTAMP,
    "session_id": "$SESSION_ID",
    "message": "$MESSAGE"
}
EOF

mv -f "$STATE_FILE.tmp" "$STATE_FILE"
