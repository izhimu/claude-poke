# Setup script for claude-poke hooks (Windows)
# This script configures Claude Code hooks to update the desktop pet state

Write-Host "🐾 Claude Poke Hooks Setup" -ForegroundColor Green
Write-Host "================================"
Write-Host ""

# Get the directory of this script
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

# Settings directory (Claude Code on Windows uses ~/.claude, not %APPDATA%\claude)
$SettingsDir = "$env:USERPROFILE\.claude"
$HooksDir = "$SettingsDir\hooks"
$ScriptName = "set-status.ps1"

# Create hooks directory if it doesn't exist
Write-Host "📁 Creating hooks directory: $HooksDir"
if (-not (Test-Path $HooksDir)) {
    New-Item -ItemType Directory -Path $HooksDir -Force | Out-Null
}

# Copy the hook script
Write-Host "📋 Copying $ScriptName to $HooksDir"
Copy-Item "$ProjectDir\hooks\$ScriptName" "$HooksDir\$ScriptName" -Force

# Get absolute path to the hook script
$HookScriptPath = "$HooksDir\$ScriptName"

# Check if settings.json exists
$SettingsFile = "$SettingsDir\settings.json"
if (-not (Test-Path $SettingsFile)) {
    Write-Host "📝 Creating new settings.json"
    '{}' | Out-File -FilePath $SettingsFile -Encoding UTF8
}

# Backup existing settings
$BackupFile = "$SettingsFile.backup.$(Get-Date -Format 'yyyyMMddHHmmss')"
Write-Host "💾 Backing up settings to: $BackupFile"
Copy-Item $SettingsFile $BackupFile

# Update the settings.json
Write-Host "⚙️  Updating settings.json with hooks configuration"

$settings = Get-Content $SettingsFile -Raw | ConvertFrom-Json

# Define the hooks configuration
# Each hook uses shell="powershell" so Claude Code runs .ps1 scripts via PowerShell
$hooks = @{
    SessionStart = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
    SessionEnd = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Sleeping"
                }
            )
        }
    )
    UserPromptSubmit = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    PreToolUse = @(
        @{
            matcher = ".*"
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Working"
                }
            )
        }
    )
    PermissionRequest = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath PendingApproval"
                }
            )
        }
    )
    PostToolUse = @(
        @{
            matcher = ".*"
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    PostToolUseFailure = @(
        @{
            matcher = ".*"
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Error"
                }
            )
        }
    )
    Notification = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Notify"
                }
            )
        }
    )
    SubagentStart = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath SubAgentWorking"
                }
            )
        }
    )
    SubagentStop = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    Stop = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
    StopFailure = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Error"
                }
            )
        }
    )
    TaskCreated = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Working"
                }
            )
        }
    )
    TaskCompleted = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    PreCompact = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    PostCompact = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
    PermissionDenied = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Error"
                }
            )
        }
    )
    PostToolBatch = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    UserPromptExpansion = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Thinking"
                }
            )
        }
    )
    TeammateIdle = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
    FileChanged = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
    CwdChanged = @(
        @{
            hooks = @(
                @{
                    type = "command"
                    shell = "powershell"
                    command = "$HookScriptPath Idle"
                }
            )
        }
    )
}

# Update settings
$settings | Add-Member -NotePropertyName "hooks" -NotePropertyValue $hooks -Force

# Write back
$settings | ConvertTo-Json -Depth 10 | Out-File -FilePath $SettingsFile -Encoding UTF8

Write-Host ""
Write-Host "✅ Hooks configuration updated successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📍 Hook script installed at: $HookScriptPath"
Write-Host "📍 Settings updated at: $SettingsFile"
Write-Host ""
Write-Host "🐾 The pet will now reflect Claude Code's state:" -ForegroundColor Cyan
Write-Host "   • Sleeping: Session not active"
Write-Host "   • Idle: Waiting for your input"
Write-Host "   • Thinking: Claude is processing"
Write-Host "   • Working: Executing a tool"
Write-Host "   • PendingApproval: Waiting for permission"
Write-Host "   • Notify: Notification received"
Write-Host "   • SubAgentWorking: Sub-agent spawned"
Write-Host "   • Error: Something went wrong"
Write-Host ""
Write-Host "🚀 Start the pet with: cargo run" -ForegroundColor Yellow
Write-Host ""
