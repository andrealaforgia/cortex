# Cortex AI Terminal -- Design Decisions (v2 Post-Phase 1)

**Version:** 2 (Post-Phase 1 Implementation)
**Date:** 2026-03-23
**Total decisions:** 18 (12 documented, 6 inferred from implementation)

---

## Documented Decisions

These decisions are explicitly stated in the design documents.

### ADR-001: Tauri v2 Over Electron

**Context:** The application needs a desktop framework that combines a web frontend with native backend capabilities (PTY management, HTTP streaming, file system access).

**Decision:** Use Tauri v2 with Rust backend.

**Consequences:**
- (+) ~10MB binary vs ~150MB for Electron
- (+) Rust backend provides native-speed PTY management and HTTP streaming
- (+) Cross-platform (macOS, Linux, Windows)
- (-) Smaller ecosystem than Electron
- (-) WebView rendering differences across platforms

**Source:** architecture-design.md Section 5.1

---

### ADR-002: xterm.js with WebGL Over Custom Renderer

**Context:** The terminal emulator needs to render VT100/ANSI escape sequences, handle Unicode, IME input, and provide adequate performance.

**Decision:** Use xterm.js with the WebGL renderer addon.

**Consequences:**
- (+) Battle-tested: powers VS Code terminal, Tabby, Hyper
- (+) Full VT100/ANSI support including Unicode and accessibility
- (+) WebGL addon provides GPU-accelerated rendering within the webview
- (-) Not as fast as native GPU rendering (e.g., Alacritty)
- (-) Depends on webview GPU support

**Source:** architecture-design.md Section 5.2, technology-stack.md Section 2.2

---

### ADR-003: portable-pty for PTY Management

**Context:** Need a cross-platform PTY abstraction to spawn shell processes and manage I/O.

**Decision:** Use the portable-pty crate.

**Consequences:**
- (+) Created by WezTerm author, production-proven
- (+) Cross-platform: Unix PTY on macOS/Linux, ConPTY on Windows
- (+) Clean API: spawn, read, write, resize
- (-) Synchronous read API requires dedicated thread (not Tokio-compatible)

**Source:** technology-stack.md Section 2.3

---

### ADR-004: reqwest + Manual SSE Parsing for Claude API

**Context:** No official Anthropic Rust SDK exists. Need to make streaming HTTP requests to the Messages API.

**Decision:** Use reqwest for HTTP and parse SSE manually.

**Consequences:**
- (+) Full control over timeouts, retries, error handling
- (+) No dependency on unofficial/unmaintained SDKs
- (-) Manual SSE parsing required
- (-) Must track Anthropic API changes manually

**Source:** technology-stack.md Section 2.4

---

### ADR-005: Streaming Callback Pattern

**Context:** AI responses must be streamed to the frontend in real-time. The architecture must keep Tauri event emission in the command layer (primary adapter), not in the API client (secondary adapter).

**Decision:** Use a closure-based callback (`impl Fn(StreamEvent)`) passed to the API client. The callback emits Tauri events.

**Consequences:**
- (+) API client has no knowledge of Tauri (clean separation)
- (+) Flexible: callback can be any closure (testable with mock callbacks)
- (-) Design doc specified a formal trait (`StreamingCallbacks`); implementation uses a simpler closure

**Source:** component-boundaries.md Section 1.2 (ClaudeApiPort), architecture-design.md Section 4.4

---

### ADR-006: SQLite via rusqlite

**Context:** Need persistent storage for configuration, command history, AI cache, and conversations.

**Decision:** Use rusqlite with SQLite, WAL journal mode, and foreign keys.

**Consequences:**
- (+) Zero-configuration embedded database
- (+) WAL mode for crash safety and concurrent read/write
- (+) Single-file storage
- (-) No async API (but queries are fast and local)

**Source:** technology-stack.md Section 2.5

---

### ADR-007: OSC 133 for Block Detection

**Context:** Need to detect command boundaries (prompt start, command execution, command completion) to segment terminal output into discrete blocks.

**Decision:** Use the OSC 133 escape sequence protocol.

**Consequences:**
- (+) Industry standard (Ghostty, Kitty, WezTerm, VS Code, iTerm2)
- (+) Shell integration scripts are simple (~20 lines each)
- (+) Compatible with other terminals that support OSC 133
- (-) Requires shell integration scripts to be sourced

**Source:** architecture-design.md Section 5.5

---

### ADR-008: OS Keychain for API Key Storage

**Context:** The Anthropic API key provides direct access to a paid service. Storage must be secure.

**Decision:** Use OS keychain via the keyring crate (macOS Keychain, Windows Credential Manager, Linux Secret Service). Fall back to ANTHROPIC_API_KEY environment variable.

**Consequences:**
- (+) OS-level encryption for the API key
- (+) Cross-platform support
- (-) Not yet implemented -- MVP uses env var only

**Source:** architecture-design.md Section 5.6, technology-stack.md Section 2.6

**Implementation status:** Deferred. Currently using `std::env::var("ANTHROPIC_API_KEY")`.

---

### ADR-009: Redaction Engine for Security

**Context:** Terminal output may contain secrets (API keys, passwords, connection strings, private keys). This content must never reach the Claude API unfiltered.

**Decision:** Implement a regex-based redaction engine that processes all content before API calls.

**Consequences:**
- (+) Defense-in-depth against accidental secret leakage
- (+) Testable in isolation (4 unit tests)
- (+) Extensible with custom patterns
- (-) Regex-based approach may have false positives/negatives
- (-) No ML-based secret detection yet

**Source:** architecture-design.md Section 6.2

---

### ADR-010: Hexagonal Architecture

**Context:** The system needs clear boundaries between business logic and infrastructure to enable testability and future adapter swapping.

**Decision:** Use hexagonal (ports and adapters) architecture with the invariant that domain logic never imports infrastructure code.

**Consequences:**
- (+) Domain logic testable without infrastructure
- (+) Adapters are swappable (e.g., mock PTY for tests)
- (+) Clear dependency direction enforced by module structure
- (-) More files and indirection than a flat structure
- Note: MVP simplifies by not using formal traits -- ports/ contains concrete implementations

**Source:** architecture-design.md Section 2, component-boundaries.md Section 3

---

### ADR-011: Zustand for Frontend State

**Context:** Need a state management solution for terminal sessions, AI conversations, and streaming state.

**Decision:** Use Zustand.

**Consequences:**
- (+) Minimal boilerplate (~30 lines per store)
- (+) No providers or context wrappers
- (+) ~2KB bundle size
- (+) TypeScript-first with good type inference
- (-) Less structured than Redux for large teams

**Source:** technology-stack.md Section 2.9

---

### ADR-012: AI Conversation Binding Rules

**Context:** Some AI operations (translate, explain) are ephemeral and scoped to a block. Others (chat) are persistent conversations.

**Decision:** Translate and explain operations do not create conversation entries. Chat operations create/append to persistent conversations stored in SQLite.

**Consequences:**
- (+) Clean separation between inline help and persistent chat
- (+) Less storage for ephemeral interactions
- (-) No history for translate/explain interactions

**Source:** component-boundaries.md Section 1.1 (AIAssistantPort)

---

## Inferred Decisions

These decisions are not explicitly documented but are inferred from the implementation.

### INF-001: Deferred Trait-Based Abstraction

**Context:** The design documents specify formal Rust traits for ports (PtyPort, ClaudeApiPort, StoragePort) with separate adapter implementations.

**Decision (inferred):** For the MVP, port modules contain concrete structs (PtyHandle, Storage) rather than trait definitions with separate adapter impls. No `adapters/` directory exists.

**Evidence:** `src-tauri/src/ports/pty.rs` defines `PtyHandle` and `PtyReader` as concrete structs, not trait implementations. Same for `Storage` and `claude_api` module.

**Consequences:**
- (+) Faster development, less boilerplate
- (-) Cannot swap adapters or use mocks without refactoring
- (-) Diverges from documented architecture
- The critical domain isolation invariant is still preserved

---

### INF-002: Dedicated OS Thread for PTY Read Loop

**Context:** portable-pty provides a synchronous (blocking) read API. Tokio's runtime should not be blocked by synchronous I/O.

**Decision (inferred):** Use `std::thread::spawn` for the PTY read loop instead of `tokio::spawn`.

**Evidence:** `commands/terminal.rs` line 57 uses `std::thread::spawn(move || { pty_read_loop(...); })`.

**Consequences:**
- (+) Correct: does not block the Tokio async runtime
- (+) Simple and reliable for blocking I/O
- (-) OS thread per terminal session (acceptable for desktop app)

---

### INF-003: Full-Body SSE Parsing Instead of True Streaming

**Context:** The Claude API returns Server-Sent Events (SSE). True streaming would process each chunk as it arrives from the network.

**Decision (inferred):** Read the entire HTTP response body as text, then parse SSE lines. Callbacks fire only after the full body is buffered.

**Evidence:** `ports/claude_api.rs` line 63: `let text = response.text().await?;` followed by `for line in text.lines()`.

**Consequences:**
- (-) Defeats the purpose of SSE streaming -- user sees nothing until full response arrives
- (-) For long AI responses, creates a noticeable delay
- (+) Simpler implementation; avoids dealing with partial chunks
- This should be fixed by switching to `response.bytes_stream()` with incremental parsing

---

### INF-004: Catppuccin Mocha Theme Hardcoded

**Context:** The terminal needs a visual theme.

**Decision (inferred):** Hardcode the Catppuccin Mocha color scheme in useTerminal.ts and index.html.

**Evidence:** `useTerminal.ts` lines 29-51 define all 16 ANSI colors using Catppuccin Mocha hex values. `index.html` sets `background: #1e1e2e`.

**Consequences:**
- (+) Consistent, attractive dark theme out of the box
- (-) No theme switching yet (design doc mentions dark/light themes)
- (-) Theme values duplicated between HTML and TypeScript

---

### INF-005: Pragmatic Cleanup Storage in useTerminal

**Context:** The React useTerminal hook creates multiple cleanup functions (event unlisteners, window resize handler) that must be called on unmount.

**Decision (inferred):** Store cleanup functions as properties on the xterm.js Terminal instance using `(terminal as any)._cleanup`.

**Evidence:** `useTerminal.ts` lines 114, 127 assign to `(terminal as any)._cleanup` and `(terminal as any)._resizeCleanup`.

**Consequences:**
- (+) Works correctly; cleanup executes on unmount
- (-) Bypasses TypeScript type checking
- (-) Non-idiomatic React (a useRef for cleanup would be standard)

---

### INF-006: Session Map with Arc-Mutex-HashMap

**Context:** Multiple Tauri commands access the active PTY sessions concurrently.

**Decision (inferred):** Use `Arc<Mutex<HashMap<String, PtyHandle>>>` as Tauri managed state for session storage.

**Evidence:** `commands/terminal.rs` line 8: `pub type SessionMap = Arc<Mutex<HashMap<String, PtyHandle>>>;`

**Consequences:**
- (+) Thread-safe concurrent access
- (+) Simple and correct for low-contention scenarios
- (-) Mutex lock held during PTY write/resize operations
- (-) For many concurrent sessions, could cause contention (unlikely in desktop app with few tabs)
