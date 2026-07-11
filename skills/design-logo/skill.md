---
name: design-logo
description: Design a logo for a DuckEight tool using pixel art emojis and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** A clean, full-detail emoji paired with a smooth modern wordmark
  (Nunito) — polished and contemporary. (In all-pixel mode the wordmark switches
  to a pixel font to match.)
- **Composition:** One or more emojis + tool name text, arranged horizontally (emojis left, text right)
- **Font:** Nunito (bundled, default) for the smooth look; Press Start 2P for
  the pixel look; or supply any font with `--font`
- **Background:** Transparent
- **Color:** Pick a text color that reads on both light and dark backgrounds and
  ties into the emoji palette (e.g. amber `f4c430`, green `7ac74f`). Avoid pure
  white text — it vanishes on light README themes.
- **Pixelated emoji (optional):** `--emoji-pixelate true` renders the emoji as
  pixel art too, for one all-pixel aesthetic. Use it for simple, bold emoji;
  fine-detail ones read better crisp (the default).
- **Emoji artwork:** bundled emoji are Noto Emoji (detailed, shaded — the closest
  freely-licensable match to Apple's style). Apple's own emoji are proprietary
  and can't be bundled; to use genuine Apple artwork on your own machine, export
  the glyphs as PNGs named by codepoint and pass `--sprites-dir` (see
  `pixie/THIRD_PARTY_NOTICES.md`).

## Process

1. **Ask the user** which DuckEight tool needs a logo
2. **Discuss emoji choices** — pick 1-3 emojis that represent the tool. Keep it
   simple.
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
  --emoji-pixelate t|f Pixelate the emoji instead of rendering it crisp (default: false)
  --text-style S       auto | smooth | pixel (default: auto — smooth with crisp emoji)
  --grid N             Logical pixels per pixelated emoji, 8-128 (default: 28; smaller = chunkier)
  --zoom N             Output pixels per logical pixel, 1-32 (default: 5)
  --colors N           Palette size for emoji color reduction, 0 keeps source (default: 16)
  --outline true|false Dark silhouette outline around pixelated emojis (default: true)
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
