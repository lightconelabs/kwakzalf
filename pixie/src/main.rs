use clap::{Parser, ValueEnum};
use std::fmt;
use std::path::PathBuf;

mod compose;
mod emoji;
mod text;

#[derive(Parser)]
#[command(name = "pixie", about = "Emoji-to-pixel-art logo generator")]
struct Cli {
    /// Emoji characters to render
    #[arg(long)]
    emojis: String,

    /// Text to render next to emojis
    #[arg(long)]
    text: Option<String>,

    /// Pixel grid resolution per emoji (32 or 64)
    #[arg(long, default_value_t = 32, value_parser = parse_resolution)]
    resolution: u32,

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

    // Render emojis
    let emoji_images: Vec<_> = chars
        .iter()
        .filter_map(|ch| {
            let img = emoji::render_emoji(ch, cli.resolution, cli.sprites_dir.as_deref());
            if img.is_none() {
                eprintln!("Warning: could not render emoji {}", ch);
            }
            img
        })
        .collect();
    if emoji_images.is_empty() {
        return Err("could not render any of the requested emoji sequences".to_string());
    }

    // Render text
    let text_img = cli
        .text
        .as_ref()
        .map(|label| {
            let font = text::load_font(cli.font.as_deref())?;
            let font_size = cli.resolution as f32 * 0.5;
            Ok::<_, String>(text::render_text(label, &font, font_size, cli.color))
        })
        .transpose()?;

    // Compose
    let padding = cli.resolution / 4;
    let logo = compose::compose_horizontal(&emoji_images, text_img.as_ref(), padding);

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

fn parse_resolution(raw: &str) -> Result<u32, String> {
    match raw.parse::<u32>() {
        Ok(32 | 64) => raw.parse::<u32>().map_err(|err| err.to_string()),
        Ok(_) => Err("resolution must be 32 or 64".to_string()),
        Err(err) => Err(format!("invalid resolution: {err}")),
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

/// Convert an RGBA image to SVG by drawing each non-transparent pixel as a rect
fn png_to_svg(img: &image::RgbaImage) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" shape-rendering="crispEdges">"#,
        img.width(),
        img.height()
    );
    svg.push('\n');

    for y in 0..img.height() {
        for x in 0..img.width() {
            let px = img.get_pixel(x, y);
            if px.0[3] > 0 {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="1" height="1" fill="rgba({},{},{},{:.2})"/>"#,
                    x,
                    y,
                    px.0[0],
                    px.0[1],
                    px.0[2],
                    px.0[3] as f32 / 255.0
                ));
                svg.push('\n');
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::{parse_hex_color, parse_resolution};

    #[test]
    fn rejects_short_hex_colors() {
        assert!(parse_hex_color("fff").is_err());
    }

    #[test]
    fn parses_six_digit_hex_colors() {
        assert_eq!(parse_hex_color("#f4a030").unwrap(), [0xf4, 0xa0, 0x30]);
    }

    #[test]
    fn rejects_unsupported_resolutions() {
        assert!(parse_resolution("1").is_err());
    }

    #[test]
    fn accepts_supported_resolutions() {
        assert_eq!(parse_resolution("64").unwrap(), 64);
    }
}
