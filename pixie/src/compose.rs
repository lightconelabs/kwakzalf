use image::{imageops, RgbaImage};

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
