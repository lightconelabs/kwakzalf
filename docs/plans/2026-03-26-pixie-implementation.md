# Pixie Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `pixie`, a Rust CLI that renders emojis as pixel art and composes them with pixel-font text into logos. Ship it as part of the `duckeight-tools` plugin with a `design-logo` agent skill.

**Architecture:** CLI takes emoji chars + text, rasterizes emojis from a bundled color emoji font (Noto Color Emoji), renders text with a bundled pixel font (Press Start 2P), downscales emojis with nearest-neighbor for pixel art look, composites horizontally, outputs PNG or SVG. Custom sprite PNGs can override any emoji.

**Tech Stack:** Rust, clap (CLI), image (PNG I/O + compositing), resvg + tiny-skia (SVG emoji rasterization + SVG output), fontdue (text rendering)

---

### Task 1: Scaffold Rust project

**Files:**
- Create: `pixie/Cargo.toml`
- Create: `pixie/src/main.rs`

**Step 1: Initialize cargo project**

Run:
```bash
cd /Users/beheerder/kwakzalf && cargo init pixie
```

**Step 2: Add dependencies to Cargo.toml**

Edit `pixie/Cargo.toml` to set:

```toml
[package]
name = "pixie"
version = "0.1.0"
edition = "2021"
description = "Emoji-to-pixel-art logo generator"

[dependencies]
clap = { version = "4", features = ["derive"] }
image = "0.25"
resvg = "0.45"
tiny-skia = "0.11"
fontdue = "0.9"
```

**Step 3: Write minimal main with clap args**

Replace `pixie/src/main.rs` with:

```rust
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
```

**Step 4: Verify it compiles**

Run: `cd /Users/beheerder/kwakzalf/pixie && cargo build`
Expected: compiles successfully

**Step 5: Verify help works**

Run: `cargo run -- --help`
Expected: shows help text with all options

**Step 6: Commit**

```bash
git add pixie/
git commit -m "feat: scaffold pixie CLI with clap arg parsing"
```

---

### Task 2: Download and bundle fonts

**Files:**
- Create: `pixie/fonts/PressStart2P-Regular.ttf`
- Create: `pixie/fonts/NotoColorEmoji-Regular.ttf`
- Create: `pixie/build.rs` (optional, for embedding)

**Step 1: Download Press Start 2P**

```bash
mkdir -p /Users/beheerder/kwakzalf/pixie/fonts
curl -L "https://github.com/google/fonts/raw/main/ofl/pressstart2p/PressStart2P-Regular.ttf" \
  -o /Users/beheerder/kwakzalf/pixie/fonts/PressStart2P-Regular.ttf
```

Expected: TTF file downloaded (~30KB)

**Step 2: Download Noto Color Emoji**

We need the SVG version (NotoColorEmoji) since color bitmap fonts are hard to rasterize in Rust. Download the individual SVG emoji files or the full CBDT font. The easiest approach: use Twemoji SVGs instead, which are small and well-organized.

```bash
# Download a few test SVGs from Twemoji for our needed emojis
mkdir -p /Users/beheerder/kwakzalf/pixie/emoji-svg
# Duck emoji (U+1F986)
curl -L "https://raw.githubusercontent.com/jdecked/twemoji/main/assets/svg/1f986.svg" \
  -o /Users/beheerder/kwakzalf/pixie/emoji-svg/1f986.svg
# Pool 8 ball (U+1F3B1)
curl -L "https://raw.githubusercontent.com/jdecked/twemoji/main/assets/svg/1f3b1.svg" \
  -o /Users/beheerder/kwakzalf/pixie/emoji-svg/1f3b1.svg
```

Expected: SVG files downloaded

**Step 3: Commit**

```bash
git add pixie/fonts/ pixie/emoji-svg/
git commit -m "feat: bundle Press Start 2P font and Twemoji SVGs"
```

---

### Task 3: Implement emoji SVG rasterization and pixelation

**Files:**
- Create: `pixie/src/emoji.rs`
- Modify: `pixie/src/main.rs`

**Step 1: Create emoji rendering module**

Create `pixie/src/emoji.rs`:

```rust
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
```

**Step 2: Wire into main.rs**

Update `pixie/src/main.rs` to add `mod emoji;` and test rendering:

```rust
use clap::Parser;
use std::path::PathBuf;

mod emoji;

#[derive(Parser)]
#[command(name = "pixie", about = "Emoji-to-pixel-art logo generator")]
struct Cli {
    #[arg(long)]
    emojis: String,

    #[arg(long)]
    text: Option<String>,

    #[arg(long, default_value = "32")]
    resolution: u32,

    #[arg(long, default_value = "png")]
    format: String,

    #[arg(long)]
    font: Option<PathBuf>,

    #[arg(long)]
    sprites_dir: Option<PathBuf>,

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
}
```

**Step 3: Verify it compiles and renders**

Run:
```bash
cd /Users/beheerder/kwakzalf/pixie && cargo run -- --emojis "🦆🎱" -o test.png
```
Expected: prints rendered dimensions for each emoji

**Step 4: Commit**

```bash
git add pixie/src/
git commit -m "feat: emoji SVG rasterization with pixelation"
```

---

### Task 4: Implement text rendering with pixel font

**Files:**
- Create: `pixie/src/text.rs`
- Modify: `pixie/src/main.rs`

**Step 1: Create text rendering module**

Create `pixie/src/text.rs`:

```rust
use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};
use std::path::Path;

const DEFAULT_FONT: &[u8] = include_bytes!("../fonts/PressStart2P-Regular.ttf");

/// Load font from path or use bundled default
pub fn load_font(custom_path: Option<&Path>) -> Font {
    match custom_path {
        Some(path) => {
            let data = std::fs::read(path).expect("Failed to read font file");
            Font::from_bytes(data, FontSettings::default()).expect("Failed to parse font")
        }
        None => Font::from_bytes(DEFAULT_FONT, FontSettings::default())
            .expect("Failed to parse bundled font"),
    }
}

/// Render text to an RgbaImage with the given font at pixel_size
pub fn render_text(text: &str, font: &Font, pixel_size: f32) -> RgbaImage {
    let color = Rgba([255, 255, 255, 255]); // white text

    // Measure total width and max height
    let mut total_width = 0u32;
    let mut max_height = 0u32;
    let mut glyphs = Vec::new();

    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, pixel_size);
        total_width += metrics.advance_width.ceil() as u32;
        max_height = max_height.max(metrics.height as u32);
        glyphs.push((metrics, bitmap));
    }

    let baseline = pixel_size as u32;
    let img_height = baseline + 4; // some padding
    let mut img = RgbaImage::new(total_width, img_height);

    let mut x_offset = 0i32;
    for (metrics, bitmap) in &glyphs {
        let y_start = (baseline as i32 - metrics.height as i32 - metrics.ymin) as i32;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha > 0 {
                    let px = x_offset + col as i32;
                    let py = y_start + row as i32;
                    if px >= 0 && py >= 0 && (px as u32) < img.width() && (py as u32) < img.height() {
                        img.put_pixel(px as u32, py as u32, Rgba([color.0[0], color.0[1], color.0[2], alpha]));
                    }
                }
            }
        }
        x_offset += metrics.advance_width.ceil() as i32;
    }

    img
}
```

**Step 2: Wire into main.rs to test text rendering**

Add `mod text;` and test:

```rust
mod text;

// ... in main(), after emoji rendering:
if let Some(ref label) = cli.text {
    let font = text::load_font(cli.font.as_deref());
    let text_img = text::render_text(label, &font, cli.resolution as f32 * 0.5);
    println!("Text '{}' -> {}x{}", label, text_img.width(), text_img.height());
}
```

**Step 3: Verify it compiles and renders text**

Run:
```bash
cd /Users/beheerder/kwakzalf/pixie && cargo run -- --emojis "🦆" --text "test" -o test.png
```
Expected: prints text dimensions

**Step 4: Commit**

```bash
git add pixie/src/text.rs pixie/src/main.rs
git commit -m "feat: text rendering with bundled Press Start 2P pixel font"
```

---

### Task 5: Implement composition and PNG output

**Files:**
- Create: `pixie/src/compose.rs`
- Modify: `pixie/src/main.rs`

**Step 1: Create composition module**

Create `pixie/src/compose.rs`:

```rust
use image::{RgbaImage, imageops};

/// Compose emoji images and text into a single horizontal logo.
/// Layout: [emoji1][padding][emoji2][padding]...[text]
/// All vertically centered.
pub fn compose_horizontal(emojis: &[RgbaImage], text: Option<&RgbaImage>, padding: u32) -> RgbaImage {
    // Calculate total dimensions
    let emoji_total_width: u32 = emojis.iter().map(|e| e.width()).sum::<u32>()
        + padding * emojis.len().saturating_sub(1) as u32;

    let text_width = text.map(|t| t.width() + padding).unwrap_or(0);
    let total_width = emoji_total_width + text_width;

    let emoji_max_height = emojis.iter().map(|e| e.height()).max().unwrap_or(0);
    let text_height = text.map(|t| t.height()).unwrap_or(0);
    let total_height = emoji_max_height.max(text_height);

    let mut canvas = RgbaImage::new(total_width, total_height);

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
```

**Step 2: Wire everything together in main.rs**

Replace `pixie/src/main.rs`:

```rust
use clap::Parser;
use std::path::PathBuf;

mod compose;
mod emoji;
mod text;

#[derive(Parser)]
#[command(name = "pixie", about = "Emoji-to-pixel-art logo generator")]
struct Cli {
    #[arg(long)]
    emojis: String,

    #[arg(long)]
    text: Option<String>,

    #[arg(long, default_value = "32")]
    resolution: u32,

    #[arg(long, default_value = "png")]
    format: String,

    #[arg(long)]
    font: Option<PathBuf>,

    #[arg(long)]
    sprites_dir: Option<PathBuf>,

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
            println!("Saved PNG: {:?} ({}x{})", cli.output, logo.width(), logo.height());
        }
        "svg" => {
            let svg = png_to_svg(&logo);
            std::fs::write(&cli.output, svg).expect("Failed to save SVG");
            println!("Saved SVG: {:?} ({}x{})", cli.output, logo.width(), logo.height());
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
```

**Step 3: Build and test end-to-end**

Run:
```bash
cd /Users/beheerder/kwakzalf/pixie && cargo run -- --emojis "🦆🎱" --text "duckeight" --resolution 64 -o test-logo.png
```
Expected: saves a PNG file

**Step 4: Commit**

```bash
git add pixie/src/
git commit -m "feat: horizontal composition and PNG/SVG output"
```

---

### Task 6: Generate DuckEight logo and create logos directory

**Files:**
- Create: `logos/duckeight-logo.png`
- Create: `logos/duckeight-logo.svg`

**Step 1: Create logos directory**

```bash
mkdir -p /Users/beheerder/kwakzalf/logos
```

**Step 2: Generate PNG logo**

```bash
cd /Users/beheerder/kwakzalf/pixie && cargo run -- \
  --emojis "🦆🎱" --text "duckeight" --resolution 64 \
  -o ../logos/duckeight-logo.png
```

**Step 3: Generate SVG logo**

```bash
cd /Users/beheerder/kwakzalf/pixie && cargo run -- \
  --emojis "🦆🎱" --text "duckeight" --resolution 64 \
  --format svg -o ../logos/duckeight-logo.svg
```

**Step 4: Commit**

```bash
git add logos/
git commit -m "feat: generate duckeight logo with pixie"
```

---

### Task 7: Create plugin manifest and design-logo skill

**Files:**
- Create: `plugin.json`
- Create: `skills/design-logo/skill.md`

**Step 1: Create plugin.json**

Create `/Users/beheerder/kwakzalf/plugin.json`:

```json
{
  "name": "duckeight-tools",
  "description": "Agent skills for DuckEight tools — logo design with pixel art emojis",
  "skills_dir": "./skills"
}
```

**Step 2: Create the design-logo skill**

Create `/Users/beheerder/kwakzalf/skills/design-logo/skill.md`:

```markdown
---
name: design-logo
description: Design a logo for a DuckEight tool using pixel art emojis and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** Simple pixel art
- **Composition:** One or more emojis (as pixel art) + tool name text, arranged horizontally (emojis left, text right)
- **Font:** Press Start 2P pixel font (bundled with pixie) or another pixel/mono font
- **Background:** Transparent
- **Resolution:** 64x64 per emoji recommended

## Process

1. **Ask the user** which DuckEight tool needs a logo
2. **Discuss emoji choices** — pick 1-3 emojis that represent the tool. Keep it simple
3. **Confirm the tool name** text to display
4. **Generate the logo** using the `pixie` CLI:

```bash
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --resolution 64 -o logos/<tool-name>-logo.png
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --resolution 64 --format svg -o logos/<tool-name>-logo.svg
```

5. **Show the result** to the user (read the PNG file to display it)
6. **Iterate** if needed — adjust emojis, resolution, or font

## Pixie CLI Reference

```
pixie --emojis "🦆🎱" --text "duckeight" [OPTIONS] -o output.png

Options:
  --resolution 32|64    Pixel grid per emoji (default: 32)
  --format png|svg      Output format (default: png)
  --font path.ttf       Custom font override
  --sprites-dir path/   Custom sprite PNGs (named by codepoint)
```

## Examples

DuckEight logo:
```bash
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logos/duckeight-logo.png
```
```

**Step 3: Commit**

```bash
git add plugin.json skills/
git commit -m "feat: add duckeight-tools plugin manifest and design-logo skill"
```

---

### Task 8: Add README

**Files:**
- Create: `README.md`

**Step 1: Create README**

```markdown
# duckeight-tools

Agent skills plugin for Claude Code, Codex, and OpenCode.

## Skills

### design-logo

Design logos for DuckEight tools using pixel art emojis and text.

## Pixie CLI

Emoji-to-pixel-art logo generator.

### Install

```bash
cd pixie && cargo install --path .
```

### Usage

```bash
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logo.png
```

See `pixie --help` for all options.

## Plugin Installation

Add to your Claude Code settings:

```json
{
  "plugins": ["github:duckeight/duckeight-tools"]
}
```
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README"
```
