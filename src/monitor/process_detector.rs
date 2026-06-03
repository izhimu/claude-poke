use log::debug;
use sysinfo::System;

/// Detects whether the Claude Code process is running.
#[allow(dead_code)]
pub struct ProcessDetector {
    system: System,
}

#[allow(dead_code)]
impl ProcessDetector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    /// Refresh process list and check if Claude Code is running.
    pub fn is_claude_running(&mut self) -> bool {
        self.system.refresh_all();

        let claude_running = self.system.processes().iter().any(|(_, process)| {
            let name = process.name().to_string_lossy().to_lowercase();
            // Check for common Claude Code process names
            name.contains("claude")
                || name.contains("claude-code")
                || name.contains("claude_desktop")
        });

        if claude_running {
            debug!("Claude Code process detected");
        }

        claude_running
    }
}

impl Default for ProcessDetector {
    fn default() -> Self {
        Self::new()
    }
}
