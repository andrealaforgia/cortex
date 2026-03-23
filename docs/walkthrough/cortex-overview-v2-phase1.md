---
marp: true
theme: default
class: invert
paginate: true
header: "Cortex AI Terminal -- Quick Overview v2 (Post-Phase 1)"
footer: "March 2026"
style: |
  section {
    background: #1e1e2e;
    color: #cdd6f4;
    font-family: 'Inter', 'Segoe UI', sans-serif;
  }
  h1 { color: #89b4fa; }
  h2 { color: #a6e3a1; }
  h3 { color: #f9e2af; }
  strong { color: #f5c2e7; }
  code { color: #f9e2af; background: #313244; }
  a { color: #89b4fa; }
  table { font-size: 0.8em; }
  th { background: #313244; }
  td { background: #1e1e2e; }
  img { max-height: 450px; }
---

# Cortex AI Terminal

## Quick Overview -- Post-Phase 1

A desktop terminal emulator with integrated Claude AI assistance

**Rust + TypeScript + React | Tauri v2 | 1,709 lines of code**

<!-- _class: lead -->

---

# What Is Cortex?

A terminal that embeds AI directly into the developer workflow.

**Instead of:**
Terminal --> copy error --> browser AI chat --> copy fix --> terminal

**With Cortex:**
Terminal --> AI explains error / suggests fix inline

**Key properties:**
- User's own Anthropic API key (no intermediary server)
- Redaction engine strips secrets before anything reaches Claude
- Everything runs locally; only HTTPS to api.anthropic.com

---

# System Architecture

![System Context](diagrams/v2-system-context.svg)

---

# How Code Is Organized

| Layer | Tech | Lines | What It Does |
|-------|------|-------|--------------|
| **Backend** | Rust (Tauri v2) | 1,124 | PTY management, Claude API, SQLite, redaction |
| **Frontend** | React + TypeScript | 517 | xterm.js terminal, state management, IPC bridge |
| **Shell scripts** | Bash/Zsh/Fish | 68 | OSC 133 sequences for block detection |

**Communication:** Tauri IPC -- frontend invokes Rust commands (13), backend emits events (6)

---

# Hexagonal Architecture (As Implemented)

![Hexagonal Architecture](diagrams/v2-hexagonal-actual.svg)

**Critical invariant preserved:** `domain/` never imports from `ports/` or `commands/`

---

# The PTY Read Loop

The terminal's heartbeat. Runs in a dedicated OS thread to avoid blocking the Tokio async runtime.

![PTY Data Flow](diagrams/v2-pty-data-flow.svg)

---

# AI Streaming Pattern

AI commands return a `request_id` immediately. Responses stream as Tauri events.

![AI Streaming Flow](diagrams/v2-ai-streaming-flow.svg)

---

# Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Tauri v2 over Electron | 10MB binary vs 150MB; native-speed Rust backend |
| xterm.js + WebGL | Battle-tested (VS Code); full VT100 support |
| portable-pty | WezTerm-proven; cross-platform PTY |
| Direct reqwest + SSE | No official Anthropic Rust SDK |
| SQLite + WAL | Zero-config embedded DB; crash safety |
| Zustand over Redux | Minimal boilerplate; 2KB bundle |
| OSC 133 for blocks | Industry standard (Ghostty, Kitty, VS Code) |

---

# Implementation Status

![Implementation Status](diagrams/v2-implementation-status.svg)

---

# Test Coverage

| Area | Tests | Status |
|------|-------|--------|
| Redaction engine | 4 unit tests | All passing |
| PTY, API, Storage, Commands | 0 | Not yet tested |
| Frontend | 0 | No test framework |

**Total: 4 tests passing.** Only domain logic (redaction) is covered.

---

# Top Risks

| Risk | Severity |
|------|----------|
| SSE parsing buffers full response before firing callbacks | **High** |
| Minimal test coverage (4 tests total) | **Medium** |
| No CI/CD pipeline | **Medium** |
| `ai_cancel` is a no-op | **Low** |

---

# Next Steps

1. **Fix SSE streaming** -- use `bytes_stream()` for real-time chunk delivery
2. **Add tests** for PTY lifecycle and SQLite operations
3. **Implement OSC 133 parser** in frontend (Phase 2 core feature)
4. **Build AI Panel** sidebar UI for chat interaction
5. **Set up CI/CD** with cargo check, cargo test, clippy, tsc

---

# Getting Started

```bash
# Install dependencies
cd /Users/andrealaforgia/dev/cortex && npm install

# Set API key for AI features
export ANTHROPIC_API_KEY="sk-ant-..."

# Development mode
cargo tauri dev

# Run tests
cd src-tauri && cargo test
```

**Start reading:** `src-tauri/src/lib.rs` (bootstrap) then `commands/terminal.rs` (PTY lifecycle)
