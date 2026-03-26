# Pixie — Emoji-to-Pixel-Art Logo Generator

## Overview

Pixie is a Rust CLI that composes logos from pixel-art emojis and text. It ships as part of the `duckeight-tools` plugin, which provides agent skills for Claude Code, Codex, and OpenCode.

## Pixie CLI

```bash
pixie --emojis "🦆🎱" --text "duckeight" -o logo.png
```

Rasterizes emoji characters from a bundled emoji font (Noto Color Emoji) at the target resolution, downscales with nearest-neighbor interpolation, and snaps to a pixel grid. Text is rendered with a bundled pixel font (Press Start 2P).

### Options

- `--emojis` — one or more emoji characters to render
- `--text` — tool name text to render
- `--resolution 32|64` — pixel grid size per emoji (default 32)
- `--format png|svg` — output format (default png)
- `--font path/to/font.ttf` — override the text font
- `--sprites-dir path/` — custom PNG sprites override font-rendered emojis (matched by codepoint filename, e.g. `1f986.png` for 🦆)
- `-o path` — output file

### Pixelation approach

1. Render emoji at high resolution from font
2. Downscale to target grid (32x32 or 64x64) using nearest-neighbor
3. Optional color palette reduction (16-32 colors) for authentic pixel art

### Dependencies

- `image` — rasterization and PNG output
- `fontdue` or `ab_glyph` — font rendering
- `resvg` / `tiny-skia` — SVG output
- `clap` — argument parsing

## Plugin Structure

```
kwakzalf/
├── pixie/
│   ├── Cargo.toml
│   ├── src/main.rs
│   └── fonts/          # Press Start 2P + Noto Color Emoji
├── skills/
│   └── design-logo/
│       └── skill.md
├── plugin.json
├── logos/               # Generated example logos
└── README.md
```

### plugin.json

Registers the `duckeight-tools` plugin and its skills for Claude Code, Codex, and OpenCode.

### design-logo skill

Guides an agent through:
1. Asking which DuckEight tool needs a logo
2. Picking representative emojis
3. Running `pixie` to generate the logo
4. Presenting the result and iterating

Enforces DuckEight brand constraints:
- Simple pixel art style
- One or more emojis + tool name
- Pixel/mono font for text
- Horizontal layout (emojis left, text right)
- Transparent background

## First Logo — DuckEight

```bash
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logos/duckeight-logo.png
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 --format svg -o logos/duckeight-logo.svg
```

Horizontal composition: 🦆 (64x64 pixel art) + 🎱 (64x64 pixel art) + "duckeight" in Press Start 2P, vertically centered.
