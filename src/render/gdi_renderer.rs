//! Windows GDI renderer using UpdateLayeredWindow for true per-pixel alpha transparency.
//!
//! The wgpu/Vulkan swap chain used by the `pixels` crate does not reliably support
//! alpha compositing with DWM on Windows. This module replaces the wgpu presentation
//! path with `UpdateLayeredWindow`, which is the Windows API specifically designed for
//! per-pixel alpha windows.

use crate::config::WindowConfig;
use anyhow::Result;
use log::debug;

/// GDI-based renderer for Windows that uses UpdateLayeredWindow for transparency.
pub struct GdiRenderer {
    /// The BGRA pixel buffer (same layout as pixels crate)
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    /// The HWND of the target window
    hwnd: *mut core::ffi::c_void,
    /// GDI device context for the memory DC
    mem_dc: *mut core::ffi::c_void,
    /// GDI bitmap handle
    bitmap: *mut core::ffi::c_void,
    /// Pointer to the DIB pixel data
    dib_bits: *mut u8,
}

// SAFETY: HWND and GDI handles are used only from the main thread via the event loop
unsafe impl Send for GdiRenderer {}
unsafe impl Sync for GdiRenderer {}

// Win32 API declarations
extern "system" {
    fn CreateCompatibleDC(hdc: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn DeleteDC(hdc: *mut core::ffi::c_void) -> i32;
    fn DeleteObject(ho: *mut core::ffi::c_void) -> i32;
    fn SelectObject(hdc: *mut core::ffi::c_void, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn UpdateLayeredWindow(
        hwnd: *mut core::ffi::c_void,
        hdc_dst: *mut core::ffi::c_void,
        ppt_dst: *mut POINT,
        psize: *mut SIZE,
        hdc_src: *mut core::ffi::c_void,
        ppt_src: *mut POINT,
        cr_key: u32,
        pblend: *const BLENDFUNCTION,
        dw_flags: u32,
    ) -> i32;
    fn CreateDIBSection(
        hdc: *mut core::ffi::c_void,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppv_bits: *mut *mut u8,
        h_section: *mut core::ffi::c_void,
        offset: u32,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct SIZE {
    cx: i32,
    cy: i32,
}

#[repr(C)]
struct BLENDFUNCTION {
    blend_op: u8,
    blend_flags: u8,
    source_constant_alpha: u8,
    alpha_format: u8,
}

#[repr(C)]
struct BITMAPINFOHEADER {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[repr(C)]
struct BITMAPINFO {
    bmi_header: BITMAPINFOHEADER,
    bmi_colors: [u32; 1], // placeholder, not used for 32-bit
}

const ULW_ALPHA: u32 = 0x00000002;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
const AC_SRC_ALPHA: u8 = 0x01;

impl GdiRenderer {
    /// Create a new GDI renderer for the given window.
    pub fn new(window: &winit::window::Window, config: &WindowConfig) -> Result<Self> {
        use raw_window_handle::HasWindowHandle;

        let (logical_w, logical_h) = config.logical_size();
        let window_size = window.inner_size();
        // Use physical pixel size for DIB (UpdateLayeredWindow needs physical pixels)
        let phys_w = window_size.width;
        let phys_h = window_size.height;

        let handle = window.window_handle()?;
        let raw_handle = handle.as_raw();

        let hwnd = match raw_handle {
            raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get() as *mut _,
            _ => anyhow::bail!("Not a Win32 window"),
        };

        debug!("Physical window size: {}x{}, logical: {}x{}", phys_w, phys_h, logical_w, logical_h);

        // Create a 32-bit BGRA DIB section matching the physical window size.
        // UpdateLayeredWindow requires the DIB to match the window dimensions.
        let bmi = BITMAPINFO {
            bmi_header: BITMAPINFOHEADER {
                bi_size: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                bi_width: phys_w as i32,
                // Negative height = top-down DIB (origin at top-left)
                bi_height: -(phys_h as i32),
                bi_planes: 1,
                bi_bit_count: 32,
                bi_compression: BI_RGB,
                bi_size_image: phys_w * phys_h * 4,
                bi_x_pels_per_meter: 0,
                bi_y_pels_per_meter: 0,
                bi_clr_used: 0,
                bi_clr_important: 0,
            },
            bmi_colors: [0],
        };

        let mut dib_bits: *mut u8 = std::ptr::null_mut();
        let bitmap = unsafe { CreateDIBSection(std::ptr::null_mut(), &bmi, DIB_RGB_COLORS, &mut dib_bits, std::ptr::null_mut(), 0) };
        if bitmap.is_null() || dib_bits.is_null() {
            anyhow::bail!("CreateDIBSection failed");
        }

        let mem_dc = unsafe { CreateCompatibleDC(std::ptr::null_mut()) };
        if mem_dc.is_null() {
            unsafe { DeleteObject(bitmap); }
            anyhow::bail!("CreateCompatibleDC failed");
        }

        unsafe { SelectObject(mem_dc, bitmap); }

        // Clear the DIB to fully transparent
        let total_pixels = (phys_w * phys_h) as usize;
        unsafe {
            std::ptr::write_bytes(dib_bits, 0, total_pixels * 4);
        }

        // Ensure WS_EX_LAYERED is set on the window.
        // winit 0.30 does NOT set this with with_transparent(true), but
        // UpdateLayeredWindow requires it.
        extern "system" {
            fn GetWindowLongPtrW(hwnd: *mut core::ffi::c_void, n_index: i32) -> isize;
            fn SetWindowLongPtrW(hwnd: *mut core::ffi::c_void, n_index: i32, dw_new_long: isize) -> isize;
        }
        const GWL_EXSTYLE: i32 = -20;
        const WS_EX_LAYERED: isize = 0x00080000;
        unsafe {
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            if ex_style & WS_EX_LAYERED == 0 {
                debug!("Setting WS_EX_LAYERED for UpdateLayeredWindow");
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED);
            }
        }

        // Establish initial UpdateLayeredWindow state
        let mut window_pt = POINT { x: 0, y: 0 };
        let mut window_size_win = SIZE {
            cx: window_size.width as i32,
            cy: window_size.height as i32,
        };
        let blend = BLENDFUNCTION {
            blend_op: 0, // AC_SRC_OVER
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };
        let mut src_pt = POINT { x: 0, y: 0 };
        unsafe {
            UpdateLayeredWindow(
                hwnd,
                std::ptr::null_mut(),
                &mut window_pt,
                &mut window_size_win,
                mem_dc,
                &mut src_pt,
                0,
                &blend,
                ULW_ALPHA,
            );
        }

        debug!("GDI renderer initialized: {}x{} (physical)", phys_w, phys_h);

        Ok(Self {
            buffer: vec![0u8; (phys_w * phys_h * 4) as usize],
            width: phys_w,
            height: phys_h,
            hwnd,
            mem_dc,
            bitmap,
            dib_bits,
        })
    }

    /// Get a mutable reference to the pixel buffer (BGRA format).
    pub fn frame_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Get the logical width of the pixel buffer.
    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the logical height of the pixel buffer.
    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Present the pixel buffer to the screen via UpdateLayeredWindow.
    pub fn render(&self) -> Result<()> {
        // Copy our buffer to the DIB section
        let buf_size = (self.width * self.height * 4) as usize;
        unsafe {
            std::ptr::copy_nonoverlapping(self.buffer.as_ptr(), self.dib_bits, buf_size);
        }

        // Get the window's current position and size
        // We need to get the window rect to pass correct position
        extern "system" {
            fn GetWindowRect(hwnd: *mut core::ffi::c_void, rect: *mut RECT) -> i32;
        }
        #[repr(C)]
        struct RECT {
            left: i32,
            top: i32,
            right: i32,
            bottom: i32,
        }

        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        unsafe { GetWindowRect(self.hwnd, &mut rect); }

        let mut window_pt = POINT { x: rect.left, y: rect.top };
        let mut window_size = SIZE {
            cx: rect.right - rect.left,
            cy: rect.bottom - rect.top,
        };
        let blend = BLENDFUNCTION {
            blend_op: 0, // AC_SRC_OVER
            blend_flags: 0,
            source_constant_alpha: 255,
            alpha_format: AC_SRC_ALPHA,
        };
        let mut src_pt = POINT { x: 0, y: 0 };

        let ok = unsafe {
            UpdateLayeredWindow(
                self.hwnd,
                std::ptr::null_mut(),
                &mut window_pt,
                &mut window_size,
                self.mem_dc,
                &mut src_pt,
                0,
                &blend,
                ULW_ALPHA,
            )
        };

        if ok == 0 {
            anyhow::bail!("UpdateLayeredWindow failed");
        }

        Ok(())
    }

    /// No-op for GDI renderer (UpdateLayeredWindow handles sizing).
    #[allow(dead_code)]
    pub fn resize_surface(&mut self, _width: u32, _height: u32) -> Result<()> {
        Ok(())
    }

    /// Clear the frame buffer with transparent pixels.
    pub fn clear(&mut self) {
        for pixel in self.buffer.chunks_exact_mut(4) {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 0;
        }
    }

    /// Draw a sprite from raw RGBA data at the given position.
    /// The sprite is scaled to fit the physical buffer (nearest-neighbor).
    pub fn draw_sprite(
        &mut self,
        sprite_data: &[u8],
        sprite_w: u32,
        sprite_h: u32,
        offset_x: u32,
        offset_y: u32,
    ) {
        let frame_w = self.width as usize;
        let frame_h = self.height as usize;

        // Calculate scale factors (physical buffer / logical sprite)
        let scale_x = frame_w as f32 / sprite_w as f32;
        let scale_y = frame_h as f32 / sprite_h as f32;

        for y in 0..frame_h {
            for x in 0..frame_w {
                // Map physical pixel back to sprite pixel (nearest-neighbor)
                let src_x = ((x as f32 - offset_x as f32) / scale_x) as usize;
                let src_y = ((y as f32 - offset_y as f32) / scale_y) as usize;

                if src_x >= sprite_w as usize || src_y >= sprite_h as usize {
                    continue;
                }

                let src_idx = (src_y * sprite_w as usize + src_x) * 4;
                let dst_x = x;
                let dst_y = y;

                let dst_idx = (dst_y * frame_w + dst_x) * 4;

                if src_idx + 3 < sprite_data.len() && dst_idx + 3 < self.buffer.len() {
                    let a = sprite_data[src_idx + 3] as f32 / 255.0;
                    if a > 0.0 {
                        // Pre-multiplied alpha blending
                        // Sprite data is RGBA, GDI DIB is BGRA — swap R and B channels
                        let inv_a = 1.0 - a;
                        // B channel: sprite[src+2] → buffer[dst+0]
                        self.buffer[dst_idx] =
                            (sprite_data[src_idx + 2] as f32 * a + self.buffer[dst_idx] as f32 * inv_a) as u8;
                        // G channel: sprite[src+1] → buffer[dst+1]
                        self.buffer[dst_idx + 1] =
                            (sprite_data[src_idx + 1] as f32 * a + self.buffer[dst_idx + 1] as f32 * inv_a) as u8;
                        // R channel: sprite[src+0] → buffer[dst+2]
                        self.buffer[dst_idx + 2] =
                            (sprite_data[src_idx] as f32 * a + self.buffer[dst_idx + 2] as f32 * inv_a) as u8;
                        // A channel: sprite[src+3] → buffer[dst+3]
                        self.buffer[dst_idx + 3] =
                            (a * 255.0 + self.buffer[dst_idx + 3] as f32 * inv_a) as u8;
                    }
                }
            }
        }
    }
}

impl Drop for GdiRenderer {
    fn drop(&mut self) {
        unsafe {
            if !self.mem_dc.is_null() {
                DeleteDC(self.mem_dc);
            }
            if !self.bitmap.is_null() {
                DeleteObject(self.bitmap);
            }
        }
    }
}
