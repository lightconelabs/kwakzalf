use fontdue::{Font, FontSettings};
use image::{imageops, RgbaImage};
use std::path::Path;

const DEFAULT_FONT: &[u8] = include_bytes!("../fonts/Nunito-ExtraBold.ttf");

/// Load font from a custom path, or the bundled default (Nunito ExtraBold).
pub fn load_font(custom_path: Option<&Path>) -> Result<Font, String> {
    let data = match custom_path {
        Some(path) => std::fs::read(path)
            .map_err(|err| format!("failed to read font file {}: {err}", path.display()))?,
        None => DEFAULT_FONT.to_vec(),
    };
    Font::from_bytes(data, FontSettings::default())
        .map_err(|err| format!("failed to parse font: {err}"))
}

/// Render text smoothly (antialiased) into an image. Supersampled then downscaled
/// with Lanczos3 for extra-smooth edges. `tracking` adds letter spacing (px).
pub fn render_text(text: &str, font: &Font, size_px: f32, color: [u8; 3], tracking: i32) -> RgbaImage {
    let ss = 3.0f32;
    let size = size_px * ss;
    let tracking = tracking as f32 * ss;

    // Consistent line metrics so every glyph shares one baseline.
    let line = font.horizontal_line_metrics(size);
    let ascent = line.map(|m| m.ascent).unwrap_or(size * 0.8);
    let descent = line.map(|m| m.descent).unwrap_or(-size * 0.2);

    let mut glyphs = Vec::new();
    let mut total_w = 0.0f32;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        total_w += metrics.advance_width + tracking;
        glyphs.push((metrics, bitmap));
    }
    total_w -= tracking; // no trailing gap

    let margin = (size * 0.12).ceil();
    let hi_w = (total_w + margin * 2.0).ceil().max(1.0) as u32;
    let hi_h = (ascent - descent + margin * 2.0).ceil().max(1.0) as u32;
    let mut hi = RgbaImage::new(hi_w, hi_h);

    let [r, g, b] = color;
    let baseline = ascent + margin;
    let mut pen_x = margin;
    for (metrics, bitmap) in &glyphs {
        let gx = pen_x + metrics.xmin as f32;
        let gy = baseline - (metrics.height as i32 + metrics.ymin) as f32;
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let cov = bitmap[row * metrics.width + col];
                if cov == 0 {
                    continue;
                }
                let fx = (gx + col as f32) as i32;
                let fy = (gy + row as f32) as i32;
                if fx >= 0 && fy >= 0 && (fx as u32) < hi_w && (fy as u32) < hi_h {
                    let px = hi.get_pixel_mut(fx as u32, fy as u32);
                    if cov > px.0[3] {
                        px.0 = [r, g, b, cov];
                    }
                }
            }
        }
        pen_x += metrics.advance_width + tracking;
    }

    let out_w = ((hi_w as f32) / ss).round().max(1.0) as u32;
    let out_h = ((hi_h as f32) / ss).round().max(1.0) as u32;
    imageops::resize(&hi, out_w, out_h, imageops::FilterType::Lanczos3)
}
