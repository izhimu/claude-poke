#!/bin/bash
# Claude Code hook script for updating claude-poke pet state
# Usage: set-status.sh <state> [message]

STATE_FILE="/tmp/claude-pet-status.json"
# Get timestamp in milliseconds (truncate nanoseconds to 13 digits)
TIMESTAMP=$(date +%s%N | cut -c1-13)
SESSION_ID="${CLAUDE_SESSION_ID:-unknown}"
STATE="${1:-Sleeping}"
MESSAGE="${2:-}"

cat > "$STATE_FILE" << EOF
{
    "state": "$STATE",
    "timestamp": $TIMESTAMP,
    "session_id": "$SESSION_ID",
    "message": "$MESSAGE"
}
EOF
