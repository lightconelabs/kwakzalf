![pixie](../logos/pixie-logo.png)

# Pixie

Emoji-to-pixel-art logo generator.

Bundled emoji SVG assets and the default pixel font are compiled into the binary, so `cargo install --path .` produces a self-contained executable.

Emoji artwork is [Noto Emoji](https://github.com/googlefonts/noto-emoji) (SIL OFL 1.1),
with a few Twemoji fallbacks (CC-BY 4.0). Apple's emoji are proprietary and not
bundled — to use genuine Apple artwork locally, export it as PNG sprites and pass
`--sprites-dir` (see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)).

## Install

```bash
cargo install --path .
```

## Usage

```bash
pixie --emojis "🦆🎱" --text "duckeight" --color f4a030 -o logo.png
```

By default the emoji is rendered **crisp** — full-detail, smoothly antialiased —
so a clean vector icon sits next to a chunky pixel wordmark. The text is drawn
with a bundled pixel font and a thin dark outline for a cohesive sticker look.

Pass `--emoji-pixelate true` to pixelate the emoji too (one all-pixel-art
aesthetic): the source artwork is supersampled, downscaled to a small logical
grid, reduced to a flat limited palette (killing gradients), given a thin dark
silhouette outline, and scaled up with nearest-neighbor for crisp square pixels.
Simple, bold emoji pixelate well; fine-detail ones (e.g. a musical score) read
better crisp.

### Options

- `--emojis` — emoji characters to render
- `--text` — text to display next to emojis
- `--emoji-pixelate true|false` — pixelate the emoji instead of rendering it crisp (default: false)
- `--grid N` — logical pixels per emoji when pixelated, 8–128 (default: 28; smaller is chunkier)
- `--zoom N` — output pixels per logical pixel, 1–32 (default: 5)
- `--colors N` — palette size for emoji color reduction, 0 keeps source colors (default: 16)
- `--outline true|false` — dark silhouette outline around pixelated emojis (default: true)
- `--text-outline true|false` — dark outline around the text (default: true)
- `--text-scale F` — text cap height as a fraction of the emoji box (default: 0.34)
- `--tracking N` — extra letter spacing in logical pixels (default: 2)
- `--color FFCC4D` — 6-digit text color hex (default: white)
- `--format png|svg` — output format (default: png)
- `--font path.ttf` — custom font override
- `--sprites-dir path/` — custom sprite PNGs by emoji sequence codepoints, e.g. `1f986.png` or `1f1eb-1f1f7.png`
- `-o path` — output file

See `pixie --help` for all options.
