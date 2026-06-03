#!/bin/bash
# Setup script for claude-poke hooks
# This script configures Claude Code hooks to update the desktop pet state

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo -e "${GREEN}🐾 Claude Poke Hooks Setup${NC}"
echo "================================"
echo ""

# Determine OS
OS="$(uname -s)"
case "$OS" in
    Linux*)
        SETTINGS_DIR="$HOME/.claude"
        HOOKS_DIR="$SETTINGS_DIR/hooks"
        SCRIPT_NAME="set-status.sh"
        ;;
    Darwin*)
        SETTINGS_DIR="$HOME/.claude"
        HOOKS_DIR="$SETTINGS_DIR/hooks"
        SCRIPT_NAME="set-status.sh"
        ;;
    CYGWIN*|MINGW32*|MSYS*|MINGW*)
        SETTINGS_DIR="$HOME/.claude"
        HOOKS_DIR="$SETTINGS_DIR/hooks"
        SCRIPT_NAME="set-status.ps1"
        echo -e "${YELLOW}⚠️  Windows detected. Please run set-status.ps1 manually.${NC}"
        ;;
    *)
        echo -e "${RED}❌ Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# Create hooks directory if it doesn't exist
echo "📁 Creating hooks directory: $HOOKS_DIR"
mkdir -p "$HOOKS_DIR"

# Copy the hook script
echo "📋 Copying $SCRIPT_NAME to $HOOKS_DIR"
cp "$PROJECT_DIR/hooks/$SCRIPT_NAME" "$HOOKS_DIR/"
chmod +x "$HOOKS_DIR/$SCRIPT_NAME"

# Get absolute path to the hook script
HOOK_SCRIPT_PATH="$HOOKS_DIR/$SCRIPT_NAME"

# Check if settings.json exists
SETTINGS_FILE="$SETTINGS_DIR/settings.json"
if [ ! -f "$SETTINGS_FILE" ]; then
    echo "📝 Creating new settings.json"
    echo '{}' > "$SETTINGS_FILE"
fi

# Backup existing settings
BACKUP_FILE="$SETTINGS_FILE.backup.$(date +%Y%m%d%H%M%S)"
echo "💾 Backing up settings to: $BACKUP_FILE"
cp "$SETTINGS_FILE" "$BACKUP_FILE"

# Use Python to update the settings.json
echo "⚙️  Updating settings.json with hooks configuration"
python3 << PYTHON_SCRIPT
import json
import sys

settings_file = "$SETTINGS_FILE"
hook_script = "$HOOK_SCRIPT_PATH"

# Read existing settings
with open(settings_file, 'r') as f:
    settings = json.load(f)

# Define the hooks configuration
hooks = {
    "SessionStart": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ],
    "SessionEnd": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Sleeping"
                }
            ]
        }
    ],
    "UserPromptSubmit": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "PreToolUse": [
        {
            "matcher": ".*",
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Working"
                }
            ]
        }
    ],
    "PermissionRequest": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} PendingApproval"
                }
            ]
        }
    ],
    "PostToolUse": [
        {
            "matcher": ".*",
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "PostToolUseFailure": [
        {
            "matcher": ".*",
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Error"
                }
            ]
        }
    ],
    "Notification": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Notify"
                }
            ]
        }
    ],
    "SubagentStart": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} SubAgentWorking"
                }
            ]
        }
    ],
    "SubagentStop": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "Stop": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ],
    "StopFailure": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Error"
                }
            ]
        }
    ],
    "TaskCreated": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Working"
                }
            ]
        }
    ],
    "TaskCompleted": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "PreCompact": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "PostCompact": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ],
    "PermissionDenied": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Error"
                }
            ]
        }
    ],
    "PostToolBatch": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "UserPromptExpansion": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Thinking"
                }
            ]
        }
    ],
    "TeammateIdle": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ],
    "FileChanged": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ],
    "CwdChanged": [
        {
            "hooks": [
                {
                    "type": "command",
                    "command": f"{hook_script} Idle"
                }
            ]
        }
    ]
}

# Update settings
settings["hooks"] = hooks

# Write back
with open(settings_file, 'w') as f:
    json.dump(settings, f, indent=2)

print("✅ Hooks configuration updated successfully!")
PYTHON_SCRIPT

echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo "📍 Hook script installed at: $HOOK_SCRIPT_PATH"
echo "📍 Settings updated at: $SETTINGS_FILE"
echo ""
echo "🐾 The pet will now reflect Claude Code's state:"
echo "   • Sleeping: Session not active"
echo "   • Idle: Waiting for your input"
echo "   • Thinking: Claude is processing"
echo "   • Working: Executing a tool"
echo "   • PendingApproval: Waiting for permission"
echo "   • Notify: Notification received"
echo "   • SubAgentWorking: Sub-agent spawned"
echo "   • Error: Something went wrong"
echo ""
echo "🚀 Start the pet with: cargo run"
echo ""
