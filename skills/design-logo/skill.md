---
name: design-logo
description: Design a logo for a DuckEight tool using emoji in a domino badge and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** Two full-detail emoji housed in a **domino tile** (`--badge true`, the default)
  — a soft rounded plate split into two cells by a divider — paired with a smooth
  modern wordmark (Nunito). Polished, self-contained, works on any background.
- **Composition:** Two emojis in a domino badge + tool name text to the right.
  Pick exactly two emoji so each fills one cell of the domino.
- **Font:** Nunito (bundled, default), or supply any font with `--font`
- **Background:** Transparent, with an even margin of one shadow-width on all
  four sides
- **Spacing:** The badge-to-wordmark gap is fixed at the wordmark's x-height
  (equivalently, the width of its `o`) — the usual icon-to-wordmark measure for
  a horizontal lockup, and the same rule Google's Android brand uses. It is
  computed, not a knob, so every house logo lines up.
- **Color:** Pick a text color that reads on both light and dark backgrounds and
  ties into the emoji palette (e.g. amber `f4c430`, green `7ac74f`). Avoid pure
  white text — it vanishes on light README themes.
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
pixie --emojis "<two-emojis>" --text "<tool-name>" --color <hex> -o logos/<tool-name>-logo.png
pixie --emojis "<two-emojis>" --text "<tool-name>" --color <hex> --format svg -o logos/<tool-name>-logo.svg
```

5. **Inspect the result** — read the PNG to view it. Check it on both a light and
   a dark background (composite it over white and over dark gray) so the outline
   and text color read on both. This visual feedback loop is the whole point:
   look at the pixels, then adjust.
6. **Iterate** if needed — swap emojis, change `--color`, or tune `--badge-fill`
   / `--text-scale`.

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
  --size N             Emoji box size in pixels (default: 150)
  --badge true|false   House the emoji in a domino tile (default: true)
  --badge-fill HEX     Tile fill color (default: ivory f9f7f1); divider auto-contrasts
  --color HEX          Text color, 6-digit hex (default: ffffff)
  --text-scale F       Text size as a fraction of the emoji box (default: 0.72)
  --tracking N         Extra letter spacing in pixels (default: 0)
  --font path.ttf      Custom font override
  --sprites-dir path/  Custom sprite PNGs (named by emoji codepoints)
  --format png|svg     Output format (default: png)
```

## Examples

DuckEight logo:
```bash
pixie --emojis "🦆🎱" --text "duckeight" --color f4a030 -o logos/duckeight-logo.png
```

Teleprompt logo:
```bash
pixie --emojis "🎬🎙" --text "teleprompt" --color 3a86c8 -o logos/teleprompt-logo.png
```

## Gotchas

- **Strip variation selectors.** Sprites are named by base codepoint, so an
  emoji carrying U+FE0F (🗣️, 🎙️, 🎞️) fails to resolve with "could not render
  emoji". Pass the bare glyph (🗣, 🎙, 🎞) instead.
