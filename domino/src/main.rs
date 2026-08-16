use clap::{Parser, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod compose;
mod emoji;
mod text;

use compose::BadgeStyle;

#[derive(Parser)]
#[command(name = "domino", about = "Emoji logo generator")]
struct Cli {
    /// Emoji characters to render (two work best with the domino badge)
    #[arg(long)]
    emojis: String,

    /// Text to render next to the emojis
    #[arg(long)]
    text: Option<String>,

    /// Emoji box size in pixels
    #[arg(long, default_value_t = 150)]
    size: u32,

    /// House the emoji in a domino tile
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    badge: bool,

    /// Badge tile fill color as hex (divider/border auto-contrast)
    #[arg(long, default_value = "f9f7f1", value_parser = parse_hex_color)]
    badge_fill: [u8; 3],

    /// Text color as hex
    #[arg(long, default_value = "ffffff", value_parser = parse_hex_color)]
    color: [u8; 3],

    /// Text size as a fraction of the emoji box height
    #[arg(long, default_value_t = 0.72)]
    text_scale: f32,

    /// Extra letter spacing between glyphs, in pixels
    #[arg(long, default_value_t = 0)]
    tracking: i32,

    /// Emoji saturation multiplier: 1.0 keeps the artwork as-is, lower mutes it
    #[arg(long, default_value_t = 1.0)]
    emoji_sat: f32,

    /// Drop shadow under the badge
    #[arg(long, default_value_t = Shadow::Soft, value_enum)]
    shadow: Shadow,

    /// Custom font path for text
    #[arg(long)]
    font: Option<PathBuf>,

    /// Directory with custom sprite PNGs (named by codepoint, e.g. 1f986.png)
    #[arg(long)]
    sprites_dir: Option<PathBuf>,

    /// Output format: png or svg
    #[arg(long, default_value_t = OutputFormat::Png, value_enum)]
    format: OutputFormat,

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
enum Shadow {
    /// No shadow — the tile sits flat on the background.
    None,
    /// Tight contact shadow, close under the tile.
    Tight,
    /// Soft ambient shadow.
    Soft,
}

impl fmt::Display for Shadow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Tight => write!(f, "tight"),
            Self::Soft => write!(f, "soft"),
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

    let emoji_images: Vec<_> = chars
        .iter()
        .filter_map(|ch| {
            let img = emoji::render_emoji(ch, cli.size, cli.sprites_dir.as_deref());
            if img.is_none() {
                eprintln!("Warning: could not render emoji {}", ch);
            }
            img
        })
        .map(|img| compose::desaturate(&img, cli.emoji_sat))
        .collect();
    if emoji_images.is_empty() {
        return Err("could not render any of the requested emoji sequences".to_string());
    }

    let text_img = cli
        .text
        .as_ref()
        .map(|label| {
            let font = text::load_font(cli.font.as_deref())?;
            let size = (cli.size as f32 * cli.text_scale).max(6.0);
            Ok::<_, String>(text::render_text(label, &font, size, cli.color, cli.tracking))
        })
        .transpose()?;

    let logo = if cli.badge {
        let badge = BadgeStyle {
            fill: cli.badge_fill,
            shadow: match cli.shadow {
                Shadow::None => compose::ShadowStyle::None,
                Shadow::Tight => compose::ShadowStyle::Tight,
                Shadow::Soft => compose::ShadowStyle::Soft,
            },
        };
        compose::compose_domino(&emoji_images, text_img.as_ref(), &badge)
    } else {
        // Gap between the mark and the text scales with the emoji size.
        let padding = (cli.size as f32 * 0.28).round() as u32;
        compose::compose_horizontal(&emoji_images, text_img.as_ref(), padding)
    };

    match cli.format {
        OutputFormat::Png => {
            logo.save(&cli.output)
                .map_err(|err| format!("failed to save PNG {}: {err}", cli.output.display()))?;
        }
        OutputFormat::Svg => {
            std::fs::write(&cli.output, png_to_svg(&logo))
                .map_err(|err| format!("failed to save SVG {}: {err}", cli.output.display()))?;
        }
    }
    println!("Saved {:?} ({}x{})", cli.output, logo.width(), logo.height());
    Ok(())
}

fn parse_hex_color(hex: &str) -> Result<[u8; 3], String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("color must be a 6-digit hex value like ffffff".to_string());
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16);
    match (channel(0), channel(2), channel(4)) {
        (Ok(r), Ok(g), Ok(b)) => Ok([r, g, b]),
        _ => Err("color has invalid hex digits".to_string()),
    }
}

/// Wrap the rendered image in an SVG as a base64-embedded PNG — compact,
/// lossless, and scalable (a per-pixel-rect SVG would be many megabytes).
fn png_to_svg(img: &image::RgbaImage) -> String {
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .expect("failed to encode PNG for SVG");
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}"><image width="{w}" height="{h}" href="data:image/png;base64,{b64}"/></svg>"#,
        w = img.width(),
        h = img.height(),
        b64 = base64_encode(&png),
    )
}

/// Minimal standard base64 encoder (avoids a dependency).
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | (*chunk.get(2).unwrap_or(&0) as u32);
        out.push(TABLE[(n >> 18 & 63) as usize] as char);
        out.push(TABLE[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[(n >> 6 & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::parse_hex_color;

    #[test]
    fn rejects_short_hex_colors() {
        assert!(parse_hex_color("fff").is_err());
    }

    #[test]
    fn parses_six_digit_hex_colors() {
        assert_eq!(parse_hex_color("#f4a030").unwrap(), [0xf4, 0xa0, 0x30]);
    }
}
