use image::{Rgba, RgbaImage, imageops};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded_emoji.rs"));

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

/// Try to load a custom sprite for an emoji
pub fn load_custom_sprite(emoji: &str, sprites_dir: &Path, resolution: u32) -> Option<RgbaImage> {
    let codepoint = emoji_codepoint(emoji);
    let sprite_path = sprites_dir.join(format!("{}.png", codepoint));
    if sprite_path.exists() {
        let img = image::open(&sprite_path).ok()?.into_rgba8();
        Some(imageops::resize(&img, resolution, resolution, imageops::FilterType::Nearest))
    } else {
        None
    }
}

/// Render an emoji from SVG data, pixelate to target resolution
pub fn render_emoji_from_svg(svg_data: &[u8], resolution: u32) -> Option<RgbaImage> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &options).ok()?;

    // Render at the target resolution directly
    let mut pixmap = Pixmap::new(resolution, resolution)?;

    let size = tree.size();
    let scale_x = resolution as f32 / size.width();
    let scale_y = resolution as f32 / size.height();
    let scale = scale_x.min(scale_y);

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut img = RgbaImage::from_raw(resolution, resolution, pixmap.data().to_vec())?;

    // Snap pixels: threshold alpha to fully opaque or transparent,
    // and quantize colors to remove anti-aliasing blur
    for pixel in img.pixels_mut() {
        if pixel.0[3] < 128 {
            pixel.0 = [0, 0, 0, 0]; // fully transparent
        } else {
            pixel.0[3] = 255; // fully opaque
            // Quantize each color channel to reduce gradients (snap to 8 levels)
            for c in 0..3 {
                pixel.0[c] = (pixel.0[c] / 32) * 32 + 16;
            }
        }
    }

    // Add 1px black outline: expand canvas by 2px, draw black behind opaque pixels
    let w = img.width();
    let h = img.height();
    let mut outlined = RgbaImage::new(w + 2, h + 2);
    let black = Rgba([0, 0, 0, 255]);

    // First pass: draw black in all 8 neighbors of each opaque pixel
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y).0[3] > 0 {
                for dy in 0..=2i32 {
                    for dx in 0..=2i32 {
                        let ox = x as i32 + dx - 1 + 1; // +1 for canvas offset
                        let oy = y as i32 + dy - 1 + 1;
                        if ox >= 0 && oy >= 0 && (ox as u32) < outlined.width() && (oy as u32) < outlined.height() {
                            outlined.put_pixel(ox as u32, oy as u32, black);
                        }
                    }
                }
            }
        }
    }

    // Second pass: draw original pixels on top (offset by 1)
    imageops::overlay(&mut outlined, &img, 1, 1);

    Some(outlined)
}

/// Find bundled SVG data for an emoji sequence.
pub fn bundled_svg(emoji: &str) -> Option<&'static str> {
    let codepoint = emoji_codepoint(emoji);
    bundled_svg_data(&codepoint)
}

/// Render a single emoji: try custom sprite first, then bundled SVG
pub fn render_emoji(emoji: &str, resolution: u32, sprites_dir: Option<&Path>) -> Option<RgbaImage> {
    // Try custom sprite first
    if let Some(dir) = sprites_dir {
        if let Some(img) = load_custom_sprite(emoji, dir, resolution) {
            return Some(img);
        }
    }

    // Fall back to bundled SVG
    let svg_data = bundled_svg(emoji)?;
    render_emoji_from_svg(svg_data.as_bytes(), resolution)
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
        assert_eq!(emoji_codepoint("👨‍👩‍👧‍👦"), "1f468-200d-1f469-200d-1f467-200d-1f466");
    }

    #[test]
    fn finds_bundled_svg_for_flag_sequence() {
        assert!(bundled_svg("🇫🇷").is_some());
    }
}
