#![cfg(feature = "tray")]

use tray_icon::menu::{accelerator::Accelerator, CheckMenuItem, Menu, MenuEvent, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Menu item IDs.
pub const MENU_SHOW_HIDE: &str = "show_hide";
pub const MENU_ALWAYS_ON_TOP: &str = "always_on_top";
pub const MENU_QUIT: &str = "quit";

/// Create and manage the system tray icon.
pub struct SystemTray {
    _tray_icon: TrayIcon,
    show_item: MenuItem,
    top_item: CheckMenuItem,
}

impl SystemTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let menu = Menu::new();

        let show_item = MenuItem::with_id(MENU_SHOW_HIDE, "隐藏", true, None::<Accelerator>);
        let top_item = CheckMenuItem::with_id(MENU_ALWAYS_ON_TOP, "置顶", true, true, None::<Accelerator>);
        let quit_item = MenuItem::with_id(MENU_QUIT, "退出", true, None::<Accelerator>);

        menu.append_items(&[&show_item, &top_item, &quit_item])?;

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
            show_item,
            top_item,
        })
    }

    /// Update the show/hide menu item label to reflect current visibility.
    /// When visible, the action is "隐藏"; when hidden, the action is "显示".
    pub fn set_visibility_label(&self, visible: bool) {
        if visible {
            self.show_item.set_text("隐藏");
        } else {
            self.show_item.set_text("显示");
        }
    }

    /// Sync the "置顶" checkmark state.
    pub fn set_always_on_top(&self, on: bool) {
        self.top_item.set_checked(on);
    }

    /// Check for menu events. Returns the menu item ID string if something was clicked.
    pub fn try_recv_menu_event(&self) -> Option<String> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            Some(event.id.0.clone())
        } else {
            None
        }
    }

    /// Check for tray icon events (click, etc.).
    #[allow(dead_code)]
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
