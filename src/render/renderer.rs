use crate::config::WindowConfig;
use anyhow::Result;
use log::debug;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};
use winit::window::Window;

/// The pixel renderer that manages the pixel buffer and rendering.
/// Contains both the window and the pixel buffer to avoid lifetime issues.
pub struct PetRenderer {
    pixels: Pixels<'static>,
    width: u32,
    height: u32,
}

impl PetRenderer {
    /// Create a new renderer for the given window.
    /// This takes ownership of the window by leaking its reference
    /// and reconstructing it with a 'static lifetime.
    /// SAFETY: The window must outlive the renderer. This is guaranteed
    /// because both are stored in the same App struct and dropped together.
    pub fn new(window: &Window, config: &WindowConfig) -> Result<Self> {
        let (logical_w, logical_h) = config.logical_size();
        let window_size = window.inner_size();

        // SAFETY: We leak the window reference to get 'static.
        // This is safe because the window is stored in App and dropped after the renderer.
        let window_ref: &'static Window = unsafe { &*(window as *const Window) };
        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window_ref);

        // Use PixelsBuilder to configure transparency support:
        // - clear_color with alpha=0 so the background is transparent
        // - alpha_mode PostMultiplied for proper compositing with the window
        let pixels = PixelsBuilder::new(logical_w, logical_h, surface_texture)
            .clear_color(pixels::wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            })
            .build()?;
        debug!(
            "Surface format: {:?}, alpha mode: {:?}",
            pixels.surface_texture_format(),
            pixels.context().surface_capabilities.alpha_modes
        );

        Ok(Self {
            pixels,
            width: logical_w,
            height: logical_h,
        })
    }

    /// Get a mutable reference to the pixel frame buffer.
    #[allow(dead_code)]
    pub fn frame_mut(&mut self) -> &mut [u8] {
        self.pixels.frame_mut()
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

    /// Render the pixel buffer to the screen.
    pub fn render(&self) -> Result<()> {
        self.pixels.render()?;
        Ok(())
    }

    /// Resize the surface when the window size changes.
    pub fn resize_surface(&mut self, width: u32, height: u32) -> Result<()> {
        self.pixels.resize_surface(width, height)?;
        Ok(())
    }

    /// Clear the frame buffer with a transparent background.
    /// Note: Pixel buffer format is BGRA, but setting all to 0 works regardless.
    pub fn clear(&mut self) {
        let frame = self.pixels.frame_mut();
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0; // B (or R in RGBA - doesn't matter when all 0)
            pixel[1] = 0; // G
            pixel[2] = 0; // R (or B in RGBA - doesn't matter when all 0)
            pixel[3] = 0; // A (transparent)
        }
    }

    /// Draw a sprite from raw RGBA data at the given position.
    /// sprite_data: raw RGBA pixels, sprite_w/sprite_h: sprite dimensions
    pub fn draw_sprite(
        &mut self,
        sprite_data: &[u8],
        sprite_w: u32,
        sprite_h: u32,
        offset_x: u32,
        offset_y: u32,
    ) {
        let frame = self.pixels.frame_mut();
        let frame_w = self.width as usize;

        for y in 0..sprite_h as usize {
            for x in 0..sprite_w as usize {
                let src_idx = (y * sprite_w as usize + x) * 4;
                let dst_x = offset_x as usize + x;
                let dst_y = offset_y as usize + y;

                if dst_x >= frame_w || dst_y >= self.height as usize {
                    continue;
                }

                let dst_idx = (dst_y * frame_w + dst_x) * 4;

                if src_idx + 3 < sprite_data.len() && dst_idx + 3 < frame.len() {
                    let a = sprite_data[src_idx + 3] as f32 / 255.0;
                    if a > 0.0 {
                        // Alpha blending
                        frame[dst_idx] =
                            (sprite_data[src_idx] as f32 * a + frame[dst_idx] as f32 * (1.0 - a))
                                as u8;
                        frame[dst_idx + 1] = (sprite_data[src_idx + 1] as f32 * a
                            + frame[dst_idx + 1] as f32 * (1.0 - a))
                            as u8;
                        frame[dst_idx + 2] = (sprite_data[src_idx + 2] as f32 * a
                            + frame[dst_idx + 2] as f32 * (1.0 - a))
                            as u8;
                        frame[dst_idx + 3] =
                            (a * 255.0).max(frame[dst_idx + 3] as f32) as u8;
                    }
                }
            }
        }
    }
}
