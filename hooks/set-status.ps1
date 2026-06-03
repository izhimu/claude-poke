# Claude Code hook script for updating claude-poke pet state (Windows)
# Usage: .\set-status.ps1 <state> [message]

param(
    [Parameter(Mandatory=$true)]
    [string]$State,
    [string]$Message = ""
)

$stateFile = "$env:TEMP\claude-pet-status.json"
$timestamp = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$sessionId = if ($env:CLAUDE_SESSION_ID) { $env:CLAUDE_SESSION_ID } else { "unknown" }

# Handle Notify state with message as nested object
if ($State -eq "Notify" -and $Message) {
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

Set-Content -Path "$stateFile.tmp" -Value $json
Move-Item -Path "$stateFile.tmp" -Destination $stateFile -Force
