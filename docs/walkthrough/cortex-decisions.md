# Cortex AI Terminal -- Design Decisions

Status: All decisions documented in design-phase documents (no implementation yet).
Date: 2026-03-22

---

## ADR-001: Application Framework -- Tauri v2

**Status:** Documented (architecture-design.md, technology-stack.md)

**Context:**
A desktop terminal emulator needs a native shell (windowing, process management) with a rich UI layer. Four approaches were evaluated: Tauri v2, Electron, full Rust with GPU rendering, and native Swift (macOS only). The research document analyzed 53 sources across all approaches.

**Decision:**
Use Tauri v2 with a React/TypeScript frontend and Rust backend.

**Rationale:**
- ~10 MB binary vs ~150 MB for Electron (15x smaller)
- Rust backend provides native-speed PTY management and HTTP streaming
- IPC supports both synchronous commands and async events
- Cross-platform (macOS, Linux, Windows) from single codebase
- 85k+ GitHub stars, active development

**Alternatives rejected:**
- **Electron:** 15x larger binary, higher memory overhead, Node.js intermediary adds complexity
- **Full Rust + WGPU:** 12-24+ months estimated effort for custom UI framework and terminal renderer
- **Native Swift (macOS only):** Not cross-platform, limits future audience

**Consequences:**
- Two-language complexity (TypeScript + Rust)
- Webview rendering performance limited compared to native GPU (mitigated by xterm.js WebGL)
- Tauri v2 ecosystem is younger than Electron's

---

## ADR-002: Terminal Emulation -- xterm.js with WebGL Renderer

**Status:** Documented (architecture-design.md, technology-stack.md)

**Context:**
The terminal emulator core needs full VT100/ANSI escape sequence parsing, Unicode support, IME input, accessibility, and efficient rendering. Building this from scratch is a multi-year effort.

**Decision:**
Use xterm.js (v5.x) with the WebGL renderer addon.

**Rationale:**
- Battle-tested: powers VS Code terminal, Tabby, Hyper, hundreds of web terminals
- Complete VT100/ANSI handling including Unicode, IME, and screen reader support
- WebGL renderer addon provides GPU-accelerated text rendering within the webview
- Rich addon ecosystem (fit, search, web-links, image support)

**Alternatives rejected:**
- **Custom WGPU renderer:** Multi-year effort for escape sequences, Unicode, font shaping, cursor, selection
- **alacritty_terminal crate:** Designed for native rendering, higher integration complexity with Tauri webview
- **Termwiz (WezTerm):** Less documented, tighter coupling to WezTerm's architecture

**Consequences:**
- Rendering performance bounded by webview (good, not excellent)
- Dependency on xterm.js maintenance and addon ecosystem
- Evolution path: hexagonal boundary allows future swap to native renderer

---

## ADR-003: PTY Management -- portable-pty

**Status:** Documented (technology-stack.md)

**Context:**
The terminal needs to spawn shell processes and manage bidirectional I/O through pseudo-terminal pairs. This must work cross-platform (Unix PTY on macOS/Linux, ConPTY on Windows).

**Decision:**
Use the portable-pty Rust crate.

**Rationale:**
- Created by the WezTerm author, production-proven
- Cross-platform: Unix PTY and Windows ConPTY
- 902K+ monthly downloads on crates.io
- Clean API: spawn shell, read/write master fd, resize
- Integrates naturally with the Rust backend

**Alternatives rejected:**
- **tauri-plugin-pty:** Less mature, fewer users, adds plugin dependency layer
- **node-pty:** Node.js library, unnecessary complexity in a Tauri app
- **Raw libc PTY calls:** Windows ConPTY handling is significant work

**Consequences:**
- Dependency on a single maintainer (WezTerm author)
- ConPTY edge cases on Windows may require workarounds

---

## ADR-004: AI API Integration -- reqwest + Manual SSE Parsing

**Status:** Documented (technology-stack.md, architecture-design.md)

**Context:**
The application needs to call the Claude Messages API with streaming responses. No official Anthropic Rust SDK exists. The API uses server-sent events (SSE) for streaming.

**Decision:**
Use reqwest for HTTP and eventsource-client for SSE parsing. Call the API from the Rust backend, not the frontend.

**Rationale:**
- No official Anthropic Rust SDK available
- Direct HTTP gives full control over timeouts, retries, error handling
- The API key must not be exposed to the webview JavaScript context
- The redaction engine runs in Rust before data leaves the process
- Backend can enforce rate limiting and cost controls

**Alternatives rejected:**
- **anthropic-rs (unofficial):** Risk of breaking changes or abandonment
- **TypeScript SDK from frontend:** API key exposed to webview, security violation
- **Calling API through a proxy:** Adds server infrastructure, reduces privacy

**Consequences:**
- Must implement/maintain the HTTP integration
- SSE parsing is additional code to maintain
- If Anthropic releases an official Rust SDK, migration is possible without frontend changes

---

## ADR-005: Streaming Callback Pattern

**Status:** Documented (component-boundaries.md)

**Context:**
AI responses stream via SSE. The frontend needs real-time text chunks via Tauri events. The question is which layer emits the Tauri events.

**Decision:**
The secondary adapter (AnthropicHttpAdapter) invokes a StreamingCallbacks trait. The primary adapter (TauriCommandAdapter) provides the callback implementation that emits Tauri events.

**Rationale:**
- Keeps infrastructure concerns (Tauri events) out of the secondary adapter
- The HTTP adapter has no dependency on Tauri (clean separation)
- Testing: mock callbacks verify streaming behavior without Tauri runtime

**Consequences:**
- Additional trait and indirection
- Callback trait must be Send + Sync for cross-task invocation
- Primary adapter creates callback impl per request

---

## ADR-006: Persistence -- SQLite via rusqlite

**Status:** Documented (technology-stack.md, data-models.md)

**Context:**
The application needs to persist configuration, command history, AI response cache, and chat conversations. Data volume is small (terminal history + cached AI responses).

**Decision:**
Use rusqlite with a single SQLite file in the OS-appropriate app data directory. WAL mode for crash safety.

**Rationale:**
- Zero-configuration embedded database, single file
- Handles all persistence needs without external dependencies
- WAL mode provides crash safety and concurrent read/write
- rusqlite is the most mature SQLite binding for Rust

**Alternatives rejected:**
- **sqlx:** Async-first, compile-time query checking is overkill for simple schema
- **sled:** Less mature, limited query capabilities
- **Plain JSON files:** No query capability, no transactional safety

**Consequences:**
- Schema migrations managed manually (schema_version table)
- SQLite access must be serialized (Tokio mutex or dedicated thread)
- Single-file database simplifies backup and portability

---

## ADR-007: Block Detection -- OSC 133 Protocol

**Status:** Documented (architecture-design.md, research doc)

**Context:**
To provide Warp-style "blocks" (discrete containers for each command execution), the terminal needs to detect where each command and its output begin and end.

**Decision:**
Use the OSC 133 escape sequence protocol with shell integration scripts for bash, zsh, and fish.

**Rationale:**
- Established standard: supported by Ghostty, Kitty, WezTerm, VS Code, iTerm2
- Shell integration scripts are portable across OSC 133-capable terminals
- Exit code tracking built into the protocol
- Prompt navigation and command selection are protocol features

**Consequences:**
- Requires shell integration scripts (distributed with the app, auto-injected)
- Users of non-standard shells must manually add integration
- Custom metadata requires additional OSC sequences (backward-compatible extension)

---

## ADR-008: API Key Storage -- OS System Keychain

**Status:** Documented (architecture-design.md, technology-stack.md)

**Context:**
The Anthropic API key grants access to a paid service. It must be stored securely on the user's machine.

**Decision:**
Store the API key in the OS system keychain using the keyring crate. Support ANTHROPIC_API_KEY environment variable as fallback.

**Rationale:**
- OS keychain provides hardware-backed encryption on modern systems
- No need to manage encryption keys (the recursive key-storage problem)
- Cross-platform: macOS Keychain, Windows Credential Manager, Linux Secret Service
- keyring crate: 1M+ downloads, simple API

**Alternatives rejected:**
- **Encrypted config file:** Must manage encryption key, recursive problem
- **Environment variable only:** Not user-friendly for desktop app

**Consequences:**
- API key never in config files, SQLite, or webview
- Keychain access may require user authentication on some systems
- Environment variable fallback for CI/automation users

---

## ADR-009: Security -- Redaction Engine

**Status:** Documented (architecture-design.md, component-boundaries.md)

**Context:**
Terminal context sent to the Claude API can contain secrets: API keys, passwords, private keys, connection strings, PII. This is a critical security concern documented in multiple sources.

**Decision:**
Implement a regex-based redaction engine that filters all content before it reaches the Claude API. Apply to command output, environment snippets, and any context included in AI prompts.

**Rationale:**
- Terminal output is a high-risk surface for secret exposure
- Regex patterns catch common secret formats reliably
- Defense in depth: redaction + user consent + opt-in context sharing
- Runs in Rust, before data leaves the process

**Consequences:**
- Regex patterns may have false positives (over-redaction) or false negatives (missed secrets)
- User-defined custom patterns provide escape valve for false positives/negatives
- Evolution: ML-based detection can supplement regex patterns

---

## ADR-010: Hexagonal Architecture

**Status:** Documented (architecture-design.md, component-boundaries.md)

**Context:**
The system integrates four distinct external systems (PTY, Claude API, SQLite, OS Keychain), each with different communication patterns. The team wants domain logic to be testable without infrastructure.

**Decision:**
Adopt hexagonal (ports and adapters) architecture. Domain logic depends on port traits. Each external system gets its own adapter behind a port interface.

**Rationale:**
- Domain logic (redaction, context building, block management) testable without infrastructure
- Adapters are swappable (mock for testing, alternative implementations for evolution)
- Clear module dependency rules (domain never imports from adapters)
- The frontend-backend boundary maps naturally to primary adapters

**Consequences:**
- More indirection (7 port traits to define and maintain)
- All external access goes through adapter implementations
- Testing can use in-memory/mock adapters
- Trade-off: architectural overhead is acceptable for a system with 4 external integrations

---

## ADR-011: Frontend State Management -- Zustand

**Status:** Documented (technology-stack.md)

**Context:**
The React frontend needs state management for terminal sessions, AI conversations, and configuration. State updates from Tauri events arrive asynchronously and frequently (PTY output).

**Decision:**
Use Zustand for frontend state management.

**Rationale:**
- Minimal boilerplate compared to Redux
- No providers or context wrappers needed
- Supports subscriptions (useful for streaming AI state updates)
- ~2 KB bundle size, TypeScript-first
- No re-render issues at scale (unlike React Context)

**Alternatives rejected:**
- **Redux Toolkit:** More boilerplate, larger bundle, overkill for this state complexity
- **Jotai:** Atomic model less intuitive for grouped state (sessions, conversations)
- **React Context:** Re-render issues with high-frequency terminal state updates

**Consequences:**
- Three stores: terminalStore, aiStore, configStore
- Stores are the source of truth; components read from stores via hooks

---

## ADR-012: AI Conversation Binding Rules

**Status:** Documented (component-boundaries.md)

**Context:**
The AI subsystem serves two interaction patterns: quick inline responses (command translation, error diagnosis) and persistent chat conversations. These have different lifecycle and persistence requirements.

**Decision:**
Inline AI operations (translate_command, explain_error, explain_command) are ephemeral -- scoped to a block, no conversation history, discarded on session end. Chat operations create/append to persistent conversations stored in SQLite.

**Rationale:**
- Inline responses are contextual and transient; persisting them adds noise
- Chat conversations benefit from history (resume, reference, search)
- Clear separation simplifies the data model and UI

**Consequences:**
- Two code paths for AI response handling (inline vs conversational)
- Inline responses not searchable after session ends
- Chat conversations stored in ai_conversations + ai_messages tables
