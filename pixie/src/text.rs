use fontdue::{Font, FontSettings};
use image::{imageops, Rgba, RgbaImage};
use std::path::Path;

const PIXEL_FONT: &[u8] = include_bytes!("../fonts/PressStart2P-Regular.ttf");
const SMOOTH_FONT: &[u8] = include_bytes!("../fonts/Poppins-SemiBold.ttf");

/// Load font from a custom path, or the bundled default for the chosen mode
/// (Poppins for smooth text, Press Start 2P for pixel text).
pub fn load_font(custom_path: Option<&Path>, smooth: bool) -> Result<Font, String> {
    match custom_path {
        Some(path) => {
            let data = std::fs::read(path)
                .map_err(|err| format!("failed to read font file {}: {err}", path.display()))?;
            Font::from_bytes(data, FontSettings::default())
                .map_err(|err| format!("failed to parse font {}: {err}", path.display()))
        }
        None => {
            let data = if smooth { SMOOTH_FONT } else { PIXEL_FONT };
            Font::from_bytes(data, FontSettings::default())
                .map_err(|err| format!("failed to parse bundled font: {err}"))
        }
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
    /// Target text size in output pixels (font size for smooth, cap height for pixel).
    pub pixel_size: f32,
    /// Fill color.
    pub color: [u8; 3],
    /// Draw a dark outline (applied on the native grid so it scales crisply).
    pub outline: bool,
    /// Extra gap between glyphs (logical pixels for pixel text, output px for smooth).
    pub tracking: i32,
    /// Render smoothly (antialiased) instead of on the pixel grid.
    pub smooth: bool,
}

/// Render text to an image, dispatching to the smooth or pixel renderer.
pub fn render_text(text: &str, font: &Font, style: TextStyle) -> RgbaImage {
    if style.smooth {
        render_text_smooth(text, font, style)
    } else {
        render_text_pixel(text, font, style)
    }
}

/// Render text into an RgbaImage at the font's native pixel grid, then scale up
/// with nearest-neighbor so pixels stay crisp. Outline (if enabled) is added on
/// the native grid so it is a single logical pixel thick after scaling.
fn render_text_pixel(text: &str, font: &Font, style: TextStyle) -> RgbaImage {
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

/// Render text smoothly (antialiased) with a proportional font, for the clean
/// modern look that pairs with crisp emoji. Supersampled then downscaled with
/// Lanczos3 for extra-smooth edges.
fn render_text_smooth(text: &str, font: &Font, style: TextStyle) -> RgbaImage {
    let ss = 3.0f32; // supersample factor
    let size = style.pixel_size * ss;
    let tracking = style.tracking as f32 * ss;

    // Use consistent line metrics so every glyph shares one baseline.
    let line = font.horizontal_line_metrics(size);
    let ascent = line.map(|m| m.ascent).unwrap_or(size * 0.8);
    let descent = line.map(|m| m.descent).unwrap_or(-size * 0.2);

    // Rasterize glyphs and measure the total advance.
    let mut glyphs = Vec::new();
    let mut total_w = 0.0f32;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        total_w += metrics.advance_width + tracking;
        glyphs.push((metrics, bitmap));
    }
    total_w -= tracking; // no trailing gap

    let margin = (size * 0.12).ceil(); // breathing room for round glyph overshoot
    let hi_w = (total_w + margin * 2.0).ceil().max(1.0) as u32;
    let hi_h = (ascent - descent + margin * 2.0).ceil().max(1.0) as u32;
    let mut hi = RgbaImage::new(hi_w, hi_h);

    let [r, g, b] = style.color;
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
                    // Keep the strongest coverage where glyphs overlap.
                    if cov > px.0[3] {
                        px.0 = [r, g, b, cov];
                    }
                }
            }
        }
        pen_x += metrics.advance_width + tracking;
    }

    // Downscale to final size with a high-quality filter.
    let out_w = ((hi_w as f32) / ss).round().max(1.0) as u32;
    let out_h = ((hi_h as f32) / ss).round().max(1.0) as u32;
    imageops::resize(&hi, out_w, out_h, imageops::FilterType::Lanczos3)
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
