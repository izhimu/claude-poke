use anyhow::Result;
use image::GenericImageView;
use std::path::Path;

/// A sprite sheet loaded from a PNG file.
/// The sheet contains frames arranged horizontally.
pub struct SpriteSheet {
    data: Vec<u8>,
    total_width: u32,
    frame_width: u32,
    frame_height: u32,
    frame_count: u32,
}

impl SpriteSheet {
    /// Load a sprite sheet from a PNG file.
    /// Each frame is assumed to be `frame_width` x `frame_height` pixels,
    /// arranged horizontally in the image.
    pub fn load(path: &Path, frame_width: u32, frame_height: u32) -> Result<Self> {
        let img = image::open(path)?;
        let (total_width, total_height) = img.dimensions();

        if total_height != frame_height {
            anyhow::bail!(
                "Sprite sheet height {} doesn't match expected frame height {}",
                total_height,
                frame_height
            );
        }

        let frame_count = total_width / frame_width;
        if frame_count == 0 {
            anyhow::bail!("Sprite sheet too narrow for any frames");
        }

        let rgba = img.to_rgba8();
        let data = rgba.into_raw();

        Ok(Self {
            data,
            total_width,
            frame_width,
            frame_height,
            frame_count,
        })
    }

    /// Get the raw RGBA data for a specific frame.
    /// Frames are arranged horizontally, so frame `index` occupies
    /// x=[index*frame_width, (index+1)*frame_width), y=[0, frame_height).
    pub fn frame_data(&self, index: u32) -> Vec<u8> {
        let index = index % self.frame_count;
        let bytes_per_pixel = 4u32;
        let frame_bytes = (self.frame_width * self.frame_height * bytes_per_pixel) as usize;
        let mut frame_data = vec![0u8; frame_bytes];

        // Copy frame pixels from the horizontal strip
        for y in 0..self.frame_height {
            let src_start = ((y * self.total_width + index * self.frame_width) * bytes_per_pixel) as usize;
            let src_end = src_start + (self.frame_width * bytes_per_pixel) as usize;
            let dst_start = (y * self.frame_width * bytes_per_pixel) as usize;
            let dst_end = dst_start + (self.frame_width * bytes_per_pixel) as usize;
            frame_data[dst_start..dst_end].copy_from_slice(&self.data[src_start..src_end]);
        }

        frame_data
    }

    #[allow(dead_code)]
    pub fn frame_width(&self) -> u32 {
        self.frame_width
    }

    #[allow(dead_code)]
    pub fn frame_height(&self) -> u32 {
        self.frame_height
    }

    #[allow(dead_code)]
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}

/// Create a placeholder sprite (solid color rectangle) for when real art isn't available.
pub fn create_placeholder_sprite(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for pixel in data.chunks_exact_mut(4) {
        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];
        pixel[3] = color[3];
    }
    data
}
