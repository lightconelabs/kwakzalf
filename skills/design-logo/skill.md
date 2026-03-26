---
name: design-logo
description: Design a logo for a DuckEight tool using pixel art emojis and text
---

# Design a DuckEight Logo

You are designing a logo for a DuckEight tool. All DuckEight logos follow these brand constraints:

- **Style:** Simple pixel art
- **Composition:** One or more emojis (as pixel art) + tool name text, arranged horizontally (emojis left, text right)
- **Font:** Press Start 2P pixel font (bundled with pixie) or another pixel/mono font
- **Background:** Transparent
- **Resolution:** 64x64 per emoji recommended

## Process

1. **Ask the user** which DuckEight tool needs a logo
2. **Discuss emoji choices** — pick 1-3 emojis that represent the tool. Keep it simple
3. **Confirm the tool name** text to display
4. **Generate the logo** using the `pixie` CLI:

```bash
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --resolution 64 -o logos/<tool-name>-logo.png
pixie --emojis "<chosen-emojis>" --text "<tool-name>" --resolution 64 --format svg -o logos/<tool-name>-logo.svg
```

5. **Show the result** to the user (read the PNG file to display it)
6. **Iterate** if needed — adjust emojis, resolution, or font

## Pixie CLI Reference

```
pixie --emojis "🦆🎱" --text "duckeight" [OPTIONS] -o output.png

Options:
  --resolution 32|64    Pixel grid per emoji (default: 32)
  --format png|svg      Output format (default: png)
  --font path.ttf       Custom font override
  --sprites-dir path/   Custom sprite PNGs (named by codepoint)
```

## Examples

DuckEight logo:
```bash
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logos/duckeight-logo.png
```
