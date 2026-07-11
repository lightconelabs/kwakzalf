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

/// Style options for text rendering.
#[derive(Clone, Copy)]
pub struct TextStyle {
    /// Target cap height in output pixels.
    pub pixel_size: f32,
    /// Fill color.
    pub color: [u8; 3],
    /// Draw a dark outline (applied on the native grid so it scales crisply).
    pub outline: bool,
    /// Extra logical-pixel gap between glyph cells (letter tracking).
    pub tracking: i32,
}

/// Render text into an RgbaImage at the font's native pixel grid, then scale up
/// with nearest-neighbor so pixels stay crisp. Outline (if enabled) is added on
/// the native grid so it is a single logical pixel thick after scaling.
pub fn render_text(text: &str, font: &Font, style: TextStyle) -> RgbaImage {
    let color = Rgba([style.color[0], style.color[1], style.color[2], 255]);
    let ink = Rgba([26u8, 24, 30, 255]);

    let native_size = detect_native_size(font, style.pixel_size);
    let scale = (style.pixel_size / native_size).round().max(1.0) as u32;

    // Rasterize all glyphs at native size.
    let mut glyphs = Vec::new();
    let mut total_width = 0i32;
    let mut max_ascent = 0i32;
    let mut max_descent = 0i32;
    let tracking = style.tracking.max(0);

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, native_size);
        let ascent = metrics.height as i32 + metrics.ymin;
        let descent = -metrics.ymin;
        max_ascent = max_ascent.max(ascent);
        max_descent = max_descent.max(descent);
        let advance = metrics.advance_width.ceil() as i32;
        total_width += advance + tracking;
        glyphs.push((metrics, bitmap, advance));
    }
    total_width -= tracking; // no trailing gap

    let native_h = (max_ascent + max_descent).max(1) as u32;
    let native_w = total_width.max(1) as u32;
    // 1px margin so an optional outline has room on every side.
    let mut img = RgbaImage::new(native_w + 2, native_h + 2);
    let baseline_y = max_ascent + 1;
    let margin_x = 1i32;

    // Draw glyphs centered within their monospace cell.
    let mut x = margin_x;
    for (metrics, bitmap, advance) in &glyphs {
        let cell_width = *advance;
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
        x += cell_width + tracking;
    }

    // Optional dark outline on the native grid (1 logical pixel, 4-connected).
    if style.outline {
        img = outline_native(&img, color, ink);
    }

    // Scale up with nearest-neighbor for crisp pixels.
    if scale > 1 {
        imageops::resize(
            &img,
            img.width() * scale,
            img.height() * scale,
            imageops::FilterType::Nearest,
        )
    } else {
        img
    }
}

/// Add a 1px dark outline around the filled glyph pixels at native resolution.
fn outline_native(img: &RgbaImage, fill: Rgba<u8>, ink: Rgba<u8>) -> RgbaImage {
    let w = img.width();
    let h = img.height();
    let mut out = img.clone();
    for y in 0..h {
        for x in 0..w {
            // Only outline into empty cells adjacent to a filled glyph pixel.
            if img.get_pixel(x, y).0[3] != 0 {
                continue;
            }
            let mut touch = false;
            for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h {
                    let p = img.get_pixel(nx as u32, ny as u32);
                    if p.0 == fill.0 {
                        touch = true;
                        break;
                    }
                }
            }
            if touch {
                out.put_pixel(x, y, ink);
            }
        }
    }
    out
}
