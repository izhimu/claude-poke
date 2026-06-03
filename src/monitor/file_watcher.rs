use crate::state::StatusFile;
use anyhow::Result;
use log::{debug, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

/// Watches the status file for changes using the notify crate.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<notify::Result<Event>>,
    status_path: PathBuf,
}

impl FileWatcher {
    pub fn new(status_path: PathBuf) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let watcher = Watcher::new(tx, notify::Config::default())?;

        Ok(Self {
            _watcher: watcher,
            rx,
            status_path,
        })
    }

    /// Start watching the directory containing the status file.
    pub fn start(&mut self) -> Result<()> {
        let watch_dir = self
            .status_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/"));
        debug!("Watching directory: {}", watch_dir.display());
        self._watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;
        Ok(())
    }

    /// Try to receive a status update (non-blocking).
    /// Returns Some(StatusFile) if the status file was modified.
    pub fn try_recv_status(&self) -> Option<StatusFile> {
        // Drain all pending events, only care about the latest
        let mut latest_event = None;
        while let Ok(event) = self.rx.try_recv() {
            latest_event = Some(event);
        }

        if let Some(Ok(event)) = latest_event {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    // Check if the event is about our status file
                    let status_file_name = self
                        .status_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    let relevant = event.paths.iter().any(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == status_file_name)
                            .unwrap_or(false)
                    });

                    if relevant {
                        return self.read_status_file();
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn read_status_file(&self) -> Option<StatusFile> {
        match std::fs::read_to_string(&self.status_path) {
            Ok(content) => match StatusFile::from_json(&content) {
                Ok(status) => Some(status),
                Err(e) => {
                    warn!("Failed to parse status file: {}", e);
                    None
                }
            },
            Err(e) => {
                debug!("Failed to read status file: {}", e);
                None
            }
        }
    }
}
