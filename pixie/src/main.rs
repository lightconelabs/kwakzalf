use clap::Parser;
use std::path::PathBuf;

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
    println!("Emojis: {}, Text: {:?}, Resolution: {}, Format: {}, Output: {:?}",
        cli.emojis, cli.text, cli.resolution, cli.format, cli.output);
}
