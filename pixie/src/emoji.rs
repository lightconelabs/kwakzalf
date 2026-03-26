use image::{RgbaImage, imageops};
use resvg::tiny_skia::Pixmap;
use resvg::usvg;
use std::path::{Path, PathBuf};

/// Get the codepoint hex string for an emoji character
pub fn emoji_codepoint(ch: char) -> String {
    format!("{:x}", ch as u32)
}

/// Split an emoji string into individual emoji characters
/// Handles multi-codepoint emojis by treating each char individually for now
pub fn split_emojis(emojis: &str) -> Vec<char> {
    emojis.chars().filter(|c| !c.is_ascii()).collect()
}

/// Try to load a custom sprite for an emoji
pub fn load_custom_sprite(ch: char, sprites_dir: &Path, resolution: u32) -> Option<RgbaImage> {
    let codepoint = emoji_codepoint(ch);
    let sprite_path = sprites_dir.join(format!("{}.png", codepoint));
    if sprite_path.exists() {
        let img = image::open(&sprite_path).ok()?.into_rgba8();
        Some(imageops::resize(&img, resolution, resolution, imageops::FilterType::Nearest))
    } else {
        None
    }
}

/// Render an emoji from its SVG file, pixelate to target resolution
pub fn render_emoji_from_svg(svg_path: &Path, resolution: u32) -> Option<RgbaImage> {
    let svg_data = std::fs::read(svg_path).ok()?;
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &options).ok()?;

    // Render at high resolution first (4x target for quality)
    let high_res = resolution * 4;
    let mut pixmap = Pixmap::new(high_res, high_res)?;

    let size = tree.size();
    let scale_x = high_res as f32 / size.width();
    let scale_y = high_res as f32 / size.height();
    let scale = scale_x.min(scale_y);

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert to image::RgbaImage
    let high_res_img = RgbaImage::from_raw(high_res, high_res, pixmap.data().to_vec())?;

    // Downscale with nearest-neighbor for pixel art look
    Some(imageops::resize(&high_res_img, resolution, resolution, imageops::FilterType::Nearest))
}

/// Find the bundled SVG for an emoji character
pub fn bundled_svg_path(ch: char) -> PathBuf {
    let codepoint = emoji_codepoint(ch);
    // Look relative to the executable, then fall back to compile-time path
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        let path = dir.join("emoji-svg").join(format!("{}.svg", codepoint));
        if path.exists() {
            return path;
        }
    }

    // Fallback: relative to cwd
    PathBuf::from(format!("emoji-svg/{}.svg", codepoint))
}

/// Render a single emoji: try custom sprite first, then bundled SVG
pub fn render_emoji(ch: char, resolution: u32, sprites_dir: Option<&Path>) -> Option<RgbaImage> {
    // Try custom sprite first
    if let Some(dir) = sprites_dir {
        if let Some(img) = load_custom_sprite(ch, dir, resolution) {
            return Some(img);
        }
    }

    // Fall back to bundled SVG
    let svg_path = bundled_svg_path(ch);
    render_emoji_from_svg(&svg_path, resolution)
}
