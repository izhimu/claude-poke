use std::path::PathBuf;

/// Default HTTP port for the status server.
pub const DEFAULT_HTTP_PORT: u16 = 9527;

/// Get the HTTP port for the status server.
/// Can be overridden via the CLAUDE_POKE_PORT environment variable.
pub fn get_http_port() -> u16 {
    std::env::var("CLAUDE_POKE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_HTTP_PORT)
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
