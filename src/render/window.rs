use winit::dpi::LogicalSize;
use winit::window::{Window, WindowAttributes, WindowLevel};

/// Create window attributes for the pet window.
/// The window is transparent, borderless, and hidden from the taskbar/dock.
pub fn pet_window_attributes(title: &str, width: u32, height: u32, always_on_top: bool) -> WindowAttributes {
    let level = if always_on_top {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    };
    let attrs = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(width, height))
        .with_transparent(true)
        .with_decorations(false)
        .with_window_level(level)
        .with_resizable(false);

    // Hide from the system taskbar / dock on each platform.
    // Windows: with_skip_taskbar sets WS_EX_TOOLWINDOW which excludes from taskbar.
    #[cfg(target_os = "windows")]
    let attrs = {
        use winit::platform::windows::WindowAttributesExtWindows;
        attrs.with_skip_taskbar(true)
    };

    // X11 (Linux/GNOME via XWayland): _NET_WM_WINDOW_TYPE_UTILITY tells the
    // window manager this is a small utility window, not an app — excluded from taskbar.
    #[cfg(target_os = "linux")]
    let attrs = {
        use winit::platform::x11::{WindowAttributesExtX11, WindowType};
        attrs.with_x11_window_type(vec![WindowType::Utility])
    };

    attrs
}
