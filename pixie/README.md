![pixie](../logos/pixie-logo.png)

# Pixie

Emoji-to-pixel-art logo generator.

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
- `--color FFCC4D` — text color as hex (default: white)
- `--font path.ttf` — custom font override
- `--sprites-dir path/` — custom sprite PNGs by codepoint
- `-o path` — output file

See `pixie --help` for all options.
