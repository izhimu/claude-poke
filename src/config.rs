use std::path::PathBuf;

/// Get the path to the status file that Claude Code hooks write to.
pub fn get_status_file_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::temp_dir().join("claude-pet-status.json")
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/tmp/claude-pet-status.json")
    }
}

/// Get the application config directory.
#[allow(dead_code)]
pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude-poke")
}

/// Get the path to sprite assets.
pub fn get_assets_dir() -> PathBuf {
    // Try exe directory first, then current directory
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let candidates = [
        exe_dir.clone().map(|d| d.join("assets")),
        Some(PathBuf::from("assets")),
    ];

    for candidate in &candidates {
        if let Some(path) = candidate {
            if path.exists() {
                return path.clone();
            }
        }
    }

    // Default to current dir
    PathBuf::from("assets")
}

/// Pet window configuration.
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub scale: u32,
    #[allow(dead_code)]
    pub always_on_top: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 64,
            height: 64,
            scale: 3,
            always_on_top: true,
        }
    }
}

impl WindowConfig {
    pub fn logical_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn physical_size(&self) -> (u32, u32) {
        (self.width * self.scale, self.height * self.scale)
    }
}
