#![cfg(all(feature = "tray", target_os = "linux"))]

use std::sync::{mpsc, LazyLock};

use ksni::blocking::TrayMethods;
use ksni::menu::*;

/// Menu event IDs sent from tray callbacks to the app event loop.
pub const MENU_SHOW_HIDE: &str = "show_hide";
pub const MENU_ALWAYS_ON_TOP: &str = "always_on_top";
pub const MENU_QUIT: &str = "quit";

/// Cached 16x16 ARGB32 icon (gold circle). Computed once, reused on every property check.
static DEFAULT_ICON: LazyLock<ksni::Icon> = LazyLock::new(create_default_icon);

/// Tray state that implements `ksni::Tray`.
/// Menu callbacks send event IDs through the channel to the main thread.
#[derive(Debug)]
struct TrayState {
    visible: bool,
    always_on_top: bool,
    tx: mpsc::Sender<String>,
}

impl ksni::Tray for TrayState {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Claude Poke".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![DEFAULT_ICON.clone()]
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(MENU_SHOW_HIDE.into());
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx_show = self.tx.clone();
        let tx_top = self.tx.clone();
        let tx_quit = self.tx.clone();
        let visible = self.visible;
        let always_on_top = self.always_on_top;

        vec![
            StandardItem {
                label: if visible { "隐藏" } else { "显示" }.into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_show.send(MENU_SHOW_HIDE.into());
                }),
                ..Default::default()
            }
            .into(),
            CheckmarkItem {
                label: "置顶".into(),
                checked: always_on_top,
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_top.send(MENU_ALWAYS_ON_TOP.into());
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "退出".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_quit.send(MENU_QUIT.into());
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Manages the system tray icon and receives menu events.
pub struct SystemTray {
    handle: ksni::blocking::Handle<TrayState>,
    rx: mpsc::Receiver<String>,
}

impl SystemTray {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();

        let state = TrayState {
            visible: true,
            always_on_top: true,
            tx,
        };

        let handle = state.spawn()?;

        Ok(Self { handle, rx })
    }

    /// Update the show/hide menu item label based on visibility.
    pub fn set_visibility_label(&self, visible: bool) {
        if self.handle.update(move |state| {
            state.visible = visible;
        }).is_none() {
            log::warn!("Tray service unavailable, menu state not updated");
        }
    }

    /// Sync the "置顶" checkmark state.
    pub fn set_always_on_top(&self, on: bool) {
        if self.handle.update(move |state| {
            state.always_on_top = on;
        }).is_none() {
            log::warn!("Tray service unavailable, menu state not updated");
        }
    }

    /// Check for menu events. Returns the menu item ID string if something was clicked.
    pub fn try_recv_menu_event(&self) -> Option<String> {
        self.rx.try_recv().ok()
    }
}

/// Create a default 16x16 ARGB32 icon (gold circle).
/// ARGB32 in network byte order: [A, R, G, B] per pixel.
fn create_default_icon() -> ksni::Icon {
    let size = 16usize;
    let mut data = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let dx = (x as f64 - 7.5).powi(2);
            let dy = (y as f64 - 7.5).powi(2);
            if dx + dy < 49.0 {
                // ARGB32 network byte order: [A, R, G, B]
                data[idx] = 255;     // A
                data[idx + 1] = 255; // R
                data[idx + 2] = 215; // G
                data[idx + 3] = 0;   // B
            }
        }
    }
    ksni::Icon {
        width: size as i32,
        height: size as i32,
        data,
    }
}
