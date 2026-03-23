---
marp: true
theme: default
class: invert
paginate: true
header: "Cortex AI Terminal -- System Walkthrough v2 (Post-Phase 1)"
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
  .tag { font-size: 0.6em; color: #585b70; }
---

# Cortex AI Terminal

## System Walkthrough -- Version 2

Post-Phase 1 Implementation

**1,709 lines of source code | Rust + TypeScript + React**
**4 passing tests | 5 git commits | Single developer**

<!-- _class: lead -->

---

# What Changed Since v1

Version 1 analyzed design documents only. The project now has a working implementation.

| Aspect | v1 (Design Phase) | v2 (Phase 1 Complete) |
|--------|--------------------|-----------------------|
| Source code | 0 lines | 1,709 lines |
| Rust backend | Planned | 1,124 lines across 13 files |
| TypeScript frontend | Planned | 517 lines across 9 files |
| Shell scripts | Planned | 68 lines across 3 files |
| Tests | 0 | 4 (redaction engine) |
| Database | Schema designed | Schema implemented, WAL mode |
| IPC bridge | Contracts defined | 13 commands + 6 event types |

<span class="tag">Type: Explanation | Tier: 1</span>

---

<!-- _header: "SETTING" -->

# The Problem

Developers context-switch between their terminal and AI chat tools constantly.

**Current workflow:**
1. Hit an error in the terminal
2. Copy the error text
3. Switch to browser / AI chat
4. Paste, wait for response
5. Copy the suggested fix
6. Switch back to terminal
7. Paste and run

Cortex eliminates steps 2 through 6 by embedding Claude directly into the terminal.

<span class="tag">Type: Explanation | Tier: 1</span>

---

# System Context

![System Context](diagrams/v2-system-context.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# How It Works (30-Second Version)

1. **Cortex is a desktop app** built with Tauri v2 (Rust backend + web frontend)
2. **The terminal** uses xterm.js to render a real shell (bash/zsh/fish) via PTY
3. **AI features** call the Anthropic Claude API directly with the user's own API key
4. **A redaction engine** strips secrets from all content before it reaches Claude
5. **SQLite** stores command history, AI conversations, and configuration
6. **No server** -- everything runs locally; only HTTPS to api.anthropic.com

<span class="tag">Type: Explanation | Tier: 1</span>

---

<!-- _header: "CHARACTERS" -->

# Code Organization: The Two Worlds

| Layer | Language | Lines | Purpose |
|-------|----------|-------|---------|
| Backend (`src-tauri/src/`) | Rust | 1,124 | PTY management, Claude API, storage, domain logic |
| Frontend (`src/`) | TypeScript/React | 517 | Terminal UI, state management, IPC bridge |
| Shell scripts (`shell-integration/`) | Bash/Zsh/Fish | 68 | OSC 133 escape sequences for block detection |

**Communication:** Tauri IPC -- frontend invokes Rust commands, backend emits events.

<span class="tag">Type: Reference | Tier: 1</span>

---

# Rust Backend Module Map

![Rust Module Architecture](diagrams/v2-rust-modules.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# Backend: File-by-File Breakdown

| Module | File | Lines | Responsibility |
|--------|------|-------|----------------|
| Entry | `lib.rs` | 58 | App bootstrap, state init, command registration |
| Entry | `main.rs` | 6 | Calls `app_lib::run()` |
| Commands | `commands/terminal.rs` | 140 | PTY session CRUD + read loop |
| Commands | `commands/ai.rs` | 301 | AI translate, explain, chat (largest file) |
| Commands | `commands/config.rs` | 67 | Config get/set, API key management |
| Domain | `domain/context.rs` | 78 | Prompt construction from shell context |
| Domain | `domain/redaction.rs` | 77 | Regex-based secret stripping (4 tests) |
| Ports | `ports/pty.rs` | 95 | PtyHandle + PtyReader wrappers |
| Ports | `ports/claude_api.rs` | 140 | HTTP + SSE stream parsing |
| Ports | `ports/storage.rs` | 154 | SQLite schema + CRUD operations |

<span class="tag">Type: Reference | Tier: 2</span>

---

# Frontend Component Hierarchy

![Frontend Components](diagrams/v2-frontend-components.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# Frontend: File-by-File Breakdown

| Layer | File | Lines | Responsibility |
|-------|------|-------|----------------|
| Entry | `main.tsx` | 9 | React root mount |
| Layout | `App.tsx` | 9 | Full-viewport dark container |
| Component | `TerminalView.tsx` | 54 | xterm.js container + error banner |
| Hook | `useTerminal.ts` | 148 | Terminal lifecycle, PTY session, event wiring |
| Bridge | `tauri-bridge.ts` | 162 | Typed wrappers for all 13 Tauri commands + 6 event listeners |
| Types | `types.ts` | 60 | Block, ShellContext, AppConfig, AIMessage |
| Store | `terminalStore.ts` | 30 | Session ID, blocks, error state |
| Store | `aiStore.ts` | 45 | Streaming state, conversations |

<span class="tag">Type: Reference | Tier: 2</span>

---

<!-- _header: "CHARACTERS -- DEEP DIVES" -->

# Deep Dive 1: The Hexagonal Architecture (As Built)

![Hexagonal Architecture](diagrams/v2-hexagonal-actual.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# Design vs. Implementation: Architecture Comparison

The design specified formal port traits and separate adapter modules. The implementation simplifies this for the MVP.

| Design Doc | Actual Implementation | Status |
|------------|----------------------|--------|
| `ports/` = trait definitions only | `ports/` = concrete implementations (PtyHandle, Storage, claude_api) | Simplified |
| `adapters/` = concrete impls | No `adapters/` directory exists | Deferred |
| `domain/` never imports `adapters/` | **Maintained** -- `domain/` imports only `domain/` types | Invariant preserved |
| Formal `StreamingCallbacks` trait | Closure-based callback (`impl Fn(StreamEvent)`) | Pragmatic MVP |
| Keychain via `keyring` crate | Environment variable (`ANTHROPIC_API_KEY`) | MVP fallback |

**Inferred:** The "trait + adapter" indirection was deferred for MVP velocity, but the critical invariant -- domain logic does not depend on infrastructure -- is maintained.

<span class="tag">Type: Explanation | Tier: 2</span>

---

# Deep Dive 2: PTY Read Loop

The PTY read loop is the heartbeat of the terminal. It runs in a dedicated OS thread (not a Tokio task) to avoid blocking the async runtime.

```rust
// commands/terminal.rs, line 64
fn pty_read_loop(mut reader: PtyReader, session_id: String, app: AppHandle) {
    let mut buf = [0u8; 4096];
    loop {
        match reader.read_chunk(&mut buf) {
            Ok(0) => {
                // EOF - shell exited
                app.emit("pty:exit", PtyExitPayload { session_id, code: 0 });
                break;
            }
            Ok(n) => {
                app.emit("pty:data", PtyDataPayload {
                    session_id, data: buf[..n].to_vec()
                });
            }
            Err(e) => {
                app.emit("pty:error", PtyErrorPayload {
                    session_id, error_type: "read_io_error",
                    message: e.to_string()
                });
                break;
            }
        }
    }
}
```

<span class="tag">Type: Explanation | Tier: 2</span>

---

# PTY Read Loop: Design Rationale

**Why a dedicated OS thread, not a Tokio task?**

`portable-pty`'s reader is synchronous (blocking `read()`). Running a blocking read inside a Tokio task would starve the async runtime. A dedicated `std::thread::spawn` is the correct pattern.

**Buffer size: 4096 bytes.** Matches typical PTY buffer sizes. Large outputs (e.g., `cat` of a big file) arrive as multiple 4KB chunks, each emitted as a separate `pty:data` event.

**Error handling has three exit paths:**

| Condition | Meaning | Event Emitted |
|-----------|---------|---------------|
| `Ok(0)` | EOF -- shell exited cleanly | `pty:exit` (code 0) |
| `Ok(n)` | Data received | `pty:data` (bytes) |
| `Err(e)` | I/O error (fd closed, crash) | `pty:error` + loop breaks |

**Documented concern:** The design doc specifies that on error, the loop should also cancel pending AI requests and flush history. This is not yet implemented.

<span class="tag">Type: Explanation | Tier: 2</span>

---

# PTY Data Flow: End to End

![PTY Data Flow](diagrams/v2-pty-data-flow.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# Deep Dive 3: Claude API Streaming

The streaming callback pattern is the key architectural decision for AI integration. Instead of waiting for the full response, each text chunk is emitted as a Tauri event in real time.

**Pattern: Async task + closure callback**

```rust
// commands/ai.rs (simplified)
tauri::async_runtime::spawn(async move {
    claude_api::send_message_streaming(
        &api_key, model, &system_prompt, &messages, max_tokens,
        move |event| match event {
            StreamEvent::TextDelta(text) =>
                app.emit("ai:stream-chunk", { request_id, text }),
            StreamEvent::Completion(meta) =>
                app.emit("ai:stream-end", { request_id, meta }),
            StreamEvent::Error(msg) =>
                app.emit("ai:error", { request_id, msg }),
        },
    ).await;
});
```

The command returns `{ request_id }` immediately. The frontend subscribes to events with that request_id.

<span class="tag">Type: Explanation | Tier: 2</span>

---

# AI Streaming: End to End

![AI Streaming Flow](diagrams/v2-ai-streaming-flow.svg)

<span class="tag">Type: Explanation | Tier: 1</span>

---

# SSE Parsing Implementation

The Claude API adapter in `ports/claude_api.rs` parses the SSE stream by reading the full response body and splitting on `data: ` prefixed lines.

**Current approach** (lines 62-108): Read entire response as text, then iterate lines.

```rust
// ports/claude_api.rs (simplified)
let text = response.text().await?;
for line in text.lines() {
    if !line.starts_with("data: ") { continue; }
    let data = &line[6..];
    match event["type"].as_str() {
        Some("content_block_delta") =>
            on_event(StreamEvent::TextDelta(text)),
        Some("message_delta") => { /* extract stop_reason */ }
        Some("message_start") => { /* extract model, input_tokens */ }
        _ => {}
    }
}
```

**Observation:** This reads the full body before parsing, which means the callback fires after the entire response is buffered -- not as each SSE chunk arrives. For true streaming, the implementation should use `response.bytes_stream()` and parse incrementally.

<span class="tag">Type: Explanation | Tier: 3</span>

---

# Deep Dive 4: The Redaction Engine

The only module with tests. Protects against sending secrets to the Claude API.

**6 regex patterns:**

| Pattern | Example Match | Replacement |
|---------|---------------|-------------|
| API keys/tokens/passwords | `API_KEY=sk-abc123` | `API_KEY=***REDACTED***` |
| Bearer tokens | `Bearer eyJhbG...` | `Bearer ***REDACTED***` |
| AWS credentials | `AWS_SECRET_ACCESS_KEY=...` | `AWS_SECRET_ACCESS_KEY=***REDACTED***` |
| Private keys | `-----BEGIN RSA PRIVATE KEY-----` | `***PRIVATE_KEY_REDACTED***` |
| Connection strings | `postgres://user:pass@host` | `postgres://***REDACTED***` |
| Anthropic API keys | `sk-ant-api03-...` | `***API_KEY_REDACTED***` |

**Test coverage:** 4 tests verify redaction of API keys, bearer tokens, connection strings, and passthrough of normal text.

<span class="tag">Type: Explanation | Tier: 2</span>

---

# Deep Dive 5: SQLite Schema

Implemented in `ports/storage.rs::initialize_schema()`. Uses WAL journal mode and foreign keys.

**6 tables created:**

| Table | Purpose | Primary Key |
|-------|---------|-------------|
| `config` | Key-value app settings | `key TEXT` |
| `command_history` | Executed commands with output | `id TEXT` (UUID) |
| `ai_cache` | Cached AI responses by prompt hash | `prompt_hash TEXT` |
| `ai_conversations` | Chat conversation headers | `id TEXT` (UUID) |
| `ai_messages` | Individual chat messages | `id TEXT` (UUID) |
| `schema_version` | Migration tracking | `version INTEGER` |

**Indexes:** 3 indexes on `command_history` (session, created_at) and `ai_messages` (conversation_id + created_at).

**Note:** The `save_command_history` and `get_config` methods exist but are currently unused (compiler warning confirms this). They are ready for Phase 2 block persistence.

<span class="tag">Type: Reference | Tier: 2</span>

---

# Deep Dive 6: The IPC Bridge

![IPC Events and Commands](diagrams/v2-ipc-events.svg)

<span class="tag">Type: Reference | Tier: 2</span>

---

# IPC Bridge: Type Safety Across Languages

The `tauri-bridge.ts` file (162 lines) provides typed wrappers for every Tauri command and event. This is the single integration point between React and Rust.

**Commands (Frontend to Backend):**
- 4 terminal commands: `createTerminalSession`, `writeToPty`, `resizePty`, `closeTerminalSession`
- 4 AI commands: `aiTranslateCommand`, `aiExplainError`, `aiChat`, `aiCancel`
- 5 config commands: `getConfig`, `setConfig`, `storeApiKey`, `hasApiKey`, `deleteApiKey`

**Events (Backend to Frontend):**
- 3 PTY events: `pty:data`, `pty:exit`, `pty:error`
- 3 AI events: `ai:stream-chunk`, `ai:stream-end`, `ai:error`

**Type mirroring:** TypeScript interfaces in `types.ts` mirror Rust structs (ShellContext, CommandEntry, etc.). Both sides serialize to the same JSON shape.

<span class="tag">Type: Reference | Tier: 2</span>

---

# Deep Dive 7: State Management

Two Zustand stores manage frontend state. Zustand was chosen over Redux for minimal boilerplate and over React Context to avoid re-render problems with high-frequency terminal updates.

### terminalStore.ts
| State | Type | Purpose |
|-------|------|---------|
| `sessionId` | `string | null` | Active PTY session |
| `blocks` | `Block[]` | Command blocks (for future OSC 133) |
| `error` | `string | null` | Displayed in error banner |

### aiStore.ts
| State | Type | Purpose |
|-------|------|---------|
| `isStreaming` | `boolean` | Whether AI response is in progress |
| `currentRequestId` | `string | null` | Active AI request |
| `streamingText` | `string` | Accumulated streaming text |
| `conversations` | `AIConversation[]` | Chat history |
| `error` | `string | null` | AI-specific errors |

<span class="tag">Type: Reference | Tier: 2</span>

---

# The useTerminal Hook: Terminal Lifecycle

`useTerminal.ts` (148 lines) is the most complex frontend file. It orchestrates the full terminal lifecycle.

**Initialization sequence:**
1. Create xterm.js `Terminal` with Catppuccin Mocha theme
2. Load addons: FitAddon, WebLinksAddon, WebglAddon (with canvas fallback)
3. Call `createTerminalSession()` via Tauri bridge
4. Wire `terminal.onData` to `writeToPty()` (keystrokes to PTY)
5. Wire `terminal.onResize` to `resizePty()` (dimension changes)
6. Subscribe to `pty:data`, `pty:exit`, `pty:error` events
7. Set up window resize handler for `fitAddon.fit()`

**Cleanup:** On unmount, unsubscribes all event listeners, closes PTY session, disposes xterm.js instance.

**Inferred:** Cleanup functions are stored as `(terminal as any)._cleanup` -- a pragmatic but type-unsafe approach. A cleanup ref would be more idiomatic React.

<span class="tag">Type: Explanation | Tier: 2</span>

---

<!-- _header: "CONFLICT" -->

# Shell Integration: OSC 133 Protocol

Three scripts emit escape sequences to mark prompt boundaries, enabling block detection.

**Protocol summary:**

| Sequence | Meaning | Emitted When |
|----------|---------|--------------|
| `ESC]133;A` | Prompt start | Before PS1 is displayed |
| `ESC]133;B` | Command start | After user presses Enter (implicit) |
| `ESC]133;C` | Command executed | Just before shell runs the command |
| `ESC]133;D;N` | Command finished | After command exits (N = exit code) |

**Implementation status:** Scripts exist for bash (trap + PROMPT_COMMAND), zsh (precmd + preexec hooks), and fish (event handlers). However, the frontend OSC 133 parser (`osc133.ts`) is **not yet implemented** -- block detection is a Phase 2 deliverable.

<span class="tag">Type: Explanation | Tier: 2</span>

---

<!-- _header: "RESOLUTION" -->

# Key Design Decisions (Documented)

| ID | Decision | Rationale |
|----|----------|-----------|
| ADR-001 | Tauri v2 over Electron | 10MB vs 150MB binary; Rust backend performance |
| ADR-002 | xterm.js + WebGL | Battle-tested (VS Code, Hyper); full VT100 support |
| ADR-003 | portable-pty | WezTerm-proven; cross-platform PTY abstraction |
| ADR-004 | reqwest + manual SSE | No official Anthropic Rust SDK |
| ADR-005 | Closure-based streaming callbacks | Keeps Tauri event emission in command layer |
| ADR-006 | SQLite via rusqlite | Zero-config; WAL mode for crash safety |
| ADR-007 | OSC 133 for block detection | Industry standard (Ghostty, Kitty, VS Code) |

<span class="tag">Type: Reference | Tier: 1</span>

---

# Design Decisions (Inferred from Implementation)

| Observation | Inferred Decision | Consequence |
|-------------|-------------------|-------------|
| No `adapters/` directory | Deferred trait-based abstraction for MVP | Ports are concrete structs, not trait objects |
| `std::thread::spawn` for PTY loop | Avoid blocking Tokio runtime | Correct pattern for synchronous I/O |
| `response.text().await` in SSE parser | Simpler parsing over true streaming | Callbacks fire after full response buffered |
| `(terminal as any)._cleanup` | Pragmatic cleanup storage | TypeScript type safety sacrificed |
| Catppuccin Mocha hardcoded | Theme matching is important for UX | No theme switching implemented yet |
| `Arc<Mutex<HashMap>>` for sessions | Thread-safe session storage | Potential lock contention with many sessions |

<span class="tag">Type: Explanation | Tier: 2</span>

---

# Module Dependency Invariant

**The critical architectural rule:** `domain/` never imports from `ports/` or `commands/`.

**Verification from the actual code:**

```
domain/context.rs imports:
  - crate::domain::redaction::RedactionEngine  (domain)
  - serde::{Deserialize, Serialize}            (external)

domain/redaction.rs imports:
  - regex::Regex                               (external)
```

**Result: INVARIANT PRESERVED.** Domain logic depends only on other domain types and external utility crates. It can be tested without any infrastructure.

The reverse direction confirms correct dependency flow:
- `commands/ai.rs` imports from `domain::context` and `domain::redaction` (allowed)
- `commands/terminal.rs` imports from `ports::pty` (allowed)
- No circular dependencies detected.

<span class="tag">Type: Explanation | Tier: 2</span>

---

<!-- _header: "EPILOGUE" -->

# Implementation Status

![Implementation Status](diagrams/v2-implementation-status.svg)

<span class="tag">Type: Reference | Tier: 1</span>

---

# What Works Now vs. What Is Planned

### Implemented (Phase 1)
- Working terminal with PTY I/O (spawn, read, write, resize, close)
- xterm.js with WebGL rendering and Catppuccin theme
- Claude API client with SSE streaming (buffered, not true streaming)
- Redaction engine with 6 patterns and 4 passing tests
- SQLite schema with 6 tables and WAL mode
- Shell integration scripts for bash, zsh, fish
- Zustand stores for terminal and AI state
- Full Tauri IPC bridge with 13 commands and 6 event types

### Not Yet Implemented (Phase 2+)
- OSC 133 block detection and parsing
- AI Panel sidebar UI
- Settings UI and theme switching
- Keychain integration (currently env var only)
- Command suggestions with confirm/reject UI
- AI response caching
- Multi-tab terminal sessions
- CI/CD pipeline

<span class="tag">Type: Reference | Tier: 1</span>

---

# Test Coverage

| Module | Tests | Type | Status |
|--------|-------|------|--------|
| `domain/redaction.rs` | 4 | Unit | All passing |
| `domain/context.rs` | 0 | -- | No tests |
| `ports/pty.rs` | 0 | -- | No tests (needs real PTY) |
| `ports/claude_api.rs` | 0 | -- | No tests (needs mock server) |
| `ports/storage.rs` | 0 | -- | No tests (needs temp DB) |
| `commands/*` | 0 | -- | No tests |
| Frontend (`src/`) | 0 | -- | No test framework configured |

**Compiler warnings:** `get_config` and `save_command_history` in `storage.rs` are defined but never called (dead code). These are pre-built for Phase 2.

<span class="tag">Type: Reference | Tier: 1</span>

---

# Risks and Technical Debt

| Risk | Severity | Details |
|------|----------|---------|
| SSE parsing is buffered, not streaming | **High** | `response.text().await` buffers entire response before parsing. For long AI responses, the user sees nothing until the full response arrives. Must switch to incremental `bytes_stream()` parsing. |
| No tests for PTY, API, or storage | **Medium** | Only redaction engine has tests. Infrastructure code is untested. Mock adapters (from design doc) would enable isolated testing. |
| No CI/CD pipeline | **Medium** | No automated checks. Regressions can be introduced silently. |
| `ai_cancel` is a no-op | **Low** | The cancel command exists in the IPC contract but does nothing. No way to abort a running AI request. |
| Dead code in storage | **Low** | Two storage methods unused. Not harmful but indicates incomplete integration. |
| Cleanup type hack in useTerminal | **Low** | `(terminal as any)._cleanup` bypasses TypeScript type checking. |

<span class="tag">Type: Explanation | Tier: 1</span>

---

# Recommendations

| Priority | Action | Effort |
|----------|--------|--------|
| 1 | **Fix SSE streaming** -- switch from `response.text()` to `response.bytes_stream()` for real-time chunk delivery | Medium |
| 2 | **Add integration tests** for PTY spawn/read/write cycle and SQLite schema initialization | Medium |
| 3 | **Implement OSC 133 parser** in frontend to enable block detection (Phase 2 core) | Medium |
| 4 | **Set up CI/CD** with `cargo check`, `cargo test`, `cargo clippy`, and `tsc --noEmit` | Low |
| 5 | **Add mock adapters** for Claude API and PTY to enable domain logic testing | Medium |
| 6 | **Implement real `ai_cancel`** using Tokio `AbortHandle` to cancel in-flight requests | Low |

<span class="tag">Type: How-To | Tier: 1</span>

---

# Technology Stack Summary

### Backend (Rust)

| Crate | Version | Purpose |
|-------|---------|---------|
| tauri | 2.10.3 | Application framework |
| portable-pty | 0.8 | PTY management |
| reqwest | 0.12 | HTTP client (Claude API) |
| rusqlite | 0.31 | SQLite (bundled) |
| tokio | 1.x | Async runtime |
| serde / serde_json | 1.x | Serialization |
| regex | 1.x | Redaction patterns |
| uuid | 1.x | Session/request IDs |

### Frontend (TypeScript)

| Package | Version | Purpose |
|---------|---------|---------|
| react | 18.3 | UI framework |
| @xterm/xterm | 5.5 | Terminal emulation |
| @xterm/addon-webgl | 0.18 | GPU-accelerated rendering |
| zustand | 4.5 | State management |
| @tauri-apps/api | 2.0 | Tauri IPC |

<span class="tag">Type: Reference | Tier: 2</span>

---

# Getting Started

```bash
# Prerequisites: Rust (1.77.2+), Node.js, npm

# Clone and install
cd /Users/andrealaforgia/dev/cortex
npm install

# Set API key (required for AI features)
export ANTHROPIC_API_KEY="sk-ant-..."

# Run in development mode
cargo tauri dev

# Run tests
cd src-tauri && cargo test

# Build for production
cargo tauri build
```

**Expected test output:**
```
running 4 tests
test domain::redaction::tests::test_redacts_api_keys ... ok
test domain::redaction::tests::test_redacts_connection_strings ... ok
test domain::redaction::tests::test_leaves_normal_text ... ok
test domain::redaction::tests::test_redacts_bearer_tokens ... ok
test result: ok. 4 passed; 0 failed
```

<span class="tag">Type: Tutorial | Tier: 1</span>

---

# Key Source Files for New Developers

**Start here** (in reading order):

1. `src-tauri/src/lib.rs` -- App bootstrap, see all managed state and registered commands
2. `src-tauri/src/commands/terminal.rs` -- PTY lifecycle and the critical read loop
3. `src-tauri/src/ports/pty.rs` -- How portable-pty is wrapped
4. `src/hooks/useTerminal.ts` -- Frontend terminal lifecycle
5. `src/lib/tauri-bridge.ts` -- The IPC contract between worlds
6. `src-tauri/src/commands/ai.rs` -- AI streaming pattern (largest file)
7. `src-tauri/src/domain/redaction.rs` -- Only tested module; good example of the testing pattern

<span class="tag">Type: How-To | Tier: 1</span>

---

# Summary

Cortex Phase 1 delivers a **functional terminal emulator** with the infrastructure for AI integration. The hexagonal architecture's critical invariant (domain independence) is preserved in practice. The codebase is compact (1,709 lines), well-organized, and ready for Phase 2 features.

**Strongest aspects:**
- Clean module separation with correct dependency direction
- Full IPC bridge with typed contracts on both sides
- Security-first design (redaction engine, API key isolation)

**Needs attention:**
- SSE streaming is buffered, not real-time
- Test coverage is minimal (4 tests, redaction only)
- No CI/CD pipeline

**Next milestone:** OSC 133 block detection + AI Panel UI (Phase 2)
