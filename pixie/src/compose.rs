use image::{imageops, Rgba, RgbaImage};

/// Options for the domino-style badge that houses the emoji.
#[derive(Clone, Copy)]
pub struct BadgeStyle {
    /// Tile fill color.
    pub fill: [u8; 3],
}

/// Compose the emoji into a domino tile (one emoji per cell, split by dividers),
/// then place the tile and text into a horizontal logo.
pub fn compose_domino(
    emojis: &[RgbaImage],
    text: Option<&RgbaImage>,
    padding: u32,
    badge: &BadgeStyle,
) -> RgbaImage {
    let tile = build_domino(emojis, badge);
    let mark = with_shadow(&tile);
    compose_horizontal(&[mark], text, padding)
}

/// Build the domino tile: a rounded 2:1 (or N:1) tile with an emoji centered in
/// each cell and a contrasting divider between cells.
fn build_domino(emojis: &[RgbaImage], badge: &BadgeStyle) -> RgbaImage {
    let emax = emojis
        .iter()
        .flat_map(|e| [e.width(), e.height()])
        .max()
        .unwrap_or(1);
    let cell_pad = (emax as f32 * 0.10).round() as u32;
    let s = emax + cell_pad * 2; // square cell side
    let n = emojis.len().max(1) as u32;
    let (w, h) = (s * n, s);
    let radius = (h as f32 * 0.2).round() as u32;

    // Pick divider/border colors that contrast with the fill.
    let [fr, fg, fb] = badge.fill;
    let lum = 0.299 * fr as f32 + 0.587 * fg as f32 + 0.114 * fb as f32;
    let (divider, border) = if lum > 140.0 {
        ([38u8, 40, 48], scale_rgb(badge.fill, 0.90))
    } else {
        ([232u8, 234, 239], scale_rgb(badge.fill, 1.30))
    };

    // Render the tile background supersampled, then downscale for smooth corners.
    let ss = 4u32;
    let (hw, hh) = (w * ss, h * ss);
    let hr = radius * ss;
    let bw = 3 * ss; // border width
    let mut hi = RgbaImage::new(hw, hh);
    fill_rounded(&mut hi, 0, 0, hw - 1, hh - 1, hr, rgb(border));
    fill_rounded(
        &mut hi,
        bw,
        bw,
        hw - 1 - bw,
        hh - 1 - bw,
        hr.saturating_sub(bw),
        rgb(badge.fill),
    );
    let dvw = 4 * ss;
    let inset = (hh as f32 * 0.12) as u32;
    for i in 1..n {
        let cx = i * s * ss;
        fill_rounded(
            &mut hi,
            cx - dvw / 2,
            inset,
            cx + dvw / 2,
            hh - inset,
            dvw / 2,
            rgb(divider),
        );
    }
    let mut tile = imageops::resize(&hi, w, h, imageops::FilterType::Lanczos3);

    // Composite the (full-resolution, crisp) emoji into each cell.
    for (i, e) in emojis.iter().enumerate() {
        let cx = i as u32 * s + s / 2;
        let ex = cx as i64 - e.width() as i64 / 2;
        let ey = h as i64 / 2 - e.height() as i64 / 2;
        imageops::overlay(&mut tile, e, ex, ey);
    }
    tile
}

/// Wrap a tile in a transparent canvas with a soft drop shadow beneath it.
fn with_shadow(tile: &RgbaImage) -> RgbaImage {
    let pad = (tile.height() as f32 * 0.22).round() as u32;
    let radius = (tile.height() as f32 * 0.2).round() as u32;
    let cw = tile.width() + pad * 2;
    let ch = tile.height() + pad * 2;

    // Shadow silhouette, offset down slightly, then blurred.
    let mut shadow = RgbaImage::new(cw, ch);
    let dy = (tile.height() as f32 * 0.05).round() as u32;
    fill_rounded(
        &mut shadow,
        pad,
        pad + dy,
        pad + tile.width() - 1,
        pad + dy + tile.height() - 1,
        radius,
        Rgba([12, 14, 20, 115]),
    );
    let shadow = imageops::blur(&shadow, pad as f32 * 0.4);

    let mut canvas = RgbaImage::new(cw, ch);
    imageops::overlay(&mut canvas, &shadow, 0, 0);
    imageops::overlay(&mut canvas, tile, pad as i64, pad as i64);
    canvas
}

fn rgb(c: [u8; 3]) -> Rgba<u8> {
    Rgba([c[0], c[1], c[2], 255])
}

fn scale_rgb(c: [u8; 3], f: f32) -> [u8; 3] {
    [
        (c[0] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * f).clamp(0.0, 255.0) as u8,
    ]
}

/// Fill a crisp rounded rectangle. Corner test: distance from the point to the
/// deflated inner rectangle must be within the radius.
fn fill_rounded(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, r: u32, color: Rgba<u8>) {
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let r = r.min((x1 - x0) / 2).min((y1 - y0) / 2) as i64;
    let (ix0, iy0, ix1, iy1) = (x0 as i64 + r, y0 as i64 + r, x1 as i64 - r, y1 as i64 - r);
    for y in y0..=y1.min(img.height() - 1) {
        for x in x0..=x1.min(img.width() - 1) {
            let cx = (x as i64).clamp(ix0, ix1);
            let cy = (y as i64).clamp(iy0, iy1);
            let (dx, dy) = (x as i64 - cx, y as i64 - cy);
            if dx * dx + dy * dy <= r * r {
                img.put_pixel(x, y, color);
            }
        }
    }
}

/// Compose emoji images and text into a single horizontal logo.
/// Layout: [emoji1][padding][emoji2][padding]...[text]
/// All vertically centered.
pub fn compose_horizontal(
    emojis: &[RgbaImage],
    text: Option<&RgbaImage>,
    padding: u32,
) -> RgbaImage {
    // Calculate total dimensions
    let emoji_total_width: u32 = emojis.iter().map(|e| e.width()).sum::<u32>()
        + padding * emojis.len().saturating_sub(1) as u32;

    let text_width = text.map(|t| t.width() + padding).unwrap_or(0);
    let total_width = emoji_total_width + text_width;

    let emoji_max_height = emojis.iter().map(|e| e.height()).max().unwrap_or(0);
    let text_height = text.map(|t| t.height()).unwrap_or(0);
    let total_height = emoji_max_height.max(text_height);

    let mut canvas = RgbaImage::new(total_width.max(1), total_height.max(1));

    // Place emojis
    let mut x = 0u32;
    for emoji_img in emojis {
        let y = (total_height - emoji_img.height()) / 2;
        imageops::overlay(&mut canvas, emoji_img, x as i64, y as i64);
        x += emoji_img.width() + padding;
    }

    // Place text
    if let Some(text_img) = text {
        let y = (total_height - text_img.height()) / 2;
        imageops::overlay(&mut canvas, text_img, x as i64, y as i64);
    }

    canvas
}
