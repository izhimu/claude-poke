use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes, WindowLevel};

/// Create window attributes for the pet window.
/// The window is transparent, borderless, and skip taskbar.
pub fn pet_window_attributes(title: &str, width: u32, height: u32, always_on_top: bool) -> WindowAttributes {
    let level = if always_on_top {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    };
    Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(level)
        .with_resizable(false)
}
