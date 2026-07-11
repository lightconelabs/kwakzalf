# Third-Party Notices

## Emoji artwork

Pixie bundles emoji SVG artwork under `emoji-svg/`, compiled into the binary.

- **Noto Emoji** — the large majority of the bundled emoji (and all country
  flags) come from Google's [Noto Emoji](https://github.com/googlefonts/noto-emoji)
  project, licensed under the **SIL Open Font License 1.1**. The full license
  text is included at `emoji-svg/LICENSE-NOTO-OFL.txt`.
  Copyright 2013 Google LLC.

- **Twemoji** — a small number of glyphs not available in Noto's SVG set
  (keycaps, ©/®, subdivision flags, and a few skin-tone ZWJ sequences) come from
  Twitter's [Twemoji](https://github.com/twitter/twemoji) project, licensed under
  **CC-BY 4.0**. Copyright Twitter, Inc and other contributors.

## Text font

- **Press Start 2P** by CodeMan38, licensed under the **SIL Open Font License 1.1**
  (`fonts/`).

## Apple emoji

Apple Color Emoji is **proprietary** and is intentionally **not** bundled. To
render logos with genuine Apple artwork on a machine you own (e.g. macOS), export
the glyphs you need as PNGs named by codepoint and point Pixie at them:

```bash
pixie --emojis "🦆🎱" --text "duckeight" --color f4a030 \
      --sprites-dir ./apple-sprites -o logo.png
# ./apple-sprites/1f986.png, ./apple-sprites/1f3b1.png, ...
```

Sprites are matched by the emoji's dash-joined codepoints (e.g. `1f986.png` for
🦆, `1f1eb-1f1f7.png` for 🇫🇷). Keep those files out of any repository you
redistribute — Apple's artwork is not licensed for redistribution.
