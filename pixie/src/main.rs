use clap::{Parser, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod compose;
mod emoji;
mod text;

use compose::BadgeStyle;
use emoji::EmojiStyle;
use text::TextStyle;

#[derive(Parser)]
#[command(name = "pixie", about = "Emoji-to-pixel-art logo generator")]
struct Cli {
    /// Emoji characters to render
    #[arg(long)]
    emojis: String,

    /// Text to render next to emojis
    #[arg(long)]
    text: Option<String>,

    /// Logical pixels per emoji side (the art resolution — smaller is chunkier)
    #[arg(long, default_value_t = 28, value_parser = parse_grid)]
    grid: u32,

    /// Output pixels per logical pixel (nearest-neighbor zoom)
    #[arg(long, default_value_t = 5, value_parser = parse_zoom)]
    zoom: u32,

    /// Palette size for emoji color reduction (0 keeps source colors)
    #[arg(long, default_value_t = 16)]
    colors: u32,

    /// Pixelate emojis. When false (default), render them crisp (full detail) next to pixel text
    #[arg(long, default_value_t = false, action = clap::ArgAction::Set)]
    emoji_pixelate: bool,

    /// Draw a dark silhouette outline around emojis (pixelated mode only)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    outline: bool,

    /// Badge housing the emoji: none, or a domino tile (one emoji per cell)
    #[arg(long, default_value_t = BadgeMode::None, value_enum)]
    badge: BadgeMode,

    /// Badge tile fill color as hex (default: ivory)
    #[arg(long, default_value = "f9f7f1", value_parser = parse_hex_color)]
    badge_fill: [u8; 3],

    /// Draw a soft drop shadow under the badge
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    badge_shadow: bool,

    /// Text rendering: auto (smooth with crisp emoji, pixel with pixelated), smooth, or pixel
    #[arg(long, default_value_t = TextMode::Auto, value_enum)]
    text_style: TextMode,

    /// Draw a dark outline around pixel text (matches the emoji outline)
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    text_outline: bool,

    /// Text size as a fraction of the emoji box height (default: mode-based)
    #[arg(long)]
    text_scale: Option<f32>,

    /// Extra letter spacing between glyphs (default: mode-based)
    #[arg(long)]
    tracking: Option<i32>,

    /// Output format: png or svg
    #[arg(long, default_value_t = OutputFormat::Png, value_enum)]
    format: OutputFormat,

    /// Custom font path for text
    #[arg(long)]
    font: Option<PathBuf>,

    /// Directory with custom sprite PNGs (named by codepoint, e.g. 1f986.png)
    #[arg(long)]
    sprites_dir: Option<PathBuf>,

    /// Text color as hex (e.g. "ffffff" for white, "f4a030" for yellow)
    #[arg(long, default_value = "ffffff", value_parser = parse_hex_color)]
    color: [u8; 3],

    /// Output file path
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Png,
    Svg,
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum BadgeMode {
    None,
    Domino,
}

impl fmt::Display for BadgeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Domino => write!(f, "domino"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum TextMode {
    /// Smooth with crisp emoji, pixel with pixelated emoji
    Auto,
    /// Antialiased modern font
    Smooth,
    /// Blocky pixel font
    Pixel,
}

impl fmt::Display for TextMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Smooth => write!(f, "smooth"),
            Self::Pixel => write!(f, "pixel"),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Png => write!(f, "png"),
            Self::Svg => write!(f, "svg"),
        }
    }
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let chars = emoji::split_emojis(&cli.emojis);
    if chars.is_empty() {
        return Err("no emoji sequences found in --emojis".to_string());
    }

    let style = EmojiStyle {
        grid: cli.grid,
        zoom: cli.zoom,
        colors: cli.colors,
        outline: cli.outline,
        pixelate: cli.emoji_pixelate,
    };

    // Render emojis
    let emoji_images: Vec<_> = chars
        .iter()
        .filter_map(|ch| {
            let img = emoji::render_emoji(ch, style, cli.sprites_dir.as_deref());
            if img.is_none() {
                eprintln!("Warning: could not render emoji {}", ch);
            }
            img
        })
        .collect();
    if emoji_images.is_empty() {
        return Err("could not render any of the requested emoji sequences".to_string());
    }

    let emoji_box = emoji_images.iter().map(|e| e.height()).max().unwrap_or(0);

    // Text style: smooth modern font by default, pixel font in all-pixel mode.
    let smooth = match cli.text_style {
        TextMode::Auto => !cli.emoji_pixelate,
        TextMode::Smooth => true,
        TextMode::Pixel => false,
    };
    // Sizing and spacing differ between the proportional and pixel fonts.
    let scale = cli.text_scale.unwrap_or(if smooth { 0.72 } else { 0.34 });
    let tracking = cli.tracking.unwrap_or(if smooth { 0 } else { 2 });

    // Render text
    let text_img = cli
        .text
        .as_ref()
        .map(|label| {
            let font = text::load_font(cli.font.as_deref(), smooth)?;
            let text_style = TextStyle {
                pixel_size: (emoji_box as f32 * scale).max(6.0),
                color: cli.color,
                outline: cli.text_outline,
                tracking,
                smooth,
            };
            Ok::<_, String>(text::render_text(label, &font, text_style))
        })
        .transpose()?;

    // Compose. Gap scales with the emoji size so layouts stay balanced.
    let padding = (emoji_box as f32 * 0.28).round() as u32;
    let logo = match cli.badge {
        BadgeMode::None => compose::compose_horizontal(&emoji_images, text_img.as_ref(), padding),
        BadgeMode::Domino => {
            let badge = BadgeStyle {
                fill: cli.badge_fill,
                shadow: cli.badge_shadow,
            };
            compose::compose_domino(&emoji_images, text_img.as_ref(), padding, &badge)
        }
    };

    // Output
    match cli.format {
        OutputFormat::Png => {
            logo.save(&cli.output)
                .map_err(|err| format!("failed to save PNG {}: {err}", cli.output.display()))?;
            println!(
                "Saved PNG: {:?} ({}x{})",
                cli.output,
                logo.width(),
                logo.height()
            );
        }
        OutputFormat::Svg => {
            let svg = png_to_svg(&logo);
            std::fs::write(&cli.output, svg)
                .map_err(|err| format!("failed to save SVG {}: {err}", cli.output.display()))?;
            println!(
                "Saved SVG: {:?} ({}x{})",
                cli.output,
                logo.width(),
                logo.height()
            );
        }
    }

    Ok(())
}

fn parse_grid(raw: &str) -> Result<u32, String> {
    match raw.parse::<u32>() {
        Ok(n) if (8..=128).contains(&n) => Ok(n),
        Ok(_) => Err("grid must be between 8 and 128".to_string()),
        Err(err) => Err(format!("invalid grid: {err}")),
    }
}

fn parse_zoom(raw: &str) -> Result<u32, String> {
    match raw.parse::<u32>() {
        Ok(n) if (1..=32).contains(&n) => Ok(n),
        Ok(_) => Err("zoom must be between 1 and 32".to_string()),
        Err(err) => Err(format!("invalid zoom: {err}")),
    }
}

fn parse_hex_color(hex: &str) -> Result<[u8; 3], String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("color must be a 6-digit hex value like ffffff".to_string());
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| "invalid red channel in color".to_string())?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| "invalid green channel in color".to_string())?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| "invalid blue channel in color".to_string())?;
    Ok([r, g, b])
}

/// Wrap the rendered image in an SVG as a base64-embedded PNG. With crisp emoji
/// and antialiased text, a per-pixel-rect SVG would be many megabytes; a data-URI
/// image is compact, lossless, and scales cleanly.
fn png_to_svg(img: &image::RgbaImage) -> String {
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("failed to encode PNG for SVG");
    let b64 = base64_encode(&png);
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><image width="{w}" height="{h}" image-rendering="auto" href="data:image/png;base64,{b64}"/></svg>"#,
        w = img.width(),
        h = img.height(),
    )
}

/// Minimal standard base64 encoder (avoids pulling in a dependency).
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18 & 63) as usize] as char);
        out.push(TABLE[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{parse_grid, parse_hex_color, parse_zoom};

    #[test]
    fn rejects_short_hex_colors() {
        assert!(parse_hex_color("fff").is_err());
    }

    #[test]
    fn parses_six_digit_hex_colors() {
        assert_eq!(parse_hex_color("#f4a030").unwrap(), [0xf4, 0xa0, 0x30]);
    }

    #[test]
    fn rejects_out_of_range_grid() {
        assert!(parse_grid("4").is_err());
        assert!(parse_grid("200").is_err());
    }

    #[test]
    fn accepts_valid_grid_and_zoom() {
        assert_eq!(parse_grid("28").unwrap(), 28);
        assert_eq!(parse_zoom("5").unwrap(), 5);
    }
}
