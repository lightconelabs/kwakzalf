---
name: design-logo
description: Design a logo for a DuckEight tool using emoji in a domino badge and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** Two full-detail emoji housed in a **domino tile** (`--badge true`, the default)
  — a soft rounded plate split into two cells by a divider — paired with a smooth
  modern wordmark (IBM Plex Sans). Polished, self-contained, works on any background.
- **Composition:** Two emojis in a domino badge + tool name text to the right.
  Pick exactly two emoji so each fills one cell of the domino.
- **Font:** IBM Plex Sans SemiBold (bundled, default), or supply any font with
  `--font`. Flat terminals and an engineered feel — avoid rounded-terminal faces
  (Nunito, Quicksand, Baloo) and very heavy weights, which read as children's
  media rather than developer tooling. Note `--font` renders a variable font at
  its default instance, so pass a static weight.
- **Background:** Transparent, with an even margin of one shadow-width on all
  four sides
- **Spacing:** The badge-to-wordmark gap is computed as 0.31x the tile height
  (56px at the default `--size`), which sits between the wordmark's `o` width
  and its x-height — the usual icon-to-wordmark measure for a horizontal
  lockup, and the same rule Google's Android brand uses. It is computed, not a
  knob, so every house logo lines up.
- **Color:** Pick a text color that ties into the emoji palette and reads on both
  light and dark backgrounds. Avoid pure white — it vanishes on light README
  themes. Keep it **muted**: saturation at or below ~0.6. Straight-from-the-emoji
  hues are bright and fully saturated, and five of those side by side read as a
  crayon box rather than a tool suite. Take the emoji's hue, then drop saturation
  and settle the lightness until it clears roughly 3:1 on white *and* 3.5:1 on
  dark — the old palette hit 10:1 on dark but 1.6:1 on white, which is why it
  looked flat and shouty at once.
- **Palette:** No two tools share a wordmark color. Desaturating compresses the
  palette, so near-matches collide easily — check a new pick against the taken
  ones and keep CIELAB ΔE above ~25 (the set's closest pair is currently 32):

  | tool | color | | sat |
  | --- | --- | --- | --- |
  | duckeight | `c2833a` | muted amber | 0.70 |
  | kwakzalf | `3d8b84` | muted teal | 0.56 |
  | primavera | `6b9550` | muted green | 0.46 |
  | teleprompt | `4a7ea8` | muted steel blue | 0.56 |
  | domino | `a85fa0` | muted orchid | 0.43 |
- **Emoji artwork:** bundled emoji are Noto Emoji (detailed, shaded — the closest
  freely-licensable match to Apple's style). Apple's own emoji are proprietary
  and can't be bundled; to use genuine Apple artwork on your own machine, export
  the glyphs as PNGs named by codepoint and pass `--sprites-dir` (see
  `domino/THIRD_PARTY_NOTICES.md`).

## Process

1. **Ask the user** which DuckEight tool needs a logo
2. **Discuss emoji choices** — pick 1-3 emojis that represent the tool. Keep it
   simple.
3. **Confirm the tool name** text to display
4. **Generate the logo** using the `domino` CLI:

```bash
domino --emojis "<two-emojis>" --text "<tool-name>" --color <hex> -o logos/<tool-name>-logo.png
domino --emojis "<two-emojis>" --text "<tool-name>" --color <hex> --format svg -o logos/<tool-name>-logo.svg
```

5. **Inspect the result** — read the PNG to view it. Check it on both a light and
   a dark background (composite it over white and over dark gray) so the outline
   and text color read on both. This visual feedback loop is the whole point:
   look at the pixels, then adjust.
6. **Iterate** if needed — swap emojis, change `--color`, or tune `--badge-fill`
   / `--text-scale`.

## Prerequisites

The `domino` CLI must be installed. Build it from the plugin source:

```bash
cd ${CLAUDE_PLUGIN_ROOT}/domino && cargo build --release
```

The binary will be at `${CLAUDE_PLUGIN_ROOT}/domino/target/release/domino`.

## Domino CLI Reference

```
domino --emojis "🦆🎱" --text "duckeight" [OPTIONS] -o output.png

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
domino --emojis "🦆🎱" --text "duckeight" --color f4a030 -o logos/duckeight-logo.png
```

Teleprompt logo:
```bash
domino --emojis "🎬🎙" --text "teleprompt" --color 3a86c8 -o logos/teleprompt-logo.png
```

## Gotchas

- **Strip variation selectors.** Sprites are named by base codepoint, so an
  emoji carrying U+FE0F (🗣️, 🎙️, 🎞️) fails to resolve with "could not render
  emoji". Pass the bare glyph (🗣, 🎙, 🎞) instead.
