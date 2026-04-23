use image::ImageFormat;
use image::{DynamicImage, RgbaImage};
use std::fs;
use std::io::Cursor;
use std::path::Path;

use crate::config::{THUMBNAIL_MAX_HEIGHT, THUMBNAIL_MAX_WIDTH};

pub fn process_and_save(
    raw_data: &[u8],
    width: u32,
    height: u32,
    stride: u32,
    out_path: &str,
) -> Result<(), String> {
    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        let row_start = (y * stride) as usize;
        let row_end = row_start.saturating_add((width * 4) as usize);
        let Some(row) = raw_data.get(row_start..row_end) else {
            return Err("Frame buffer ended unexpectedly.".into());
        };

        for pixel in row.chunks_exact(4) {
            rgba_bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
        }
    }

    let image = RgbaImage::from_raw(width, height, rgba_bytes)
        .ok_or("Failed to construct RGBA image from screencopy buffer.")?;
    let (target_width, target_height) = fit_within_bounds(width, height);
    let resized = image::imageops::resize(
        &image,
        target_width,
        target_height,
        image::imageops::FilterType::Triangle,
    );

    let mut encoded = Vec::new();
    DynamicImage::ImageRgba8(resized)
        .write_to(&mut Cursor::new(&mut encoded), ImageFormat::Png)
        .map_err(|error| format!("Failed to encode PNG: {error}"))?;

    if let Ok(existing) = fs::read(out_path) {
        if existing == encoded {
            return Ok(());
        }
    }

    if let Some(parent) = Path::new(out_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create thumbnail directory: {error}"))?;
    }

    fs::write(out_path, encoded).map_err(|error| format!("Failed to write thumbnail: {error}"))
}

fn fit_within_bounds(width: u32, height: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (1, 1);
    }

    let width_scale = THUMBNAIL_MAX_WIDTH as f32 / width as f32;
    let height_scale = THUMBNAIL_MAX_HEIGHT as f32 / height as f32;
    let scale = width_scale.min(height_scale).min(1.0);

    let scaled_width = ((width as f32 * scale).round() as u32).max(1);
    let scaled_height = ((height as f32 * scale).round() as u32).max(1);

    (scaled_width, scaled_height)
}

#[cfg(test)]
mod tests {
    use super::fit_within_bounds;

    #[test]
    fn fits_large_frames_inside_thumbnail_bounds() {
        assert_eq!(fit_within_bounds(1920, 1080), (320, 180));
        assert_eq!(fit_within_bounds(1080, 1920), (101, 180));
    }

    #[test]
    fn keeps_small_frames_at_original_size() {
        assert_eq!(fit_within_bounds(160, 90), (160, 90));
    }
}
