![kwakzalf](logos/kwakzalf-logo.png)

Agent skills plugin for Claude Code.

## Installation

Install from a marketplace that includes this plugin, or add it directly:

```bash
claude plugin add github:lightconelabs/kwakzalf
```

## Skills

### design-logo

Design logos for `duckeight` tools using pixel art emojis and text. Uses the bundled `pixie` CLI to render emoji sprites and pixel fonts into PNG/SVG logos.

## Pixie CLI

A Rust-based emoji-to-pixel-art logo generator bundled with the plugin. See [pixie/README.md](pixie/README.md) for details.

To build pixie from source:

```bash
cd pixie && cargo build --release
```
