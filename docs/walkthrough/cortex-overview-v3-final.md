---
marp: true
theme: uncover
class: invert
paginate: true
header: "AI Terminal (Cortex) -- Overview v3 Final"
footer: "March 2026"
style: |
  section {
    font-family: 'Inter', -apple-system, sans-serif;
    background: #1e1e2e;
    color: #cdd6f4;
  }
  h1 { color: #89b4fa; font-size: 1.8em; }
  h2 { color: #89b4fa; font-size: 1.4em; }
  h3 { color: #a6e3a1; font-size: 1.1em; }
  code { background: #313244; color: #f9e2af; padding: 2px 6px; border-radius: 4px; }
  pre { background: #313244; border-radius: 8px; }
  table { font-size: 0.75em; }
  th { background: #313244; color: #89b4fa; }
  td { background: #1e1e2e; border-color: #45475a; }
  img { background: transparent; }
  strong { color: #f9e2af; }
  em { color: #a6adc8; }
---

# AI Terminal (Cortex)

## Quick Overview -- v3 Final

Tauri v2 + React + xterm.js + Claude API

*All implementation phases complete*

<!-- _class: lead -->

---

# What Is It?

An AI-driven terminal emulator that runs on your desktop.

- **Terminal** -- full xterm.js emulation with WebGL rendering
- **AI sidebar** -- chat with Claude, translate natural language to commands
- **Privacy first** -- your API key, direct API calls, no intermediary servers
- **Secret redaction** -- credentials stripped before reaching the AI

**Tech:** ~2,300 lines across Rust backend and React frontend.

---

# System Context

![w:850](diagrams/v3-system-context.svg)

One external dependency: HTTPS to `api.anthropic.com`. Everything else is local.

---

# Architecture

![w:800](diagrams/v3-rust-architecture.svg)

Hexagonal-inspired layout: `commands/` (IPC) -- `domain/` (logic) -- `ports/` (external systems)

---

# Frontend Components

![w:800](diagrams/v3-component-hierarchy.svg)

---

# SSE Streaming Flow

![w:850](diagrams/v3-sse-streaming.svg)

Real-time streaming via `reqwest::bytes_stream()` -- fixed from v2.

---

# Key Technology Choices

| Choice | Why |
|--------|-----|
| **Tauri v2** over Electron | ~10MB vs ~150MB binary, Rust backend |
| **API in Rust** not JavaScript | API key never in webview, redaction before send |
| **xterm.js** | Battle-tested (VS Code, Hyper), WebGL renderer |
| **Zustand** over Redux | ~75 lines total for 2 stores, no boilerplate |
| **SQLite** | Zero-config embedded DB, WAL mode for safety |
| **Manual SSE** | No extra dependency, 76 lines of parsing code |

---

# What Shipped

| Feature | Status |
|---------|--------|
| Terminal emulation (xterm.js + WebGL) | Shipped |
| PTY management (spawn/read/write/resize) | Shipped |
| AI chat sidebar with streaming | Shipped |
| Command translation (`/cmd` prefix) | Shipped |
| Error diagnosis (backend command) | Shipped |
| Secret redaction (6 patterns + tests) | Shipped |
| Settings UI (API key + model selection) | Shipped |
| SQLite schema + config persistence | Shipped |
| Shell integration (OSC 133) | Shipped |

---

# What's Deferred

| Feature | Reason |
|---------|--------|
| Block UI (visual command boundaries) | Frontend OSC 133 parsing not wired |
| Inline completions (ghost text) | Depends on block detection |
| Keychain API key storage | In-memory + env var sufficient for MVP |
| Conversation persistence to SQLite | Schema exists, not yet connected |
| Multi-tab sessions | Single session adequate for MVP |

---

# Getting Started

```bash
# Prerequisites: Rust 1.77+, Node.js 18+, Tauri CLI

npm install
export ANTHROPIC_API_KEY="sk-ant-..."
cargo tauri dev
```

**Shortcuts:**
- `Ctrl+Shift+A` -- Toggle AI panel
- `Ctrl+,` -- Open settings

**AI panel:** type questions directly, or use `/cmd` prefix
for natural language to command translation.

---

# Codebase at a Glance

| | Files | Lines |
|---|-------|-------|
| Rust backend | 13 | 1,131 |
| React frontend | 10 | 974 |
| Shell integration | 3 | 68 |
| **Total** | **26** | **~2,173** |

**Largest file:** `commands/ai.rs` (301 lines) -- all three AI command handlers.

**Entry point:** `lib.rs` (58 lines) bootstraps the entire application.

A new developer can read the full source in under one hour.

---

# Summary

Cortex is a **working AI terminal MVP** with clean Tauri IPC boundaries.

The Rust backend handles PTY, Claude API, persistence, and security.
The React frontend handles rendering and user interaction.

**To contribute:** start with `lib.rs` (wiring), then `commands/ai.rs` (AI flow),
then `useTerminal.ts` (terminal lifecycle).

All design docs live in `docs/feature/ai-terminal/design/`.

<!-- _class: lead -->
