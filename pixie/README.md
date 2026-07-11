![pixie](../logos/pixie-logo.png)

# Pixie

Emoji-to-pixel-art logo generator.

Bundled emoji SVG assets and the default pixel font are compiled into the binary, so `cargo install --path .` produces a self-contained executable.

## Install

```bash
cargo install --path .
```

## Usage

```bash
pixie --emojis "🦆🎱" --text "duckeight" --color f4a030 -o logo.png
```

Emojis are rendered as clean, chunky pixel art: the source artwork is
supersampled, downscaled to a small logical grid, reduced to a flat limited
palette (killing gradients), given a thin dark silhouette outline, and scaled up
with nearest-neighbor for crisp square pixels. Text uses a bundled pixel font
with a matching 1-pixel outline.

### Options

- `--emojis` — emoji characters to render as pixel art
- `--text` — text to display next to emojis
- `--grid N` — logical pixels per emoji, 8–128 (default: 28; smaller is chunkier)
- `--zoom N` — output pixels per logical pixel, 1–32 (default: 5)
- `--colors N` — palette size for emoji color reduction, 0 keeps source colors (default: 16)
- `--outline true|false` — dark silhouette outline around emojis (default: true)
- `--text-outline true|false` — dark outline around the text (default: true)
- `--text-scale F` — text cap height as a fraction of the emoji box (default: 0.34)
- `--tracking N` — extra letter spacing in logical pixels (default: 2)
- `--color FFCC4D` — 6-digit text color hex (default: white)
- `--format png|svg` — output format (default: png)
- `--font path.ttf` — custom font override
- `--sprites-dir path/` — custom sprite PNGs by emoji sequence codepoints, e.g. `1f986.png` or `1f1eb-1f1f7.png`
- `-o path` — output file

See `pixie --help` for all options.
