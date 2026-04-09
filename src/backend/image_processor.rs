use image::imageops::FilterType;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::fs;
use std::path::Path;

/// Extracts raw RGBA/BGRA byte buffers mapped from Wayland SHM into a resized encoded PNG.
/// By processing aggressively out-of-band, the UI overlay remains unblocked and perfectly smooth.
pub fn process_and_save(raw_data: &[u8], width: u32, height: u32, stride: u32, out_path: &str) {
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;
            if offset + 3 < raw_data.len() {
                // Emulating typical little-endian BGRA Wayland Compositor Pixels
                let b = raw_data[offset];
                let g = raw_data[offset + 1];
                let r = raw_data[offset + 2];
                let a = raw_data[offset + 3];
                img.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }
    }

    // Task 3: Performance Optimization - hardware-agnostic Triangle filter interpolation
    // Downscales native 4K/1080p frame buffers to maximum thumbnail 400x300 limits.
    let resized = image::imageops::resize(&img, 400, 300, FilterType::Triangle);

    if let Some(parent) = Path::new(out_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = resized.save(out_path);
}
