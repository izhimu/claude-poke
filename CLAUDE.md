# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`claude-poke` is a cross-platform desktop pet application written in Rust. It renders an animated cockatiel bird that visually reflects the real-time work status of Claude Code. The pet displays different animations based on Claude Code's current activity (working, idle, errors, etc.).

## Build & Development Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (opt-level=3, LTO enabled)
cargo run                      # Build and run
cargo run --features tray      # Run with system tray icon enabled
cargo test                     # Run all unit tests (15 tests)
cargo check                    # Type-check without full compilation
RUST_LOG=debug cargo run       # Run with debug logging

# Regenerate sprite sheets (requires Python + Pillow)
python3 scripts/generate_sprites.py
```

## Architecture

The application uses a layered architecture with 5 subsystems:

### Data Flow
```
Claude Code hooks (set-status.sh/ps1) → writes JSON to /tmp/claude-pet-status.json
    ↓
FileWatcher (notify) + StatusPoller (fallback) → mpsc channel
    ↓
App event loop → StateMachine → AnimationMap → FrameManager → PetRenderer
```

### Module Responsibilities

- **`src/app.rs`** — Core application loop using winit's `ApplicationHandler`. Holds all state, spawns background monitoring thread, drives rendering on `RedrawRequested`.
- **`src/config.rs`** — Path helpers for status file (`/tmp/claude-pet-status.json`), assets directory, and `WindowConfig` struct.
- **`src/state/`** — `PetState` enum (8 variants: Sleeping, Idle, Thinking, Working, PendingApproval, Notify, SubAgentWorking, Error), `StatusFile` JSON struct, `StateMachine` with priority-based transitions (Error > Notify > PendingApproval > SubAgent > Working > Thinking > Idle/Sleeping).
- **`src/monitor/`** — `FileWatcher` (notify crate) watches status file directory for Create/Modify events. `StatusPoller` checks file mtime every 100ms as fallback.
- **`src/animation/`** — `SpriteSheet` loads horizontal-strip PNGs, `FrameManager` handles time-based frame advancement, `AnimationMap` maps PetState → AnimationDef (sprite name, frame count, FPS, placeholder color).
- **`src/render/`** — `PetRenderer` wraps pixels crate for alpha-blended sprite drawing. `window.rs` creates transparent, borderless, always-on-top windows. `DragState` handles mouse drag repositioning.
- **`src/system/`** — `SystemTray` (feature-gated behind `tray`), `autostart` for platform-specific startup (.desktop on Linux, LaunchAgent on macOS).

### Key Design Decisions

- **Dual monitoring**: FileWatcher (event-driven) + StatusPoller (100ms polling) ensures reliable status detection across platforms.
- **Priority-based state transitions**: Higher-priority states (errors, notifications) cannot be overridden by lower-priority ones until they naturally expire.
- **Placeholder sprites**: When PNG sprite sheets are missing, the renderer generates colored rectangles as fallback.
- **Background thread**: File monitoring runs on a separate thread, communicating via `mpsc` channel to avoid blocking the UI.

## Status File Format

Location: `/tmp/claude-pet-status.json` (Linux/macOS), `%TEMP%\claude-pet-status.json` (Windows)

```json
{
    "state": "Working",
    "timestamp": 1717411200000,
    "session_id": "abc123",
    "message": null
}
```

Valid states: `Sleeping`, `Idle`, `Thinking`, `Working`, `PendingApproval`, `Notify`, `SubAgentWorking`, `Error`

## Hooks Integration

The `hooks/` directory contains scripts that Claude Code invokes to update the pet's status:
- `set-status.sh` / `set-status.ps1` — Write status JSON on state changes
- `settings-template.json` — Template for `~/.claude/settings.json` hooks configuration

### Quick Setup

Run the setup script to automatically configure hooks:

**Linux/macOS:**
```bash
./scripts/setup-hooks.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\setup-hooks.ps1
```

The script will:
1. Copy the hook script to `~/.claude/hooks/`
2. Update `~/.claude/settings.json` with all required hooks
3. Create a backup of your existing settings

### Hook Events Mapping

| Claude Code Hook | Pet State | Description |
|------------------|-----------|-------------|
| `SessionStart` | `Idle` | Session started, waiting for input |
| `SessionEnd` | `Sleeping` | Session ended |
| `UserPromptSubmit` | `Thinking` | User sent prompt, Claude thinking |
| `UserPromptExpansion` | `Thinking` | Slash command expanding |
| `PreToolUse` | `Working` | About to execute tool |
| `PermissionRequest` | `PendingApproval` | Waiting for user permission |
| `PermissionDenied` | `Error` | Auto-mode denied tool call |
| `PostToolUse` | `Thinking` | Tool completed, Claude thinking next step |
| `PostToolUseFailure` | `Error` | Tool execution failed |
| `PostToolBatch` | `Thinking` | Batch of tool calls completed |
| `Notification` | `Notify` | Notification received |
| `SubagentStart` | `SubAgentWorking` | Sub-agent spawned |
| `SubagentStop` | `Thinking` | Sub-agent finished, continue thinking |
| `TaskCreated` | `Working` | Task created via TaskCreate |
| `TaskCompleted` | `Thinking` | Task marked as completed |
| `TeammateIdle` | `Idle` | Agent team teammate going idle |
| `PreCompact` | `Thinking` | Context compaction starting |
| `PostCompact` | `Idle` | Context compaction finished |
| `FileChanged` | `Idle` | Watched file changed on disk |
| `CwdChanged` | `Idle` | Working directory changed |
| `Elicitation` | `PendingApproval` | MCP server requests user input during tool call |
| `ElicitationResult` | `Thinking` | User responded to MCP elicitation |
| `WorktreeCreate` | `Working` | Worktree being created via --worktree or isolation |
| `Stop` | `Idle` | Claude finished responding |
| `StopFailure` | `Error` | API error occurred |

## Platform-Specific Notes

- **Linux/macOS**: Status file at `/tmp/claude-pet-status.json`
- **Windows**: Status file at `%TEMP%\claude-pet-status.json`, uses `winapi` for window management
- **macOS**: Uses `cocoa`/`objc` crates, supports LaunchAgent for autostart
- **Feature flags**: `tray` enables system tray icon (off by default)
