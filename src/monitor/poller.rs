use crate::state::StatusFile;
use log::{debug, warn};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Polls the status file at regular intervals as a fallback monitor.
pub struct StatusPoller {
    status_path: PathBuf,
    interval: Duration,
    last_poll: Instant,
    last_modified: Option<std::time::SystemTime>,
}

impl StatusPoller {
    pub fn new(status_path: PathBuf, interval_ms: u64) -> Self {
        Self {
            status_path,
            interval: Duration::from_millis(interval_ms),
            last_poll: Instant::now(),
            last_modified: None,
        }
    }

    /// Check if the status file has changed since last poll.
    /// Returns Some(StatusFile) if changed, None otherwise.
    pub fn poll(&mut self) -> Option<StatusFile> {
        let now = Instant::now();
        if now.duration_since(self.last_poll) < self.interval {
            return None;
        }
        self.last_poll = now;

        let metadata = match std::fs::metadata(&self.status_path) {
            Ok(m) => m,
            Err(_) => return None,
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(_) => return None,
        };

        if self.last_modified == Some(modified) {
            return None;
        }

        self.last_modified = Some(modified);

        match std::fs::read_to_string(&self.status_path) {
            Ok(content) => match StatusFile::from_json(&content) {
                Ok(status) => {
                    debug!("Poller detected status change: {:?}", status.state);
                    Some(status)
                }
                Err(e) => {
                    warn!("Poller: failed to parse status file: {}", e);
                    None
                }
            },
            Err(e) => {
                debug!("Poller: failed to read status file: {}", e);
                None
            }
        }
    }
}
