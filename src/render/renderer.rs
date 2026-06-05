use crate::config::WindowConfig;
use anyhow::Result;
use log::debug;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

type SoftContext = Context<OwnedDisplayHandle>;

/// Software-based pixel renderer using softbuffer (no GPU context).
///
/// Unlike the previous pixels/wgpu backend, this uses platform-native
/// software rendering (XShm on X11, wl_shm on Wayland, Core Graphics on macOS),
/// eliminating ~100MB of GPU driver overhead.
pub struct PetRenderer {
    surface: Surface<OwnedDisplayHandle, &'static Window>,
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl PetRenderer {
    /// Create a new renderer for the given window.
    ///
    /// # Safety
    /// The `context` and `window` must outlive the renderer. This is guaranteed
    /// because both are stored in the App struct and dropped after the renderer.
    pub unsafe fn new(
        context: &'static SoftContext,
        window: &'static Window,
        config: &WindowConfig,
    ) -> Result<Self> {
        let (logical_w, logical_h) = config.logical_size();
        let window_size = window.inner_size();
        let buf_w = window_size.width.max(1);
        let buf_h = window_size.height.max(1);

        let mut surface = Surface::new(context, window)
            .map_err(|e| anyhow::anyhow!("softbuffer Surface::new: {e}"))?;
        surface.resize(
            NonZeroU32::new(buf_w).unwrap(),
            NonZeroU32::new(buf_h).unwrap(),
        ).map_err(|e| anyhow::anyhow!("softbuffer resize: {e}"))?;

        debug!(
            "Softbuffer renderer initialized: {}x{} (physical), {}x{} (logical)",
            buf_w, buf_h, logical_w, logical_h
        );

        Ok(Self {
            surface,
            buffer: vec![0u32; (buf_w * buf_h) as usize],
            width: buf_w,
            height: buf_h,
        })
    }

    /// Get the physical width of the pixel buffer.
    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the physical height of the pixel buffer.
    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Present the pixel buffer to the screen.
    pub fn render(&mut self) -> Result<()> {
        let mut sb_buf = self.surface.buffer_mut()
            .map_err(|e| anyhow::anyhow!("softbuffer buffer_mut: {e}"))?;
        let len = self.buffer.len().min(sb_buf.len());
        sb_buf[..len].copy_from_slice(&self.buffer[..len]);
        sb_buf.present()
            .map_err(|e| anyhow::anyhow!("softbuffer present: {e}"))?;
        Ok(())
    }

    /// Resize the surface when the window size changes.
    pub fn resize_surface(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        self.surface.resize(
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        ).map_err(|e| anyhow::anyhow!("softbuffer resize: {e}"))?;
        self.buffer.resize((width * height) as usize, 0);
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Clear the frame buffer with transparent pixels.
    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    /// Draw a sprite from raw RGBA data at the given position.
    /// Performs alpha blending with the existing buffer contents.
    /// Sprite data is RGBA bytes (from the `image` crate).
    /// Buffer format is ARGB8888 u32 (softbuffer native format on Linux).
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
                let dst_idx = y * frame_w + x;

                if src_idx + 3 < sprite_data.len() && dst_idx < self.buffer.len() {
                    let a = sprite_data[src_idx + 3] as u32;
                    if a > 0 {
                        let r = sprite_data[src_idx] as u32;
                        let g = sprite_data[src_idx + 1] as u32;
                        let b = sprite_data[src_idx + 2] as u32;

                        let dst = self.buffer[dst_idx];
                        let dst_a = (dst >> 24) & 0xFF;
                        let dst_r = (dst >> 16) & 0xFF;
                        let dst_g = (dst >> 8) & 0xFF;
                        let dst_b = dst & 0xFF;

                        // Alpha blending (pre-multiplied style)
                        let inv_a = 255 - a;
                        let out_a = a + (dst_a * inv_a) / 255;
                        let out_r = (r * a + dst_r * inv_a) / 255;
                        let out_g = (g * a + dst_g * inv_a) / 255;
                        let out_b = (b * a + dst_b * inv_a) / 255;

                        self.buffer[dst_idx] =
                            (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b;
                    }
                }
            }
        }
    }
}
