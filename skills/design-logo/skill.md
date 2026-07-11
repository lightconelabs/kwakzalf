---
name: design-logo
description: Design a logo for a DuckEight tool using pixel art emojis and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** Clean, chunky pixel art — flat limited-color palettes, a thin dark
  outline on both emoji and text, no gradients or blur
- **Composition:** One or more emojis (as pixel art) + tool name text, arranged horizontally (emojis left, text right)
- **Font:** Press Start 2P pixel font (bundled with pixie) or another pixel/mono font
- **Background:** Transparent
- **Grid:** ~28 logical pixels per emoji at zoom 5 (the defaults) — chunky enough
  to read as pixel art, detailed enough for most emoji
- **Color:** Pick a text color that reads on both light and dark backgrounds and
  ties into the emoji palette (e.g. amber `f4c430`, green `7ac74f`). Avoid pure
  white text — it vanishes on light README themes.

## Process

1. **Ask the user** which DuckEight tool needs a logo
2. **Discuss emoji choices** — pick 1-3 emojis that represent the tool. Keep it
   simple. Emojis with fine detail (e.g. a musical score) reduce poorly to the
   pixel grid; prefer bold, simple shapes.
3. **Confirm the tool name** text to display
4. **Generate the logo** using the `pixie` CLI:

```bash
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --color <hex> -o logos/<tool-name>-logo.png
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --color <hex> --format svg -o logos/<tool-name>-logo.svg
```

5. **Inspect the result** — read the PNG to view it. Check it on both a light and
   a dark background (composite it over white and over dark gray) so the outline
   and text color read on both. This visual feedback loop is the whole point:
   look at the pixels, then adjust.
6. **Iterate** if needed — swap emojis, change `--color`, or tune `--grid` /
   `--zoom` / `--colors`.

## Prerequisites

The `pixie` CLI must be installed. Build it from the plugin source:

```bash
cd ${CLAUDE_PLUGIN_ROOT}/pixie && cargo build --release
```

The binary will be at `${CLAUDE_PLUGIN_ROOT}/pixie/target/release/pixie`.

## Pixie CLI Reference

```
pixie --emojis "🦆🎱" --text "duckeight" [OPTIONS] -o output.png

Options:
  --grid N             Logical pixels per emoji, 8-128 (default: 28; smaller = chunkier)
  --zoom N             Output pixels per logical pixel, 1-32 (default: 5)
  --colors N           Palette size for emoji color reduction, 0 keeps source (default: 16)
  --outline true|false Dark silhouette outline around emojis (default: true)
  --text-outline t|f   Dark outline around the text (default: true)
  --text-scale F       Text cap height as a fraction of the emoji box (default: 0.34)
  --tracking N         Extra letter spacing in logical pixels (default: 2)
  --color HEX          Text color, 6-digit hex (default: ffffff)
  --format png|svg     Output format (default: png)
  --font path.ttf      Custom font override
  --sprites-dir path/  Custom sprite PNGs (named by emoji sequence codepoints)
```

## Examples

DuckEight logo:
```bash
pixie --emojis "🦆🎱" --text "duckeight" --color f4a030 -o logos/duckeight-logo.png
```
