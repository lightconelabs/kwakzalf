use clap::Parser;
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
    #[arg(long, default_value = "32")]
    resolution: u32,

    /// Output format: png or svg
    #[arg(long, default_value = "png")]
    format: String,

    /// Custom font path for text
    #[arg(long)]
    font: Option<PathBuf>,

    /// Directory with custom sprite PNGs (named by codepoint, e.g. 1f986.png)
    #[arg(long)]
    sprites_dir: Option<PathBuf>,

    /// Output file path
    #[arg(short, long)]
    output: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let chars = emoji::split_emojis(&cli.emojis);

    // Render emojis
    let emoji_images: Vec<_> = chars
        .iter()
        .filter_map(|ch| {
            let img = emoji::render_emoji(*ch, cli.resolution, cli.sprites_dir.as_deref());
            if img.is_none() {
                eprintln!("Warning: could not render emoji {}", ch);
            }
            img
        })
        .collect();

    // Render text
    let text_img = cli.text.as_ref().map(|label| {
        let font = text::load_font(cli.font.as_deref());
        let font_size = cli.resolution as f32 * 0.5;
        text::render_text(label, &font, font_size)
    });

    // Compose
    let padding = cli.resolution / 4;
    let logo = compose::compose_horizontal(&emoji_images, text_img.as_ref(), padding);

    // Output
    match cli.format.as_str() {
        "png" => {
            logo.save(&cli.output).expect("Failed to save PNG");
            println!(
                "Saved PNG: {:?} ({}x{})",
                cli.output,
                logo.width(),
                logo.height()
            );
        }
        "svg" => {
            let svg = png_to_svg(&logo);
            std::fs::write(&cli.output, svg).expect("Failed to save SVG");
            println!(
                "Saved SVG: {:?} ({}x{})",
                cli.output,
                logo.width(),
                logo.height()
            );
        }
        _ => eprintln!("Unknown format: {}", cli.format),
    }
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
