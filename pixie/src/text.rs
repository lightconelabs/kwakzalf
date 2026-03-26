use fontdue::{Font, FontSettings};
use image::{imageops, Rgba, RgbaImage};
use std::path::Path;

const DEFAULT_FONT: &[u8] = include_bytes!("../fonts/PressStart2P-Regular.ttf");

/// Load font from path or use bundled default
pub fn load_font(custom_path: Option<&Path>) -> Result<Font, String> {
    match custom_path {
        Some(path) => {
            let data = std::fs::read(path)
                .map_err(|err| format!("failed to read font file {}: {err}", path.display()))?;
            Font::from_bytes(data, FontSettings::default())
                .map_err(|err| format!("failed to parse font {}: {err}", path.display()))
        }
        None => Font::from_bytes(DEFAULT_FONT, FontSettings::default())
            .map_err(|err| format!("failed to parse bundled font: {err}")),
    }
}

/// Find the smallest size at which the font renders cleanly on its pixel grid.
/// Pixel fonts have discrete sizes where glyphs snap to grid — we find the smallest one
/// that fits within our target pixel_size.
fn detect_native_size(font: &Font, max_size: f32) -> f32 {
    let mut prev_height = 0;
    for size_int in 4..=(max_size as u32) {
        let size = size_int as f32;
        let (metrics, _) = font.rasterize('M', size);
        let h = metrics.height;
        if h > 0 && h == prev_height && size_int > 6 {
            // Height stopped growing — previous size was the native grid
            return (size_int - 1) as f32;
        }
        prev_height = h;
    }
    // No grid detected — use the target size directly (non-pixel font)
    max_size
}

/// Render text to an RgbaImage with the given font at pixel_size and color.
/// Renders at 8px native size (Press Start 2P's design grid) then scales up.
pub fn render_text(text: &str, font: &Font, pixel_size: f32, color: [u8; 3]) -> RgbaImage {
    let color = Rgba([color[0], color[1], color[2], 255]);
    let black = Rgba([0, 0, 0, 255]);

    // Detect native pixel size: rasterize 'M' at increasing sizes until height jumps,
    // indicating we've left the font's native grid. Default to pixel_size if no grid found.
    let native_size = detect_native_size(font, pixel_size);
    let scale = (pixel_size / native_size).round().max(1.0) as u32;

    // Rasterize all glyphs at native size
    let mut glyphs = Vec::new();
    let mut total_width = 0u32;
    let mut max_ascent = 0i32;
    let mut max_descent = 0i32;

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, native_size);
        let ascent = metrics.height as i32 + metrics.ymin;
        let descent = -metrics.ymin;
        max_ascent = max_ascent.max(ascent);
        max_descent = max_descent.max(descent);
        let advance = metrics.advance_width.ceil() as u32;
        total_width += advance;
        glyphs.push((metrics, bitmap, advance));
    }

    // Canvas at native size — no outline yet
    let native_h = (max_ascent + max_descent) as u32;
    let native_w = total_width;
    let mut img = RgbaImage::new(native_w.max(1), native_h.max(1));

    let baseline_y = max_ascent;

    // Draw glyphs centered within their monospace cell
    let mut x = 0i32;
    for (metrics, bitmap, advance) in &glyphs {
        // Center the glyph horizontally within its cell
        let cell_width = *advance as i32;
        let glyph_width = metrics.xmin + metrics.width as i32;
        let x_pad = (cell_width - glyph_width) / 2;
        let gx = x + metrics.xmin + x_pad;
        let gy = baseline_y - metrics.height as i32 - metrics.ymin;
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                if bitmap[row * metrics.width + col] > 128 {
                    let fx = gx + col as i32;
                    let fy = gy + row as i32;
                    if fx >= 0 && fy >= 0 && (fx as u32) < img.width() && (fy as u32) < img.height()
                    {
                        img.put_pixel(fx as u32, fy as u32, color);
                    }
                }
            }
        }
        x += cell_width;
    }

    // Scale up with nearest-neighbor for crisp pixels
    let scaled = if scale > 1 {
        imageops::resize(
            &img,
            img.width() * scale,
            img.height() * scale,
            imageops::FilterType::Nearest,
        )
    } else {
        img
    };

    // Add 1px black outline on the scaled image
    let sw = scaled.width();
    let sh = scaled.height();
    let mut outlined = RgbaImage::new(sw + 2, sh + 2);

    // Draw black in 8 neighbors of each opaque pixel
    for y in 0..sh {
        for x in 0..sw {
            if scaled.get_pixel(x, y).0[3] > 0 {
                for dy in 0..=2u32 {
                    for dx in 0..=2u32 {
                        let ox = x + dx;
                        let oy = y + dy;
                        if ox < outlined.width() && oy < outlined.height() {
                            outlined.put_pixel(ox, oy, black);
                        }
                    }
                }
            }
        }
    }

    // Draw scaled text on top (offset by 1 for the outline margin)
    imageops::overlay(&mut outlined, &scaled, 1, 1);

    outlined
}
