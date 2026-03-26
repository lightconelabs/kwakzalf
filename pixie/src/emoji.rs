use image::{Rgba, RgbaImage, imageops};
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

    // Fallback: relative to Cargo manifest dir (works with `cargo run`)
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("emoji-svg")
        .join(format!("{}.svg", codepoint));
    if manifest_path.exists() {
        return manifest_path;
    }

    // Last resort: relative to cwd
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
