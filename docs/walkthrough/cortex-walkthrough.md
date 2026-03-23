---
marp: true
theme: default
paginate: true
backgroundColor: #1e1e2e
color: #cdd6f4
style: |
  section {
    font-family: 'Inter', 'SF Pro Display', -apple-system, sans-serif;
    background-color: #1e1e2e;
    color: #cdd6f4;
  }
  h1 {
    color: #89b4fa;
    border-bottom: 2px solid #45475a;
    padding-bottom: 8px;
  }
  h2 {
    color: #a6e3a1;
  }
  h3 {
    color: #f9e2af;
  }
  a {
    color: #89dceb;
  }
  code {
    background-color: #313244;
    color: #f38ba8;
    padding: 2px 6px;
    border-radius: 4px;
  }
  pre {
    background-color: #181825 !important;
    border: 1px solid #45475a;
    border-radius: 8px;
  }
  table {
    font-size: 0.85em;
    border-collapse: collapse;
  }
  th {
    background-color: #313244;
    color: #89b4fa;
  }
  td, th {
    border: 1px solid #45475a;
    padding: 6px 12px;
  }
  blockquote {
    border-left: 4px solid #f9e2af;
    background-color: #313244;
    padding: 12px 16px;
    margin: 12px 0;
    font-style: italic;
  }
  img {
    max-height: 480px;
  }
  .tag {
    display: inline-block;
    font-size: 0.65em;
    padding: 2px 8px;
    border-radius: 4px;
    margin-left: 8px;
    vertical-align: middle;
  }
  .tag-explanation {
    background-color: #89b4fa;
    color: #1e1e2e;
  }
  .tag-reference {
    background-color: #a6e3a1;
    color: #1e1e2e;
  }
  .tag-howto {
    background-color: #f9e2af;
    color: #1e1e2e;
  }
  .tier {
    display: inline-block;
    font-size: 0.6em;
    padding: 2px 6px;
    border-radius: 4px;
    background-color: #45475a;
    color: #bac2de;
    margin-left: 4px;
  }
  .decision-tag {
    display: inline-block;
    font-size: 0.7em;
    padding: 2px 8px;
    border-radius: 4px;
    margin-right: 6px;
  }
  .documented {
    background-color: #a6e3a1;
    color: #1e1e2e;
  }
  .inferred {
    background-color: #f9e2af;
    color: #1e1e2e;
  }
---

<!-- _class: lead -->
<!-- _paginate: false -->

# Cortex: AI Terminal

## System Architecture Walkthrough

A Warp-inspired terminal emulator with Claude AI integration

Built with Tauri v2 + xterm.js + portable-pty

---

<!-- _footer: "Section 1: Setting | Tier 1" -->

# What Problem Does Cortex Solve?

<span class="tag tag-explanation">Explanation</span>

Modern developers live in the terminal -- but terminals have not evolved to help them.

**The gap:**
- Terminal output is an undifferentiated wall of text
- Errors require manual copy-paste into search engines or chat tools
- Shell commands are a memorization exercise, hostile to newcomers
- Sending terminal context to external AI risks leaking secrets

**Cortex bridges this gap** by embedding Claude AI directly into a modern terminal emulator, using the developer's own API key -- no intermediary servers, no data collection.

<!--
Presenter notes: This is a design-phase project. No source code exists yet. All architecture is documented in 5 design docs totaling 2,585 lines. The project name "Cortex" references the cerebral cortex -- the thinking layer added on top of the primitive terminal.
-->

---

<!-- _footer: "Section 1: Setting | Tier 1" -->

# Who Is This For?

<span class="tag tag-explanation">Explanation</span>

**Primary user:** Developers who already use a terminal daily and want AI assistance without leaving their workflow.

**Key constraints:**
- User owns the AI relationship (their own Anthropic API key)
- No telemetry, no proxy servers, no data leaves the machine except to `api.anthropic.com`
- Terminal must work perfectly without AI (AI is additive, not mandatory)
- macOS first (MVP), then Linux and Windows

**Competitive landscape:** Warp terminal pioneered AI-in-terminal but requires account creation, proxies AI through their servers, and is closed-source. Cortex offers an open, private alternative.

---

<!-- _footer: "Section 1: Setting | Tier 1" -->

# System Context

<span class="tag tag-explanation">Explanation</span>

![System Context Diagram](diagrams/system-context.svg)

<!--
Presenter notes: The system boundary is the user's machine. The only external communication is HTTPS to api.anthropic.com. All PTY management, redaction, caching, and persistence happen locally. The OS keychain stores the API key -- it never touches SQLite or config files.
-->

---

<!-- _footer: "Section 2: Characters | Tier 1" -->

# Technology Stack at a Glance

<span class="tag tag-reference">Reference</span>

![Technology Stack](diagrams/technology-stack.svg)

<!--
Presenter notes: Two languages (TypeScript frontend, Rust backend) connected by Tauri IPC. The frontend handles rendering and UX; the backend handles PTY, API calls, security, and persistence. All external dependencies are MIT or Apache-2.0 licensed.
-->

---

<!-- _footer: "Section 2: Characters | Tier 1" -->

# Why This Stack?

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> Four approaches were evaluated in the research document (53 sources):

| Approach | Dev Speed | Performance | Binary Size |
|----------|-----------|-------------|-------------|
| **Tauri + xterm.js** | Excellent | Good | ~10 MB |
| Full Rust + GPU | Poor | Excellent | ~5 MB |
| Electron + xterm.js | Excellent | Fair | ~150 MB |
| Native Swift (macOS) | Good | Good | Excellent |

**Verdict:** AI integration is the differentiator, not raw rendering speed. Tauri + xterm.js maximizes development velocity while maintaining good performance.

**Migration path:** The hexagonal architecture allows swapping xterm.js for a native GPU renderer later without changing backend logic.

---

<!-- _footer: "Section 2: Characters | Tier 2" -->

# Key Technology Choices

<span class="tag tag-reference">Reference</span>

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Framework | Tauri v2 | 10 MB binary, Rust backend, cross-platform |
| Terminal | xterm.js + WebGL | Battle-tested (VS Code), full VT100 support |
| PTY | portable-pty | WezTerm-proven, cross-platform (Unix + ConPTY) |
| AI HTTP | reqwest + eventsource-client | No official Rust SDK; direct HTTP gives full control |
| Database | rusqlite (SQLite) | Zero-config, embedded, WAL mode for crash safety |
| Keychain | keyring crate | macOS Keychain, Windows Cred Manager, Linux D-Bus |
| State | Zustand | 2 KB, TypeScript-first, subscription-based |

<span class="decision-tag documented">Documented</span> All choices are documented with rejection rationale for alternatives.

---

<!-- _footer: "Section 3: Architecture | Tier 1" -->

# Hexagonal Architecture

<span class="tag tag-explanation">Explanation</span>

![Hexagonal Architecture](diagrams/hexagonal-architecture.svg)

<!--
Presenter notes: The hexagonal (ports and adapters) architecture is the central organizing principle. The domain core (Block Manager, Context Builder, Redaction Engine) depends on port traits, not concrete implementations. This means the core can be tested without infrastructure -- swap in mock adapters. The critical invariant: domain/ never imports from adapters/.
-->

---

<!-- _footer: "Section 3: Architecture | Tier 1" -->

# Why Hexagonal?

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> The architecture design document explicitly adopts ports and adapters.

**Context:** The system must integrate four distinct external systems (PTY, Claude API, SQLite, OS Keychain), each with different communication patterns (sync, streaming, file I/O, OS API).

**Decision:** Hexagonal architecture. Domain logic depends on port traits. Each external system gets its own adapter.

**Consequences:**
- Domain logic (redaction, context building, block management) is testable without infrastructure
- Adapters are swappable (e.g., replace SQLite with Postgres, or portable-pty with a mock)
- The frontend-backend boundary uses primary adapters (Tauri command handlers)
- Trade-off: more indirection, more trait definitions to maintain

---

<!-- _footer: "Section 3: Architecture | Tier 2" -->

# Component Boundaries

<span class="tag tag-reference">Reference</span>

![Component Boundaries](diagrams/component-boundary.svg)

<!--
Presenter notes: Note the clear separation. Frontend has 4 layers (Components -> Hooks -> Stores -> Lib). Backend has 4 modules (Commands -> Ports -> Domain -> Adapters). The Tauri IPC boundary sits between them with typed commands (invoke) and events (listen). The dependency rules are enforced by module structure: domain/ never imports from adapters/.
-->

---

<!-- _footer: "Section 3: Architecture | Tier 2" -->

# Three Primary Ports

<span class="tag tag-reference">Reference</span>

The application core exposes exactly three inbound ports:

**TerminalPort** -- Terminal session lifecycle
- `create_session`, `write_input`, `resize`, `get_blocks`, `close_session`
- Events: `pty:data`, `pty:exit`, `pty:error`

**AIAssistantPort** -- All AI operations
- `translate_command`, `explain_error`, `explain_command`, `chat`, `cancel_request`
- Events: `ai:stream-chunk`, `ai:stream-end`, `ai:error`

**ConfigPort** -- Application settings
- `get_config`, `set_config`, `store_api_key`, `has_api_key`, `delete_api_key`

Each port maps to a Tauri command namespace in the primary adapter.

---

<!-- _footer: "Section 3: Architecture | Tier 2" -->

# Four Secondary Ports

<span class="tag tag-reference">Reference</span>

The domain core depends on four outbound port abstractions:

| Port | Adapter | External System |
|------|---------|-----------------|
| **PtyPort** | PortablePtyAdapter | Shell process via PTY |
| **ClaudeApiPort** | AnthropicHttpAdapter | api.anthropic.com |
| **StoragePort** | SqliteAdapter | SQLite file |
| **KeychainPort** | SystemKeychainAdapter | OS keychain |

**Streaming callback pattern:** The `ClaudeApiPort` uses a callback trait (`StreamingCallbacks`) rather than returning data. The primary adapter (Tauri command handler) creates the callback impl that emits Tauri events. The secondary adapter (HTTP) invokes callbacks as SSE events arrive -- it never emits Tauri events directly.

This keeps infrastructure concerns (Tauri events) out of the secondary adapter layer.

---

<!-- _footer: "Section 3: Architecture | Tier 2" -->

# Backend Module Layout

<span class="tag tag-reference">Reference</span>

```
src-tauri/src/
  main.rs              -- Tauri setup, wires adapters to ports
  ports/               -- Port trait definitions (7 traits)
    terminal.rs, ai_assistant.rs, config.rs
    pty.rs, claude_api.rs, storage.rs, keychain.rs
  domain/              -- Core logic (NO external deps)
    block.rs           -- Block lifecycle, OSC 133 state machine
    context.rs         -- Shell state to AI prompt assembly
    redaction.rs       -- Regex-based secret filtering
    models.rs          -- Shared types (AppConfig, ShellContext)
  adapters/            -- Concrete port implementations
    pty_adapter.rs, anthropic_adapter.rs
    sqlite_adapter.rs, keychain_adapter.rs
  commands/            -- Tauri IPC handlers (primary adapter)
    terminal_commands.rs, ai_commands.rs, config_commands.rs
```

**Dependency rule:** `domain/` depends on `ports/` (traits). `adapters/` implements `ports/`. `domain/` never imports from `adapters/`.

---

<!-- _footer: "Section 3: Architecture | Tier 2" -->

# Frontend Module Layout

<span class="tag tag-reference">Reference</span>

```
src/
  App.tsx                    -- Layout: terminal + AI panel + settings
  components/
    terminal/                -- TerminalView, BlockOverlay, BlockActions
    ai/                      -- AIPanel, ChatMessage, CommandSuggestion
    settings/                -- SettingsView, ApiKeyInput, ModelSelector
    shared/                  -- LoadingIndicator, ErrorBanner
  stores/                    -- Zustand state
    terminalStore.ts         -- Sessions, blocks
    aiStore.ts               -- Conversations, streaming status
    configStore.ts           -- App configuration
  hooks/
    useTerminal.ts           -- PTY lifecycle, event listeners
    useAI.ts                 -- AI operations, streaming
    useConfig.ts             -- Settings management
  lib/
    osc133.ts                -- OSC 133 sequence parser
    tauri-bridge.ts          -- Typed IPC wrappers
    types.ts                 -- Shared TypeScript types
```

**Dependency flow:** Components -> Hooks -> Stores -> Lib (unidirectional)

---

<!-- _footer: "Section 4: Data Flows | Tier 1" -->

# Data Flow: Command Execution

<span class="tag tag-explanation">Explanation</span>

![Command Execution Flow](diagrams/data-flow-command.svg)

<!--
Presenter notes: The key insight is the OSC 133 protocol. Shell integration scripts emit escape sequences at prompt/command boundaries. The frontend parses these to create "blocks" -- discrete command execution units. This is the same protocol used by Ghostty, Kitty, WezTerm, VS Code, and iTerm2. Each block tracks: prompt, command, output, exit code, duration.
-->

---

<!-- _footer: "Section 4: Data Flows | Tier 1" -->

# Data Flow: AI Command Translation

<span class="tag tag-explanation">Explanation</span>

![AI Translation Flow](diagrams/data-flow-ai-translation.svg)

<!--
Presenter notes: Critical path: user query -> context builder -> redaction engine -> Claude API -> streaming response. The redaction engine applies regex patterns BEFORE data leaves the process. The streaming callback pattern keeps Tauri event emission in the primary adapter layer. The secondary adapter (HTTP) never emits Tauri events directly.
-->

---

<!-- _footer: "Section 4: Data Flows | Tier 2" -->

# Data Flow: PTY Error Recovery

<span class="tag tag-explanation">Explanation</span>

![PTY Error Recovery Flow](diagrams/data-flow-pty-error.svg)

<!--
Presenter notes: Three failure modes are handled explicitly: spawn_failure (shell not found), read_io_error (I/O failure mid-session), and process_crash (SIGSEGV, SIGKILL). In all cases, pending AI requests are cancelled, incomplete blocks are finalized, history is flushed, and resources are cleaned up. The user sees an error banner with "Close" or "Reopen" options.
-->

---

<!-- _footer: "Section 4: Data Flows | Tier 2" -->

# AI Conversation Model

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> Two distinct AI interaction patterns:

**Inline (ephemeral):**
- `translate_command`, `explain_error`, `explain_command`
- Scoped to a single block, rendered in-place
- No conversation history, discarded on session end
- No SQLite persistence

**Conversational (persistent):**
- `chat` operations in the AI sidebar
- Full conversation history, stored in SQLite
- Survives across sessions, can be resumed
- Creates/appends to `ai_conversations` + `ai_messages` tables

**Model routing:**
- Claude Haiku for fast completions (~200ms first token, $0.80/M input)
- Claude Sonnet for command translation, error diagnosis ($3/M input)
- User-configurable default model

---

<!-- _footer: "Section 5: Data Model | Tier 2" -->

# Data Model: SQLite Schema

<span class="tag tag-reference">Reference</span>

![Data Model](diagrams/data-model.svg)

<!--
Presenter notes: Six tables total. config is key-value. command_history stores completed blocks (truncated to 200 lines max). ai_cache provides content-addressable caching for explain_command responses. ai_conversations and ai_messages store chat sidebar history. schema_version tracks migrations. All tables use TEXT for timestamps (ISO 8601) and UUIDs.
-->

---

<!-- _footer: "Section 5: Data Model | Tier 2" -->

# Domain Data Structures

<span class="tag tag-reference">Reference</span>

| Structure | Purpose | Key Fields |
|-----------|---------|------------|
| **Block** | Single command execution cycle | command, output, exit_code, state (prompting/running/completed) |
| **TerminalSession** | Active PTY session | shell_path, pid, rows, cols, blocks[] |
| **ShellContext** | AI prompt context | shell_type, os, cwd, recent_commands[] |
| **CommandSuggestion** | AI-generated command | command, explanation, risk_level |
| **AIConversation** | Chat sidebar thread | title, messages[] |
| **AppConfig** | Application settings | default_shell, default_model, theme, redaction patterns |

**Block state machine:** `prompting` (OSC 133 A) -> `running` (OSC 133 C) -> `completed` (OSC 133 D)

Blocks live in Zustand during a session. Completed blocks persist to `command_history`.

---

<!-- _footer: "Section 5: Data Model | Tier 2" -->

# Storage and Persistence

<span class="tag tag-reference">Reference</span>

| Data | Location | Retention |
|------|----------|-----------|
| SQLite database | `~/Library/Application Support/ai-terminal/data.db` | Permanent |
| API key | macOS Keychain | Until deleted |
| Shell integration scripts | Embedded in app binary | App lifecycle |
| Logs | `{app_data_dir}/logs/` | Configurable |
| Active session blocks | Frontend memory (Zustand) | Session lifetime |
| AI streaming state | Frontend memory | Request lifetime |

**Storage policies (MVP):**
- Command history: last 10,000 entries, output truncated to 200 lines
- AI cache: max 1,000 entries, LRU eviction
- History pruning: entries older than 90 days removed at startup
- SQLite WAL mode for crash safety and concurrent read/write

---

<!-- _footer: "Section 6: Security | Tier 1" -->

# Security Architecture

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> Security is a first-class concern -- terminal context contains secrets.

**Threat boundary:** All processing is local. Only HTTPS to `api.anthropic.com` crosses the boundary.

**Defense layers:**
1. **Redaction Engine** -- Regex patterns filter secrets before API calls
2. **System Keychain** -- API key never in config files, never in SQLite, never in webview
3. **No intermediary servers** -- Direct client-to-API communication
4. **HTTPS only** -- TLS 1.3 to api.anthropic.com
5. **Opt-in context** -- User controls what context is sent to AI

**Redacted patterns (MVP):** API keys, AWS credentials, PEM private keys, connection strings, email addresses, user-defined custom patterns.

---

<!-- _footer: "Section 6: Security | Tier 2" -->

# API Key Lifecycle

<span class="tag tag-howto">How-To</span>

<span class="decision-tag documented">Documented</span> The API key is the user's direct access to a paid service.

**Lifecycle:**
1. **First run:** Settings UI prompts for API key
2. **Validation:** Test call to Anthropic API confirms key works
3. **Storage:** System keychain via `keyring` crate
4. **Runtime:** Retrieved from keychain at launch, held in Rust memory
5. **Security invariants:**
   - Never logged to file
   - Never written to SQLite
   - Never exposed to webview JavaScript context
   - Never sent to any server except api.anthropic.com

**Fallback:** `ANTHROPIC_API_KEY` environment variable for CI/advanced users.

**Why not encrypted config file?** The encryption key storage problem is recursive. OS keychain solves it natively.

---

<!-- _footer: "Section 6: Security | Tier 2" -->

# Redaction Engine

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> All content passes through redaction before reaching the Claude API.

**MVP patterns:**

| Pattern | Example Matched |
|---------|----------------|
| API keys/tokens | `API_KEY=sk-abc123...` |
| Bearer tokens | `Authorization: Bearer eyJ...` |
| AWS credentials | `AWS_SECRET_ACCESS_KEY=AKIA...` |
| Private keys | `-----BEGIN RSA PRIVATE KEY-----` |
| Connection strings | `postgres://user:pass@host/db` |
| Email addresses | `user@company.com` |
| Custom patterns | User-defined regex in settings |

**Evolution:** ML-based secret detection, configurable sensitivity levels, audit log of what was redacted.

---

<!-- _footer: "Section 7: Design Decisions | Tier 1" -->

# Key Design Decisions Summary

<span class="tag tag-explanation">Explanation</span>

| Decision | Context | Status |
|----------|---------|--------|
| Tauri v2 over Electron | Binary size, memory, Rust backend | <span class="decision-tag documented">Documented</span> |
| xterm.js over custom renderer | Dev speed vs rendering perf | <span class="decision-tag documented">Documented</span> |
| Claude API from Rust, not JS | API key security, redaction in Rust | <span class="decision-tag documented">Documented</span> |
| SQLite over complex stores | Zero-config, embedded, small data volume | <span class="decision-tag documented">Documented</span> |
| OSC 133 for blocks | Industry standard protocol | <span class="decision-tag documented">Documented</span> |
| System keychain for API key | OS-native credential security | <span class="decision-tag documented">Documented</span> |
| Hexagonal architecture | Testability, adapter swappability | <span class="decision-tag documented">Documented</span> |

All 7 decisions are documented with rejection rationale in the design documents.

---

<!-- _footer: "Section 7: Design Decisions | Tier 2" -->

# Decision Deep-Dive: Claude API from Rust

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span>

**Context:** The Claude API could be called from the frontend (TypeScript) or the backend (Rust). No official Anthropic Rust SDK exists.

**Decision:** Call the API from the Rust backend using `reqwest` + `eventsource-client`.

**Rationale:**
- The API key must not be exposed to the webview JavaScript context
- The redaction engine runs in Rust before data leaves the process
- Rust's `reqwest`/`hyper` provides robust streaming HTTP with proper error handling
- The backend can enforce rate limiting and cost controls

**Consequences:**
- Must implement SSE parsing manually (or use eventsource-client crate)
- No official SDK means building and maintaining the HTTP integration
- If Anthropic releases an official Rust SDK, it can be adopted without frontend changes

**Trade-off accepted:** More implementation work for stronger security guarantees.

---

<!-- _footer: "Section 7: Design Decisions | Tier 2" -->

# Decision Deep-Dive: Streaming Callback Pattern

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span>

**Context:** AI responses stream via SSE. The frontend needs real-time text chunks via Tauri events. But who emits the events?

**Decision:** The secondary adapter (AnthropicHttpAdapter) invokes a `StreamingCallbacks` trait. The primary adapter (TauriCommandAdapter) provides the callback implementation that emits Tauri events.

```rust
trait StreamingCallbacks: Send + Sync {
    fn on_chunk(&self, text: &str);
    fn on_tool_use(&self, tool_name: &str, input: Value);
    fn on_completion(&self, metadata: StreamMetadata);
    fn on_error(&self, error: ApiError);
}
```

**Consequences:**
- The HTTP adapter has no dependency on Tauri (clean secondary adapter)
- The callback impl in the command handler bridges HTTP events to Tauri events
- Testing: mock callbacks verify streaming behavior without Tauri runtime

---

<!-- _footer: "Section 7: Design Decisions | Tier 2" -->

# Decision Deep-Dive: Block Architecture

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span>

**Context:** Traditional terminals show an undifferentiated stream of text. Warp pioneered "blocks" -- discrete containers for each command execution.

**Decision:** Adopt OSC 133 protocol for block boundary detection.

**Why OSC 133 (not custom protocol):**
- Established standard: Ghostty, Kitty, WezTerm, VS Code, iTerm2
- Shell integration scripts are portable across terminals
- Exit code tracking built into the protocol (OSC 133 D with exit code)
- Prompt navigation and command selection for free

**Block lifecycle:**
1. `OSC 133;A` -- Prompt start: create block, state = `prompting`
2. `OSC 133;B` -- Command start: transition to input capture
3. `OSC 133;C` -- Command executed: state = `running`
4. `OSC 133;D;N` -- Command finished: state = `completed`, exit_code = N

**Evolution:** Custom OSC sequences for richer metadata (git branch, timing breakdown) without breaking standard compatibility.

---

<!-- _footer: "Section 8: Quality Attributes | Tier 2" -->

# Quality Attributes

<span class="tag tag-reference">Reference</span>

| Attribute | Strategy |
|-----------|----------|
| **Performance** | xterm.js WebGL renderer. Async PTY I/O on dedicated Tokio task. Streaming AI responses. Debounced completions (300-500ms). |
| **Security** | Redaction engine on all AI-bound content. API key in system keychain. No intermediary servers. HTTPS only. |
| **Reliability** | Graceful degradation when API is unavailable. Explicit PTY error handling (3 failure modes). SQLite WAL mode. |
| **Maintainability** | Hexagonal architecture. Module dependency rules. Domain testable without infrastructure. |
| **Usability** | Familiar terminal behavior (xterm.js). Progressive AI disclosure. Terminal works without AI. |

---

<!-- _footer: "Section 8: Quality Attributes | Tier 2" -->

# Concurrency Model

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag inferred">Inferred</span> from Tokio usage and architecture patterns described in the design docs.

**Backend (Rust + Tokio):**
- One Tokio task per PTY session (reads PTY master in a loop, emits events)
- One Tokio task per AI streaming request (reads SSE stream, invokes callbacks)
- SQLite access serialized through Tokio mutex or dedicated thread (WAL mode)
- Tauri command handlers run on the Tauri async runtime

**Frontend (JavaScript, single-threaded):**
- Zustand subscriptions trigger React re-renders
- Tauri event listeners update stores asynchronously
- xterm.js handles its own rendering loop (requestAnimationFrame)

**No shared mutable state between PTY and AI tasks** -- they communicate through the frontend (event-driven).

---

<!-- _footer: "Section 9: Implementation Roadmap | Tier 1" -->

# Implementation Roadmap

<span class="tag tag-howto">How-To</span>

<span class="decision-tag documented">Documented</span> Four phases, 16 weeks estimated.

| Phase | Weeks | Deliverable |
|-------|-------|-------------|
| **1. Core Terminal** | 1-4 | Working terminal: Tauri + xterm.js + portable-pty |
| **2. Shell Integration** | 5-8 | OSC 133 blocks, shell scripts, block actions |
| **3. Claude AI** | 9-12 | API client, redaction, translate/explain/chat |
| **4. Polish** | 13-16 | Completions, caching, themes, command palette |

**Phase 1 result:** A basic terminal emulator equivalent to Hyper.
**Phase 2 result:** A terminal with Warp-style blocks.
**Phase 3 result:** AI-powered terminal MVP.
**Phase 4 result:** Feature-complete v1.0.

---

<!-- _footer: "Section 9: Implementation Roadmap | Tier 2" -->

# Phase 1: Core Terminal (Weeks 1-4)

<span class="tag tag-howto">How-To</span>

**Goal:** A working terminal emulator.

1. Initialize Tauri v2 project with React + Vite frontend
2. Integrate xterm.js with WebGL renderer addon
3. Implement PTY management with portable-pty
4. Wire IPC: frontend xterm.js <-> Tauri commands/events <-> PTY
5. Handle terminal resize (SIGWINCH via pty.resize)
6. Basic input/output with escape sequence handling

**Key files to create first:**
- `src-tauri/src/ports/pty.rs` -- PtyPort trait
- `src-tauri/src/adapters/pty_adapter.rs` -- portable-pty implementation
- `src-tauri/src/commands/terminal_commands.rs` -- Tauri command handlers
- `src/components/terminal/TerminalView.tsx` -- xterm.js wrapper
- `src/hooks/useTerminal.ts` -- PTY lifecycle hook

---

<!-- _footer: "Section 9: Implementation Roadmap | Tier 2" -->

# Phase 3: Claude AI Integration (Weeks 9-12)

<span class="tag tag-howto">How-To</span>

**Goal:** AI-powered terminal MVP.

1. Implement API key management (keychain store + settings UI)
2. Build `AnthropicHttpAdapter` with reqwest + SSE parsing
3. Implement `StreamingCallbacks` trait and Tauri event bridge
4. Build `Redaction Engine` with MVP regex patterns
5. Implement `Context Builder` (shell state -> AI prompt)
6. Build `translate_command` flow (NL -> shell command with confirmation)
7. Build `explain_error` flow (failed block -> AI diagnosis)
8. Build AI chat sidebar with conversation persistence

**Cost control (MVP):**
- Route to Haiku for completions, Sonnet for complex tasks
- Debounce completion requests (300-500ms)
- Local LRU cache for explain_command responses
- User-configurable model selection

---

<!-- _footer: "Section 10: Risks | Tier 1" -->

# Risks and Open Questions

<span class="tag tag-explanation">Explanation</span>

| Risk | Severity | Mitigation |
|------|----------|------------|
| xterm.js WebGL perf vs native GPU | Medium | Benchmark during Phase 1; hexagonal boundary allows future swap |
| No official Anthropic Rust SDK | Medium | reqwest + manual SSE parsing; monitor for official SDK release |
| Windows ConPTY edge cases | Medium | portable-pty abstracts most issues; thorough Windows testing |
| API rate limits for individual keys | Medium | Aggressive debouncing, local caching, graceful degradation |
| Long session memory growth | Medium | Scrollback limits, block GC, configurable retention |

**Knowledge gaps identified in research:**
- Warp's exact AI prompt architecture (must design from first principles)
- Quantitative xterm.js WebGL vs native GPU performance comparison
- Claude API rate limits for individual (non-enterprise) subscriptions

---

<!-- _footer: "Section 10: Risks | Tier 2" -->

# Evolution Paths

<span class="tag tag-explanation">Explanation</span>

<span class="decision-tag documented">Documented</span> The design documents mark evolution points throughout.

**Near-term:**
- Multiple concurrent sessions (tabs)
- Session restore on app restart
- Full-text search on command history
- Structured logging to file for debugging

**Medium-term:**
- CI/CD pipeline with GitHub Actions (macOS, Linux, Windows builds)
- ML-based secret detection (beyond regex)
- Conversation branching and search
- Performance monitoring and opt-in telemetry

**Long-term:**
- Native GPU renderer (replace xterm.js WebGL if perf becomes bottleneck)
- Plugin system for custom AI tools
- Collaborative features (shared sessions)
- Data export/import and optional cloud sync

---

<!-- _footer: "Appendix" -->

# IPC Contract Reference

<span class="tag tag-reference">Reference</span>

**Tauri Commands (Frontend -> Backend):**

| Command | Returns |
|---------|---------|
| `create_terminal_session` | `{ session_id }` |
| `write_to_pty` | void |
| `resize_pty` | void |
| `ai_translate_command` | `{ request_id }` |
| `ai_explain_error` | `{ request_id }` |
| `ai_chat` | `{ request_id, conversation_id }` |
| `get_config` / `set_config` | `AppConfig` / void |
| `store_api_key` | `{ valid, error? }` |

**Tauri Events (Backend -> Frontend):**
`pty:data`, `pty:exit`, `pty:error`, `ai:stream-chunk`, `ai:stream-end`, `ai:tool-use`, `ai:error`

---

<!-- _footer: "Appendix" -->

# AI Features Summary

<span class="tag tag-reference">Reference</span>

| Feature | Trigger | Model | Persistence |
|---------|---------|-------|-------------|
| **NL to Command** | Hotkey + natural language | Sonnet | None (inline) |
| **Error Diagnosis** | Click on failed block | Sonnet | None (inline) |
| **Command Explain** | Select command + action | Sonnet (cached) | ai_cache table |
| **AI Chat** | Sidebar panel | Sonnet/Opus | ai_conversations |
| **Completions** | Ghost text while typing | Haiku | None |

**All features** go through the same pipeline: Context Builder -> Redaction Engine -> Claude API -> Streaming Response.

**Cost optimization:** Different models for different features. Local caching for deterministic responses. Debounced requests. User budget controls.

---

<!-- _class: lead -->
<!-- _paginate: false -->

# End of Walkthrough

**Project:** Cortex AI Terminal
**Status:** Architecture and design phase (no source code yet)
**Documentation:** 2,585 lines across 5 design documents, 53 research sources
**Architecture:** Hexagonal (Ports and Adapters)
**Stack:** Tauri v2 + React + xterm.js + Rust + portable-pty + Claude API

Next step: Phase 1 implementation -- Core Terminal (Weeks 1-4)
