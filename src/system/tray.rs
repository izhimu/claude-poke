#![cfg(feature = "tray")]

use log::error;
use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Create and manage the system tray icon.
pub struct SystemTray {
    _tray_icon: TrayIcon,
}

impl SystemTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let show_item = MenuItem::new("Show/Hide", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append_items(&[&show_item, &quit_item])?;

        // Create a simple 16x16 icon (gold circle)
        let icon_data = create_default_icon();
        let icon = tray_icon::Icon::from_rgba(icon_data, 16, 16)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Claude Poke")
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
        })
    }

    /// Check for menu events. Returns the menu item ID if something was clicked.
    pub fn try_recv_menu_event(&self) -> Option<String> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            Some(format!("{:?}", event.id))
        } else {
            None
        }
    }

    /// Check for tray icon events (click, etc.).
    pub fn try_recv_tray_event(&self) -> bool {
        TrayIconEvent::receiver().try_recv().is_ok()
    }
}

/// Create a default 16x16 RGBA icon (simple bird silhouette).
fn create_default_icon() -> Vec<u8> {
    let mut data = vec![0u8; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let idx = (y * 16 + x) * 4;
            let cx = (x as f64 - 7.5).powi(2);
            let cy = (y as f64 - 7.5).powi(2);
            if cx + cy < 49.0 {
                data[idx] = 255; // R
                data[idx + 1] = 215; // G (gold)
                data[idx + 2] = 0; // B
                data[idx + 3] = 255; // A
            }
        }
    }
    data
}
