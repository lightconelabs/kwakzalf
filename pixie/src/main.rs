use clap::Parser;
use std::path::PathBuf;

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
    println!("Rendering {} emojis at {}x{}", chars.len(), cli.resolution, cli.resolution);

    for ch in &chars {
        let sprites = cli.sprites_dir.as_deref();
        match emoji::render_emoji(*ch, cli.resolution, sprites) {
            Some(img) => println!("  {} -> {}x{}", ch, img.width(), img.height()),
            None => eprintln!("  {} -> FAILED (missing SVG?)", ch),
        }
    }

    if let Some(ref label) = cli.text {
        let font = text::load_font(cli.font.as_deref());
        let text_img = text::render_text(label, &font, cli.resolution as f32 * 0.5);
        println!("Text '{}' -> {}x{}", label, text_img.width(), text_img.height());
    }
}
