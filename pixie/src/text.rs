use fontdue::{Font, FontSettings};
use image::{imageops, Rgba, RgbaImage};
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

/// Render text to an RgbaImage with the given font at pixel_size and color.
/// Renders at 8px native size (Press Start 2P's design grid) then scales up.
pub fn render_text(text: &str, font: &Font, pixel_size: f32, color: [u8; 3]) -> RgbaImage {
    let color = Rgba([color[0], color[1], color[2], 255]);
    let black = Rgba([0, 0, 0, 255]);

    // Render at native 8px grid for crisp pixel font
    let native_size = 8.0f32;
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
        total_width += metrics.advance_width.ceil() as u32;
        glyphs.push((metrics, bitmap));
    }

    // Canvas at native size with 1px margin for outline
    let margin = 1u32;
    let native_h = (max_ascent + max_descent) as u32 + margin * 2;
    let native_w = total_width + margin * 2;
    let mut img = RgbaImage::new(native_w.max(1), native_h.max(1));

    let baseline_y = margin as i32 + max_ascent;

    // Draw function: places each glyph's bitmap at given offsets
    let draw = |img: &mut RgbaImage, offsets: &[(i32, i32)], px: Rgba<u8>| {
        let mut x = margin as i32;
        for (metrics, bitmap) in &glyphs {
            let gx = x + metrics.xmin;
            let gy = baseline_y - metrics.height as i32 - metrics.ymin;
            for row in 0..metrics.height {
                for col in 0..metrics.width {
                    if bitmap[row * metrics.width + col] > 128 {
                        for &(dx, dy) in offsets {
                            let fx = gx + col as i32 + dx;
                            let fy = gy + row as i32 + dy;
                            if fx >= 0 && fy >= 0 && (fx as u32) < img.width() && (fy as u32) < img.height() {
                                img.put_pixel(fx as u32, fy as u32, px);
                            }
                        }
                    }
                }
            }
            x += metrics.advance_width.ceil() as i32;
        }
    };

    // Black outline in 8 directions
    let outline_offsets: Vec<(i32, i32)> = (-1..=1i32)
        .flat_map(|dx| (-1..=1i32).map(move |dy| (dx, dy)))
        .filter(|&(dx, dy)| dx != 0 || dy != 0)
        .collect();
    draw(&mut img, &outline_offsets, black);

    // Foreground text on top
    draw(&mut img, &[(0, 0)], color);

    // Scale up with nearest-neighbor for crisp pixels
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
