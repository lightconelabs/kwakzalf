use image::{imageops, RgbaImage};
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

/// Render a single emoji into a `size`×`size` square: a custom sprite if one is
/// provided, otherwise the bundled SVG — both rendered crisp (full detail).
pub fn render_emoji(emoji: &str, size: u32, sprites_dir: Option<&Path>) -> Option<RgbaImage> {
    if let Some(dir) = sprites_dir {
        let path = dir.join(format!("{}.png", emoji_codepoint(emoji)));
        if path.exists() {
            let img = image::open(&path).ok()?.into_rgba8();
            return Some(fit_into_box(&img, size));
        }
    }
    let svg = bundled_svg(emoji)?;
    let tree = usvg::Tree::from_data(svg.as_bytes(), &usvg::Options::default()).ok()?;
    Some(render_crisp(&tree, size))
}

/// Render an emoji SVG crisp (full detail, antialiased) into a centered square of
/// `side` px. Supersampled then smoothly downscaled for clean edges.
fn render_crisp(tree: &usvg::Tree, side: u32) -> RgbaImage {
    let ss = 3u32;
    let hi = (side * ss).clamp(1, 1024);
    let mut pixmap = match Pixmap::new(hi, hi) {
        Some(p) => p,
        None => return RgbaImage::new(side, side),
    };
    let size = tree.size();
    let scale = (hi as f32 / size.width()).min(hi as f32 / size.height());
    let tx = (hi as f32 - size.width() * scale) / 2.0;
    let ty = (hi as f32 - size.height() * scale) / 2.0;
    let transform = resvg::tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    let hi_img =
        RgbaImage::from_raw(hi, hi, pixmap.data().to_vec()).unwrap_or_else(|| RgbaImage::new(hi, hi));
    imageops::resize(&hi_img, side, side, imageops::FilterType::Lanczos3)
}

/// Smoothly fit an arbitrary raster into a centered `side`×`side` canvas.
fn fit_into_box(img: &RgbaImage, side: u32) -> RgbaImage {
    let (w, h) = (img.width().max(1), img.height().max(1));
    let scale = (side as f32 / w as f32).min(side as f32 / h as f32);
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    let resized = imageops::resize(img, nw, nh, imageops::FilterType::Lanczos3);
    let mut canvas = RgbaImage::new(side, side);
    imageops::overlay(&mut canvas, &resized, ((side - nw) / 2) as i64, ((side - nh) / 2) as i64);
    canvas
}

/// Find bundled SVG data for an emoji sequence.
pub fn bundled_svg(emoji: &str) -> Option<&'static str> {
    bundled_svg_data(&emoji_codepoint(emoji))
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
    }

    #[test]
    fn finds_bundled_svg_for_flag_sequence() {
        assert!(bundled_svg("🇫🇷").is_some());
    }
}
