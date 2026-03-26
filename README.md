# duckeight-tools

Agent skills plugin for Claude Code, Codex, and OpenCode.

## Skills

### design-logo

Design logos for DuckEight tools using pixel art emojis and text.

## Pixie CLI

![pixie](logos/pixie-logo.png)

Emoji-to-pixel-art logo generator.

### Install

```bash
cd pixie && cargo install --path .
```

### Usage

```bash
pixie --emojis "🦆🎱" --text "duckeight" --resolution 64 -o logo.png
```

See `pixie --help` for all options.

## Plugin Installation

Add to your Claude Code settings:

```json
{
  "plugins": ["github:duckeight/duckeight-tools"]
}
```
