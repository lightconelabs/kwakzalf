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
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logo.png
```

### Options

- `--emojis` — emoji characters to render as pixel art
- `--text` — text to display next to emojis
- `--resolution 32|64` — pixel grid per emoji (default: 32)
- `--format png|svg` — output format (default: png)
- `--color FFCC4D` — 6-digit text color hex (default: white)
- `--font path.ttf` — custom font override
- `--sprites-dir path/` — custom sprite PNGs by emoji sequence codepoints, e.g. `1f986.png` or `1f1eb-1f1f7.png`
- `-o path` — output file

See `pixie --help` for all options.
