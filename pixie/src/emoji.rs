use image::{imageops, Rgba, RgbaImage};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded_emoji.rs"));

/// Rendering options for turning an emoji into pixel art.
#[derive(Clone, Copy)]
pub struct EmojiStyle {
    /// Logical pixels per side (the "art" resolution — chunky when small).
    pub grid: u32,
    /// Output pixels per logical pixel (nearest-neighbor zoom).
    pub zoom: u32,
    /// Number of colors in the reduced palette (0 = keep source colors).
    pub colors: u32,
    /// Draw a dark silhouette outline around the sprite.
    pub outline: bool,
}

/// Get the bundled filename stem for an emoji sequence.
pub fn emoji_codepoint(emoji: &str) -> String {
    emoji
        .chars()
        .map(|ch| format!("{:x}", ch as u32))
        .collect::<Vec<_>>()
        .join("-")
}

/// Split an emoji string into display sequences.
pub fn split_emojis(emojis: &str) -> Vec<String> {
    let chars: Vec<char> = emojis.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        let mut sequence = String::new();
        sequence.push(ch);
        i += 1;

        if is_regional_indicator(ch) {
            if i < chars.len() && is_regional_indicator(chars[i]) {
                sequence.push(chars[i]);
                i += 1;
            }
            out.push(sequence);
            continue;
        }

        loop {
            while i < chars.len() && is_sequence_modifier(chars[i]) {
                sequence.push(chars[i]);
                i += 1;
            }

            if i + 1 < chars.len() && chars[i] == '\u{200D}' {
                sequence.push(chars[i]);
                sequence.push(chars[i + 1]);
                i += 2;
                continue;
            }

            break;
        }

        out.push(sequence);
    }

    out
}

/// Try to load a custom sprite for an emoji, downscaled to the logical grid.
pub fn load_custom_sprite(emoji: &str, sprites_dir: &Path, style: EmojiStyle) -> Option<RgbaImage> {
    let codepoint = emoji_codepoint(emoji);
    let sprite_path = sprites_dir.join(format!("{}.png", codepoint));
    if !sprite_path.exists() {
        return None;
    }
    let img = image::open(&sprite_path).ok()?.into_rgba8();
    // Fit the sprite into the logical grid with a smooth downscale, then process
    // it through the same pipeline so custom sprites match the bundled look.
    let fitted = fit_into_grid(&img, style.grid);
    Some(finish_sprite(fitted, style))
}

/// Rasterize an emoji SVG into a clean, chunky pixel-art sprite.
pub fn render_emoji_from_svg(svg_data: &[u8], style: EmojiStyle) -> Option<RgbaImage> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &options).ok()?;

    // Supersample: render several device pixels per logical pixel so that when we
    // downscale to the grid, each logical pixel is a clean average of the artwork
    // (gradients and anti-aliasing collapse into flat, representative colors).
    let ss = 12u32;
    let hi = (style.grid * ss).min(768);
    let mut pixmap = Pixmap::new(hi, hi)?;

    let size = tree.size();
    let scale = (hi as f32 / size.width()).min(hi as f32 / size.height());
    let draw_w = size.width() * scale;
    let draw_h = size.height() * scale;
    // Center the artwork within the square canvas.
    let tx = (hi as f32 - draw_w) / 2.0;
    let ty = (hi as f32 - draw_h) / 2.0;
    let transform = resvg::tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let hi_img = RgbaImage::from_raw(hi, hi, pixmap.data().to_vec())?;

    // Downscale to the logical grid with a smoothing filter for averaged colors.
    let grid_img = imageops::resize(&hi_img, style.grid, style.grid, imageops::FilterType::Triangle);

    Some(finish_sprite(grid_img, style))
}

/// Shared finishing pipeline: crisp alpha, palette reduction, outline, zoom.
fn finish_sprite(mut grid_img: RgbaImage, style: EmojiStyle) -> RgbaImage {
    // Snap alpha to a hard edge — pixel art has no partial transparency.
    for px in grid_img.pixels_mut() {
        if px.0[3] < 110 {
            px.0 = [0, 0, 0, 0];
        } else {
            px.0[3] = 255;
        }
    }

    // Reduce to a cohesive limited palette (flat, artful color blocking).
    if style.colors > 0 {
        quantize_palette(&mut grid_img, style.colors as usize);
    }

    // Keep the emoji in its native grid frame. Twemoji artwork is drawn to a
    // consistent optical size inside a padded frame, so preserving the frame is
    // what keeps every emoji in a row the same visual size — trimming to the
    // content box would make padded emoji shrink and full-bleed ones dominate.
    // We only re-center the content inside that fixed frame so nothing floats.
    let grid_img = center_in_frame(grid_img);

    // Clean 1-logical-pixel dark outline hugging the silhouette.
    let bordered = if style.outline {
        add_outline(&grid_img)
    } else {
        grid_img
    };

    // Upscale to output size with crisp square pixels.
    if style.zoom > 1 {
        imageops::resize(
            &bordered,
            bordered.width() * style.zoom,
            bordered.height() * style.zoom,
            imageops::FilterType::Nearest,
        )
    } else {
        bordered
    }
}

/// Fit an arbitrary raster into a centered `grid`×`grid` canvas with a smooth downscale.
fn fit_into_grid(img: &RgbaImage, grid: u32) -> RgbaImage {
    let (w, h) = (img.width().max(1), img.height().max(1));
    let scale = (grid as f32 / w as f32).min(grid as f32 / h as f32);
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    let resized = imageops::resize(img, nw, nh, imageops::FilterType::Triangle);
    let mut canvas = RgbaImage::new(grid, grid);
    let ox = ((grid - nw) / 2) as i64;
    let oy = ((grid - nh) / 2) as i64;
    imageops::overlay(&mut canvas, &resized, ox, oy);
    canvas
}

/// Recenter the opaque content within its existing frame (no crop, no resize),
/// so emoji that the source draws off-center don't float in the logo row.
fn center_in_frame(img: RgbaImage) -> RgbaImage {
    let mut min_x = u32::MAX;
    let mut min_y = u32::MAX;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut any = false;
    for y in 0..img.height() {
        for x in 0..img.width() {
            if img.get_pixel(x, y).0[3] > 0 {
                any = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if !any {
        return img;
    }
    let cw = max_x - min_x + 1;
    let ch = max_y - min_y + 1;
    let target_x = (img.width() - cw) / 2;
    let target_y = (img.height() - ch) / 2;
    if target_x == min_x && target_y == min_y {
        return img;
    }
    let content = imageops::crop_imm(&img, min_x, min_y, cw, ch).to_image();
    let mut out = RgbaImage::new(img.width(), img.height());
    imageops::overlay(&mut out, &content, target_x as i64, target_y as i64);
    out
}

/// Draw a 1-pixel dark outline in the transparent cells that touch the silhouette.
fn add_outline(img: &RgbaImage) -> RgbaImage {
    let w = img.width();
    let h = img.height();
    let mut out = RgbaImage::new(w + 2, h + 2);
    let ink = Rgba([26u8, 24, 30, 255]);

    // Paint outline into any transparent neighbor (4-connected) of an opaque pixel.
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y).0[3] == 0 {
                continue;
            }
            for (dx, dy) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                let inside = nx >= 0 && ny >= 0 && (nx as u32) < w && (ny as u32) < h;
                let is_edge = !inside || img.get_pixel(nx as u32, ny as u32).0[3] == 0;
                if is_edge {
                    out.put_pixel((nx + 1).max(0) as u32, (ny + 1).max(0) as u32, ink);
                }
            }
        }
    }

    // Draw the sprite on top (offset by the 1px outline margin).
    imageops::overlay(&mut out, img, 1, 1);
    out
}

/// Median-cut palette reduction over the opaque pixels, remapping in place.
fn quantize_palette(img: &mut RgbaImage, max_colors: usize) {
    let mut colors: Vec<[u8; 3]> = img
        .pixels()
        .filter(|p| p.0[3] > 0)
        .map(|p| [p.0[0], p.0[1], p.0[2]])
        .collect();
    if colors.is_empty() {
        return;
    }

    let mut boxes: Vec<Vec<[u8; 3]>> = vec![std::mem::take(&mut colors)];
    while boxes.len() < max_colors {
        // Pick the box with the largest color spread to split.
        let target = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .max_by_key(|(_, b)| channel_spread(b))
            .map(|(i, _)| i);
        let Some(idx) = target else { break };

        let mut b = boxes.swap_remove(idx);
        let channel = widest_channel(&b);
        b.sort_by_key(|c| c[channel]);
        let mid = b.len() / 2;
        let hi = b.split_off(mid);
        boxes.push(b);
        boxes.push(hi);
    }

    // Palette entry = average color of each box.
    let palette: Vec<[u8; 3]> = boxes.iter().map(|b| average(b)).collect();

    for px in img.pixels_mut() {
        if px.0[3] == 0 {
            continue;
        }
        let c = [px.0[0], px.0[1], px.0[2]];
        let best = palette
            .iter()
            .min_by_key(|p| dist2(c, **p))
            .copied()
            .unwrap_or(c);
        px.0[0] = best[0];
        px.0[1] = best[1];
        px.0[2] = best[2];
    }
}

fn channel_spread(colors: &[[u8; 3]]) -> u32 {
    let mut lo = [255u8; 3];
    let mut hi = [0u8; 3];
    for c in colors {
        for k in 0..3 {
            lo[k] = lo[k].min(c[k]);
            hi[k] = hi[k].max(c[k]);
        }
    }
    (0..3).map(|k| (hi[k] - lo[k]) as u32).sum()
}

fn widest_channel(colors: &[[u8; 3]]) -> usize {
    let mut lo = [255u8; 3];
    let mut hi = [0u8; 3];
    for c in colors {
        for k in 0..3 {
            lo[k] = lo[k].min(c[k]);
            hi[k] = hi[k].max(c[k]);
        }
    }
    (0..3).max_by_key(|&k| hi[k] - lo[k]).unwrap_or(0)
}

fn average(colors: &[[u8; 3]]) -> [u8; 3] {
    let n = colors.len().max(1) as u32;
    let mut sum = [0u32; 3];
    for c in colors {
        for k in 0..3 {
            sum[k] += c[k] as u32;
        }
    }
    [
        (sum[0] / n) as u8,
        (sum[1] / n) as u8,
        (sum[2] / n) as u8,
    ]
}

fn dist2(a: [u8; 3], b: [u8; 3]) -> u32 {
    (0..3)
        .map(|k| {
            let d = a[k] as i32 - b[k] as i32;
            (d * d) as u32
        })
        .sum()
}

/// Find bundled SVG data for an emoji sequence.
pub fn bundled_svg(emoji: &str) -> Option<&'static str> {
    let codepoint = emoji_codepoint(emoji);
    bundled_svg_data(&codepoint)
}

/// Render a single emoji: try custom sprite first, then bundled SVG.
pub fn render_emoji(emoji: &str, style: EmojiStyle, sprites_dir: Option<&Path>) -> Option<RgbaImage> {
    if let Some(dir) = sprites_dir {
        if let Some(img) = load_custom_sprite(emoji, dir, style) {
            return Some(img);
        }
    }
    let svg_data = bundled_svg(emoji)?;
    render_emoji_from_svg(svg_data.as_bytes(), style)
}

fn is_regional_indicator(ch: char) -> bool {
    matches!(ch as u32, 0x1F1E6..=0x1F1FF)
}

fn is_sequence_modifier(ch: char) -> bool {
    matches!(
        ch as u32,
        0xFE0E | 0xFE0F | 0x20E3 | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F
    )
}

#[cfg(test)]
mod tests {
    use super::{bundled_svg, emoji_codepoint, split_emojis};

    #[test]
    fn splits_zwj_sequence_as_one_emoji() {
        assert_eq!(split_emojis("👨‍👩‍👧‍👦"), vec!["👨‍👩‍👧‍👦"]);
    }

    #[test]
    fn splits_flag_sequence_as_one_emoji() {
        assert_eq!(split_emojis("🇫🇷"), vec!["🇫🇷"]);
    }

    #[test]
    fn builds_codepoint_names_for_sequences() {
        assert_eq!(emoji_codepoint("🇫🇷"), "1f1eb-1f1f7");
        assert_eq!(
            emoji_codepoint("👨‍👩‍👧‍👦"),
            "1f468-200d-1f469-200d-1f467-200d-1f466"
        );
    }

    #[test]
    fn finds_bundled_svg_for_flag_sequence() {
        assert!(bundled_svg("🇫🇷").is_some());
    }
}
