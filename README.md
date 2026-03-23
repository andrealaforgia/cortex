# AI Terminal

An AI-driven terminal emulator with Claude integration, inspired by [Warp](https://www.warp.dev/).

Built with **Tauri v2** (Rust backend) + **React** + **xterm.js** (frontend), using your own Anthropic API key.

## Features

- **Terminal emulation** via xterm.js with WebGL rendering
- **AI chat sidebar** (Ctrl+Shift+A) — ask questions, get command suggestions
- **Natural language to commands** — type `/cmd` followed by what you want to do
- **Error diagnosis** — AI-powered analysis of failed commands
- **Secret redaction** — API keys, passwords, and credentials are automatically stripped before sending to Claude
- **Shell integration** — OSC 133 protocol for bash, zsh, and fish

## Architecture

Hexagonal architecture with clear separation of concerns:

```
src-tauri/src/
├── ports/          # Interface definitions (PTY, Claude API, Storage)
├── domain/         # Business logic (redaction, context building)
├── adapters/       # Concrete implementations
└── commands/       # Tauri IPC handlers (primary adapters)
```

## Prerequisites

- [Rust](https://rustup.rs/) (1.77+)
- [Node.js](https://nodejs.org/) (18+)
- [Tauri CLI](https://v2.tauri.app/start/): `cargo install tauri-cli --version "^2"`

## Setup

```bash
# Install dependencies
npm install

# Set your API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Run in development mode
cargo tauri dev
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Shift+A | Toggle AI panel |
| Ctrl+, | Open settings |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Frontend | React 18 + TypeScript |
| Terminal | xterm.js + WebGL |
| PTY | portable-pty |
| AI | Claude API (Anthropic) |
| Database | SQLite (rusqlite) |
| State | Zustand |

## License

MIT
