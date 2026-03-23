# AI Terminal (Cortex) -- Design Decisions v3 Final

**Date:** 2026-03-23
**Version:** 3 (Final -- all implementation phases complete)

---

## Decision Index

| # | Decision | Status |
|---|----------|--------|
| 1 | Tauri v2 over Electron | Documented |
| 2 | xterm.js for terminal emulation | Documented |
| 3 | API calls from Rust backend, not frontend | Documented |
| 4 | SQLite via rusqlite for persistence | Documented |
| 5 | OSC 133 for block detection | Documented |
| 6 | Zustand for frontend state | Documented |
| 7 | Manual SSE parsing over eventsource-client | Inferred |
| 8 | In-memory API key over keychain | Inferred |
| 9 | Pragmatic hexagonal (no Rust traits) | Inferred |
| 10 | Inline styles over Tailwind CSS | Inferred |
| 11 | Dedicated thread for PTY read loop | Inferred |
| 12 | Catppuccin Mocha theme | Inferred |

---

## Decision 1: Tauri v2 Over Electron

**Status:** Documented (architecture-design.md, section 5.1)

### Context
Need a desktop application framework that supports a Rust backend and a web-based frontend. The application is a terminal emulator where binary size and memory usage matter.

### Decision
Use Tauri v2.

### Consequences
- **Positive:** Binary size ~10MB vs ~150MB with Electron
- **Positive:** Rust backend provides native-speed PTY management and HTTP streaming without Node.js overhead
- **Positive:** IPC supports both synchronous commands and async events
- **Negative:** Smaller ecosystem than Electron, fewer community examples
- **Negative:** Uses OS-native webview (WebKit on macOS) which may have rendering differences from Chromium

### Alternatives Rejected
- Electron: 15x larger binary, higher memory overhead
- Native Swift (macOS only): not cross-platform
- Full Rust + WGPU: 12-24+ months estimated effort for custom terminal renderer

---

## Decision 2: xterm.js for Terminal Emulation

**Status:** Documented (architecture-design.md, section 5.2; technology-stack.md, section 2.2)

### Context
Need a terminal emulator component that handles the full complexity of VT100/ANSI parsing, Unicode, IME input, and accessibility.

### Decision
Use xterm.js with the WebGL renderer addon.

### Consequences
- **Positive:** Battle-tested in VS Code, Tabby, Hyper -- handles edge cases already solved
- **Positive:** WebGL addon provides GPU-accelerated text rendering
- **Positive:** Rich addon ecosystem (fit, search, web-links, image support)
- **Negative:** Runs in webview context, not native rendering
- **Negative:** Tied to JavaScript/WebView performance characteristics

### Alternatives Rejected
- Custom WGPU renderer: multi-year effort
- alacritty_terminal crate: designed for native rendering, not web
- Termwiz (WezTerm's lib): less documented, tighter coupling

---

## Decision 3: API Calls from Rust Backend, Not Frontend

**Status:** Documented (architecture-design.md, section 5.3)

### Context
The Claude API could be called directly from the frontend JavaScript, or routed through the Rust backend.

### Decision
All Claude API calls go through the Rust backend.

### Consequences
- **Positive:** API key never exposed to webview JavaScript context
- **Positive:** Redaction engine runs in Rust before data leaves the process
- **Positive:** Robust streaming HTTP via reqwest with proper error handling
- **Positive:** Backend can enforce rate limiting and cost controls
- **Negative:** Adds IPC overhead for every AI interaction (command invoke + event emission)

---

## Decision 4: SQLite via rusqlite for Persistence

**Status:** Documented (architecture-design.md, section 5.4; technology-stack.md, section 2.5)

### Context
Need persistent storage for configuration, command history, AI response cache, and conversation history.

### Decision
Use rusqlite with embedded SQLite (bundled feature).

### Consequences
- **Positive:** Zero-configuration, single-file database
- **Positive:** No external database process to manage
- **Positive:** WAL mode provides crash safety and concurrent read/write
- **Positive:** Handles all persistence needs without external dependencies
- **Negative:** No async database access (queries are blocking, but fast for local operations)

### Alternatives Rejected
- sqlx: async-first complexity is overkill for local SQLite
- sled: limited query capabilities for relational data
- Plain JSON files: no transactions, no query capability

---

## Decision 5: OSC 133 for Block Detection

**Status:** Documented (architecture-design.md, section 5.5)

### Context
Need a way to identify command boundaries (prompt start, command start, command output, command end) in terminal output.

### Decision
Use OSC 133, the standard protocol for semantic shell integration.

### Consequences
- **Positive:** Compatible with Ghostty, Kitty, WezTerm, VS Code, iTerm2
- **Positive:** Shell integration scripts are portable across terminals
- **Positive:** Well-defined semantics (A=prompt start, B=prompt end, C=command start, D=command end)
- **Note:** Shell integration scripts are implemented for bash, zsh, and fish
- **Note:** Frontend parsing of OSC 133 sequences is designed but deferred

---

## Decision 6: Zustand for Frontend State

**Status:** Documented (technology-stack.md, section 2.9)

### Context
Need frontend state management for terminal session state (session ID, blocks, errors) and AI state (streaming status, conversations, request tracking).

### Decision
Use Zustand.

### Consequences
- **Positive:** Minimal boilerplate -- entire terminal store is 30 lines, AI store is 45 lines
- **Positive:** No providers or context wrappers needed
- **Positive:** Direct access via `useAIStore.getState()` outside React render cycle
- **Positive:** ~2KB bundle size vs ~12KB for Redux Toolkit
- **Positive:** Subscriptions work well for high-frequency streaming updates
- **Negative:** Less structured than Redux for very large state trees (not a concern at current scale)

---

## Decision 7: Manual SSE Parsing Over eventsource-client

**Status:** Inferred

### Context
The design documents (technology-stack.md) list `eventsource-client` as a dependency for SSE stream parsing. The actual implementation parses SSE manually.

### Decision
Parse Claude's SSE stream manually using line-by-line string processing on reqwest's `bytes_stream()`.

### Evidence
- `eventsource-client` is not in `Cargo.toml`
- `claude_api.rs` implements manual line-by-line SSE parsing (lines 64-114)
- `futures` crate is used for `StreamExt` to iterate over byte chunks

### Consequences
- **Positive:** No additional dependency
- **Positive:** Full control over buffering behavior
- **Positive:** Code is straightforward (76 lines of parsing)
- **Negative:** Must handle edge cases manually (partial lines at chunk boundaries)
- **Risk:** If Claude's SSE format changes, parsing must be updated manually

---

## Decision 8: In-Memory API Key Over Keychain

**Status:** Inferred

### Context
The design documents specify OS keychain storage via the `keyring` crate for API key persistence. The implementation stores the key in memory only.

### Decision
Store the API key in `Mutex<Option<String>>` managed by Tauri state. Load from `ANTHROPIC_API_KEY` environment variable at startup. Allow runtime updates via the Settings UI, but do not persist across restarts.

### Evidence
- `keyring` is not in `Cargo.toml`
- `lib.rs` line 25: `let api_key = std::env::var("ANTHROPIC_API_KEY").ok();`
- `config.rs` `store_api_key` updates the in-memory state only
- Settings UI note: "API key is stored in memory. Set ANTHROPIC_API_KEY env var for persistence."

### Consequences
- **Positive:** Simpler implementation, fewer dependencies
- **Positive:** Adequate for development and personal use
- **Negative:** Key must be re-entered after each app restart (unless env var is set)
- **Negative:** Less secure than OS keychain for desktop deployment

---

## Decision 9: Pragmatic Hexagonal Architecture (No Rust Traits)

**Status:** Inferred

### Context
The architecture design and component boundaries documents specify a full hexagonal architecture with Rust traits as port definitions and separate adapter implementations. The implementation uses a simplified approach.

### Decision
Organize code into `commands/` (primary adapters), `domain/` (business logic), and `ports/` (secondary adapters) directories, but implement ports as concrete structs and functions rather than Rust traits.

### Evidence
- No trait definitions exist in the codebase
- `ports/pty.rs` contains `struct PtyHandle` directly, not a `PtyPort` trait
- `ports/claude_api.rs` contains `async fn send_message_streaming()` directly, not a `ClaudeApiPort` trait
- No `adapters/` directory exists
- `commands/ai.rs` calls `claude_api::send_message_streaming()` directly, not through a trait object

### Consequences
- **Positive:** Faster implementation with less boilerplate
- **Positive:** Direct function calls are simpler to trace and debug
- **Positive:** The conceptual layering is preserved (domain logic is isolated)
- **Negative:** Cannot substitute mock implementations for testing
- **Negative:** Domain logic has compile-time dependencies on infrastructure code
- **Evolution:** Can be refactored to trait-based ports when testability becomes a priority

---

## Decision 10: Inline Styles Over Tailwind CSS

**Status:** Inferred

### Context
The technology-stack.md design document specifies Tailwind CSS v4 for styling. The implementation uses React inline styles exclusively.

### Decision
Use React inline style objects for all component styling.

### Evidence
- `tailwindcss` is not in `package.json` dependencies or devDependencies
- No `tailwind.config.*` file exists
- All components use `style={{ ... }}` props
- `AIPanel.tsx` defines `PANEL_STYLES` as a `Record<string, React.CSSProperties>` object
- `SettingsView.tsx` defines `STYLES` similarly

### Consequences
- **Positive:** Zero additional tooling or build configuration
- **Positive:** Styles are co-located with components
- **Negative:** No utility class reuse across components
- **Negative:** No responsive design utilities
- **Negative:** Style objects cannot use pseudo-selectors (`:hover`, `:focus`)

---

## Decision 11: Dedicated Thread for PTY Read Loop

**Status:** Inferred

### Context
The PTY read loop needs to continuously read from the pseudo-terminal master file descriptor and emit events. This could run on a Tokio async task or a dedicated OS thread.

### Decision
Use `std::thread::spawn()` for the PTY read loop.

### Evidence
- `commands/terminal.rs` line 57: `std::thread::spawn(move || { pty_read_loop(...) })`
- `PtyReader::read_chunk()` uses blocking `std::io::Read`

### Consequences
- **Positive:** Blocking I/O does not starve the Tokio async runtime
- **Positive:** Simple, straightforward implementation
- **Positive:** Each terminal session gets its own thread for isolation
- **Negative:** One OS thread per session (acceptable for single-session MVP)
- **Negative:** Thread terminates on error with no automatic restart

---

## Decision 12: Catppuccin Mocha Theme

**Status:** Inferred

### Context
The terminal and UI need a cohesive color scheme.

### Decision
Use the Catppuccin Mocha palette throughout the application.

### Evidence
- `useTerminal.ts`: terminal theme colors match Catppuccin Mocha exactly (e.g., background `#1e1e2e`, foreground `#cdd6f4`, red `#f38ba8`, green `#a6e3a1`, blue `#89b4fa`)
- `index.html`: body background `#1e1e2e`
- `AIPanel.tsx`: sidebar background `#181825` (Catppuccin Mocha Mantle)
- `SettingsView.tsx`: panel background `#1e1e2e`, success color `#a6e3a1`, error color `#f38ba8`

### Consequences
- **Positive:** Cohesive, modern dark theme across all UI surfaces
- **Positive:** Well-known palette with good contrast ratios
- **Negative:** No light theme option (design docs mention a `theme: "dark" | "light"` config, but only dark is implemented)
