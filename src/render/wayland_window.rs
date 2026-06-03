use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WaylandDisplayHandle,
    WaylandWindowHandle, WindowHandle,
};
use std::ptr::NonNull;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Proxy};

/// A wrapper around a Wayland wl_surface that implements raw-window-handle traits.
/// This allows the pixels crate (via wgpu) to create a rendering surface from it.
pub struct WaylandWindow {
    surface: NonNull<std::ffi::c_void>,
    display: NonNull<std::ffi::c_void>,
}

impl WaylandWindow {
    /// Create from a Wayland connection and surface.
    ///
    /// # Safety
    /// The surface must remain alive for the lifetime of this wrapper.
    pub unsafe fn new(connection: &Connection, surface: &WlSurface) -> Self {
        Self {
            surface: NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
            display: NonNull::new(connection.display().id().as_ptr() as *mut _).unwrap(),
        }
    }
}

unsafe impl Send for WaylandWindow {}
unsafe impl Sync for WaylandWindow {}

impl HasWindowHandle for WaylandWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = WaylandWindowHandle::new(self.surface);
        unsafe { Ok(WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Wayland(handle))) }
    }
}

impl HasDisplayHandle for WaylandWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let handle = WaylandDisplayHandle::new(self.display);
        unsafe { Ok(DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Wayland(handle))) }
    }
}
