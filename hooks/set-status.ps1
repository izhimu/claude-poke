# Claude Code hook script for updating claude-poke pet state (Windows)
# Usage: .\set-status.ps1 <state> [message]
#
# Sends state via HTTP POST to the claude-poke daemon.
# Falls back silently if the daemon is not running.

param(
    [Parameter(Mandatory=$true)]
    [string]$State,
    [string]$Message = ""
)

$port = if ($env:CLAUDE_POKE_PORT) { $env:CLAUDE_POKE_PORT } else { "9527" }
$timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$sessionId = if ($env:CLAUDE_SESSION_ID) { $env:CLAUDE_SESSION_ID } else { "unknown" }

# Handle Notify state with message as nested object
if ($State -eq "Notify") {
    $stateValue = @{ Notify = $Message }
} else {
    $stateValue = $State
}

$json = @{
    state = $stateValue
    timestamp = $timestamp
    session_id = $sessionId
    message = $Message
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "http://127.0.0.1:${port}/status" `
        -Method Post `
        -Body $json `
        -ContentType "application/json" `
        -TimeoutSec 2 `
        -ErrorAction Stop | Out-Null
} catch {
    # Daemon not running — silently ignore
}
