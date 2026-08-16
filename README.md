![kwakzalf](logos/kwakzalf-logo.png)

Agent skills plugin for Claude Code.

## Installation

Install from a marketplace that includes this plugin, or add it directly:

```bash
claude plugin add github:lightconelabs/kwakzalf
```

## Skills

### design-logo

Design logos for `duckeight` tools by housing two emoji in a domino tile beside a modern wordmark. Uses the bundled `domino` CLI to render emoji sprites and text into PNG/SVG logos.

## Domino CLI

A Rust-based emoji logo generator bundled with the plugin. See [domino/README.md](domino/README.md) for details.

To build domino from source:

```bash
cd domino && cargo build --release
```
