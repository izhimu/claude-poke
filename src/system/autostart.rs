#[allow(unused_imports)]
use log::{info, warn};

/// Check if autostart is enabled for this application.
#[allow(dead_code)]
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        let autostart_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("autostart");
        let desktop_file = autostart_dir.join("claude-poke.desktop");
        desktop_file.exists()
    }

    #[cfg(target_os = "macos")]
    {
        // Check LaunchAgents plist
        let plist_path = dirs::home_dir()
            .unwrap_or_default()
            .join("Library/LaunchAgents/com.claude-poke.plist");
        plist_path.exists()
    }

    #[cfg(target_os = "windows")]
    {
        // Check registry Run key
        // For simplicity, we'll just check if a shortcut exists
        false
    }
}

/// Enable or disable autostart.
#[allow(dead_code)]
pub fn set_autostart(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    if enable {
        enable_autostart()
    } else {
        disable_autostart()
    }
}

#[cfg(target_os = "linux")]
fn enable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("autostart");
    std::fs::create_dir_all(&autostart_dir)?;

    let exe_path = std::env::current_exe()?;
    let desktop_content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Claude Poke\n\
         Exec={}\n\
         Hidden=false\n\
         NoDisplay=false\n\
         X-GNOME-Autostart-enabled=true\n",
        exe_path.display()
    );

    let desktop_file = autostart_dir.join("claude-poke.desktop");
    std::fs::write(&desktop_file, desktop_content)?;
    info!("Autostart enabled: {}", desktop_file.display());
    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_default()
        .join("autostart");
    let desktop_file = autostart_dir.join("claude-poke.desktop");
    if desktop_file.exists() {
        std::fs::remove_file(&desktop_file)?;
        info!("Autostart disabled");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn enable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    let plist_path = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/com.claude-poke.plist");

    let exe_path = std::env::current_exe()?;
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.claude-poke</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
        exe_path.display()
    );

    std::fs::write(&plist_path, plist_content)?;
    info!("Autostart enabled: {}", plist_path.display());
    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    let plist_path = dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/com.claude-poke.plist");
    if plist_path.exists() {
        std::fs::remove_file(&plist_path)?;
        info!("Autostart disabled");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn enable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    warn!("Windows autostart not yet implemented");
    Ok(())
}

#[cfg(target_os = "windows")]
fn disable_autostart() -> Result<(), Box<dyn std::error::Error>> {
    warn!("Windows autostart not yet implemented");
    Ok(())
}
