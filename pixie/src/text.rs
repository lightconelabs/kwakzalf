use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};
use std::path::Path;

const DEFAULT_FONT: &[u8] = include_bytes!("../fonts/PressStart2P-Regular.ttf");

/// Load font from path or use bundled default
pub fn load_font(custom_path: Option<&Path>) -> Font {
    match custom_path {
        Some(path) => {
            let data = std::fs::read(path).expect("Failed to read font file");
            Font::from_bytes(data, FontSettings::default()).expect("Failed to parse font")
        }
        None => Font::from_bytes(DEFAULT_FONT, FontSettings::default())
            .expect("Failed to parse bundled font"),
    }
}

/// Render text to an RgbaImage with the given font at pixel_size
pub fn render_text(text: &str, font: &Font, pixel_size: f32) -> RgbaImage {
    let color = Rgba([255, 255, 255, 255]); // white text

    // Measure total width and max height
    let mut total_width = 0u32;
    let mut max_height = 0u32;
    let mut glyphs = Vec::new();

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, pixel_size);
        total_width += metrics.advance_width.ceil() as u32;
        max_height = max_height.max(metrics.height as u32);
        glyphs.push((metrics, bitmap));
    }

    let baseline = pixel_size as u32;
    let img_height = baseline + 4; // some padding
    let mut img = RgbaImage::new(total_width.max(1), img_height.max(1));

    let mut x_offset = 0i32;
    for (metrics, bitmap) in &glyphs {
        let y_start = baseline as i32 - metrics.height as i32 - metrics.ymin;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha > 0 {
                    let px = x_offset + col as i32;
                    let py = y_start + row as i32;
                    if px >= 0
                        && py >= 0
                        && (px as u32) < img.width()
                        && (py as u32) < img.height()
                    {
                        img.put_pixel(
                            px as u32,
                            py as u32,
                            Rgba([color.0[0], color.0[1], color.0[2], alpha]),
                        );
                    }
                }
            }
        }
        x_offset += metrics.advance_width.ceil() as i32;
    }

    img
}
