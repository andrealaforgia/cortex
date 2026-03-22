# AI Terminal - Component Boundaries

**Status:** Approved
**Date:** 2026-03-19 (reviewed 2026-03-22)

---

## 1. Hexagonal Port Definitions

### 1.1 Primary Ports (Driving / Inbound)

These ports define how the outside world drives the application core.

#### TerminalPort

Handles all terminal session lifecycle and I/O.

| Operation | Input | Output | Sync/Async |
|-----------|-------|--------|------------|
| create_session | shell_path, env_vars, working_dir, size | session_id | Sync |
| write_input | session_id, bytes | void | Sync |
| resize | session_id, rows, cols | void | Sync |
| get_blocks | session_id | list of Block | Sync |
| close_session | session_id | void | Sync |

Events emitted by this port (backend -> frontend):
- `pty:data(session_id, bytes)` -- terminal output
- `pty:exit(session_id, exit_code)` -- shell process exited
- `pty:error(session_id, error_type, message)` -- PTY read/write failure

#### AIAssistantPort

Handles all AI-related operations.

| Operation | Input | Output | Sync/Async |
|-----------|-------|--------|------------|
| translate_command | natural_language_query, shell_context | request_id (inline response, no conversation) | Async (streamed) |
| explain_error | command, output, exit_code, shell_context | request_id (inline response, no conversation) | Async (streamed) |
| explain_command | command_text | request_id (inline response, no conversation) | Async (streamed) |
| chat | message, conversation_id?, shell_context | { request_id, conversation_id } (creates or appends to conversation) | Async (streamed) |
| cancel_request | request_id | void | Sync |

**Conversation binding rules:**
- `translate_command`, `explain_error`, `explain_command`: Inline, ephemeral responses scoped to a block. Do NOT create conversation entries. Responses are rendered in-place near the relevant block and are discarded when the session ends.
- `chat`: Persistent conversation. If `conversation_id` is null, creates a new conversation and returns its ID. If provided, appends to the existing conversation. Conversations are stored in SQLite and survive across sessions.

Events emitted:
- `ai:stream-chunk(request_id, text)` -- incremental AI text
- `ai:stream-end(request_id, metadata)` -- response complete
- `ai:error(request_id, error_details)` -- API error

#### ConfigPort

Manages application settings.

| Operation | Input | Output | Sync/Async |
|-----------|-------|--------|------------|
| get_config | void | AppConfig | Sync |
| set_config | partial AppConfig | void | Sync |
| store_api_key | api_key_string | validation_result | Sync |
| has_api_key | void | bool | Sync |
| delete_api_key | void | void | Sync |

---

### 1.2 Secondary Ports (Driven / Outbound)

These ports define what the application core needs from external systems. The core depends on these abstractions, not on concrete implementations.

#### PtyPort

Interface to the pseudo-terminal system.

| Operation | Description |
|-----------|-------------|
| open_pty | Create a new PTY pair with specified size |
| spawn_command | Launch a shell process on the PTY slave |
| read_output | Read bytes from the PTY master (blocking or async) |
| write_input | Write bytes to the PTY master |
| resize | Change PTY dimensions and signal SIGWINCH |
| kill | Terminate the child shell process |

#### ClaudeApiPort

Interface to the Anthropic Claude API. Streaming responses flow through a callback trait, keeping Tauri event emission in the primary adapter (command handler) rather than leaking infrastructure into the secondary adapter.

| Operation | Description |
|-----------|-------------|
| send_message_streaming | Send a messages request with streaming callbacks |
| send_message_with_tools | Send a messages request with tool definitions and streaming callbacks |
| validate_api_key | Test an API key with a minimal request |

**Streaming Callback Trait:**

```rust
trait StreamingCallbacks: Send + Sync {
    fn on_chunk(&self, text: &str);
    fn on_tool_use(&self, tool_name: &str, input: serde_json::Value);
    fn on_completion(&self, metadata: StreamMetadata);
    fn on_error(&self, error: ApiError);
}

struct StreamMetadata {
    model: String,
    input_tokens: u32,
    output_tokens: u32,
    stop_reason: String,
}
```

The TauriCommandAdapter (primary adapter) creates a callback implementation that emits Tauri events (`ai:stream-chunk`, `ai:stream-end`, etc.). The AnthropicHttpAdapter (secondary adapter) receives and invokes these callbacks as SSE events arrive — it never emits Tauri events directly.

Configuration exposed: model name, max_tokens, temperature.

#### StoragePort

Interface to persistent storage.

| Operation | Description |
|-----------|-------------|
| save_config | Persist application configuration |
| load_config | Retrieve application configuration |
| save_command_history | Store a completed command block |
| query_command_history | Retrieve command history with filters (time range, search term) |
| save_ai_cache_entry | Cache an AI response keyed by prompt hash |
| get_ai_cache_entry | Retrieve cached AI response by prompt hash |
| prune_old_entries | Remove entries older than a threshold |

#### KeychainPort

Interface to the OS credential store.

| Operation | Description |
|-----------|-------------|
| store_secret | Store a named secret (API key) |
| retrieve_secret | Retrieve a named secret |
| delete_secret | Remove a named secret |

---

## 2. Adapter Implementations

### 2.1 Secondary Adapters (Driven Side)

| Port | Adapter | Crate/Library | Notes |
|------|---------|---------------|-------|
| PtyPort | PortablePtyAdapter | portable-pty | Spawns shell, manages master/slave fd pair |
| ClaudeApiPort | AnthropicHttpAdapter | reqwest + eventsource-client | HTTPS to api.anthropic.com, SSE streaming |
| StoragePort | SqliteAdapter | rusqlite | Single SQLite file in app data dir |
| KeychainPort | SystemKeychainAdapter | keyring | macOS Keychain, Win Cred Manager, Linux Secret Service |

EVOLUTION: Test adapters (InMemoryStorage, MockPty, MockClaudeApi) for unit testing the core domain logic without infrastructure.

### 2.2 Primary Adapters (Driving Side)

| Port | Adapter | Technology | Notes |
|------|---------|------------|-------|
| TerminalPort | TauriCommandAdapter | Tauri `#[command]` + events | Maps Tauri IPC commands/events to port operations |
| AIAssistantPort | TauriCommandAdapter | Tauri `#[command]` + events | Same adapter, different command namespace |
| ConfigPort | TauriCommandAdapter | Tauri `#[command]` | Synchronous config operations |

The Tauri command adapter is the single bridge between the web frontend and the Rust core. It translates IPC calls into port operations and emits events back.

---

## 3. Rust Backend Module Boundaries

```
src-tauri/src/
|-- main.rs                    # Tauri app setup, plugin registration, state init
|-- lib.rs                     # Re-exports, shared types
|
|-- ports/                     # Port trait definitions (interfaces)
|   |-- mod.rs
|   |-- terminal.rs            # TerminalPort trait
|   |-- ai_assistant.rs        # AIAssistantPort trait
|   |-- config.rs              # ConfigPort trait
|   |-- pty.rs                 # PtyPort trait
|   |-- claude_api.rs          # ClaudeApiPort trait
|   |-- storage.rs             # StoragePort trait
|   |-- keychain.rs            # KeychainPort trait
|
|-- domain/                    # Core business logic (no external dependencies)
|   |-- mod.rs
|   |-- block.rs               # Block data structure and lifecycle
|   |-- context.rs             # Context builder (shell state -> AI prompt)
|   |-- redaction.rs           # Secret redaction engine
|   |-- models.rs              # Shared domain types (AppConfig, ShellContext, etc.)
|
|-- adapters/                  # Concrete port implementations
|   |-- mod.rs
|   |-- pty_adapter.rs         # PortablePtyAdapter (PtyPort impl)
|   |-- anthropic_adapter.rs   # AnthropicHttpAdapter (ClaudeApiPort impl)
|   |-- sqlite_adapter.rs      # SqliteAdapter (StoragePort impl)
|   |-- keychain_adapter.rs    # SystemKeychainAdapter (KeychainPort impl)
|
|-- commands/                  # Tauri IPC command handlers (primary adapter)
|   |-- mod.rs
|   |-- terminal_commands.rs   # create_session, write_input, resize, etc.
|   |-- ai_commands.rs         # translate_command, explain_error, chat, etc.
|   |-- config_commands.rs     # get_config, set_config, store_api_key, etc.
|
|-- shell_integration/         # Shell integration script resources
|   |-- mod.rs
|   |-- scripts/
|       |-- bash-integration.sh
|       |-- zsh-integration.zsh
|       |-- fish-integration.fish
```

### Module Dependency Rules

- `ports/` depends on: `domain/models` (shared types only)
- `domain/` depends on: `ports/` (trait definitions for driven ports)
- `adapters/` depends on: `ports/` (implements traits), external crates
- `commands/` depends on: `ports/` (uses trait objects), `domain/` (constructs domain objects)
- `main.rs` depends on: everything (wires adapters to ports, registers commands)

The critical invariant: **domain/ never imports from adapters/**. Domain logic is tested without infrastructure.

---

## 4. React Frontend Module Boundaries

```
src/
|-- main.tsx                   # React root, Tauri event listeners
|-- App.tsx                    # Layout: terminal + AI panel + settings
|
|-- components/
|   |-- terminal/
|   |   |-- TerminalView.tsx   # xterm.js instance wrapper
|   |   |-- BlockOverlay.tsx   # Renders block boundaries over xterm.js output
|   |   |-- BlockActions.tsx   # Copy, collapse, diagnose actions per block
|   |
|   |-- ai/
|   |   |-- AIPanel.tsx        # Chat sidebar container
|   |   |-- ChatMessage.tsx    # Single chat message (user or AI)
|   |   |-- CommandSuggestion.tsx  # Suggested command with confirm/reject
|   |   |-- StreamingText.tsx  # Renders AI text as it streams in
|   |
|   |-- settings/
|   |   |-- SettingsView.tsx   # Settings panel
|   |   |-- ApiKeyInput.tsx    # API key entry + validation
|   |   |-- ModelSelector.tsx  # Choose default AI model
|   |
|   |-- shared/
|       |-- LoadingIndicator.tsx
|       |-- ErrorBanner.tsx
|
|-- stores/
|   |-- terminalStore.ts       # Terminal session state, block list
|   |-- aiStore.ts             # AI conversation state, streaming status
|   |-- configStore.ts         # App configuration state
|
|-- hooks/
|   |-- useTerminal.ts         # Terminal lifecycle: create session, attach xterm, listen for pty:data
|   |-- useAI.ts               # AI operations: invoke commands, listen for ai:stream-*
|   |-- useConfig.ts           # Config operations: load, save, API key management
|
|-- lib/
|   |-- osc133.ts              # OSC 133 sequence parser for block boundary detection
|   |-- tauri-bridge.ts        # Typed wrappers around Tauri invoke/listen calls
|   |-- types.ts               # Shared TypeScript types (Block, AIMessage, AppConfig, etc.)
```

### Frontend Dependency Rules

- `stores/` depends on: `lib/types` (data types)
- `hooks/` depends on: `stores/` (state management), `lib/tauri-bridge` (IPC calls)
- `components/` depends on: `hooks/` (behavior), `stores/` (state reading)
- `lib/` depends on: nothing internal (utility layer)

---

## 5. Interface Contracts Between Frontend and Backend

### 5.1 Tauri Commands (Frontend -> Backend)

All commands are invoked via `@tauri-apps/api` invoke function. Return types are JSON-serialized Rust structs.

| Command | Parameters | Returns |
|---------|-----------|---------|
| `create_terminal_session` | `{ shell?: string, cwd?: string }` | `{ session_id: string }` |
| `write_to_pty` | `{ session_id: string, data: number[] }` | `void` |
| `resize_pty` | `{ session_id: string, rows: number, cols: number }` | `void` |
| `close_terminal_session` | `{ session_id: string }` | `void` |
| `ai_translate_command` | `{ query: string, context: ShellContext }` | `{ request_id: string }` |
| `ai_explain_error` | `{ command: string, output: string, exit_code: number, context: ShellContext }` | `{ request_id: string }` |
| `ai_explain_command` | `{ command: string }` | `{ request_id: string }` |
| `ai_chat` | `{ message: string, conversation_id?: string, context: ShellContext }` | `{ request_id: string }` |
| `ai_cancel` | `{ request_id: string }` | `void` |
| `get_config` | `{}` | `AppConfig` |
| `set_config` | `{ config: Partial<AppConfig> }` | `void` |
| `store_api_key` | `{ key: string }` | `{ valid: bool, error?: string }` |
| `has_api_key` | `{}` | `{ exists: bool }` |
| `delete_api_key` | `{}` | `void` |

### 5.2 Tauri Events (Backend -> Frontend)

| Event | Payload | Description |
|-------|---------|-------------|
| `pty:data` | `{ session_id: string, data: number[] }` | Raw bytes from shell output |
| `pty:exit` | `{ session_id: string, code: number }` | Shell process terminated |
| `pty:error` | `{ session_id: string, error_type: "read_io_error" \| "spawn_failure" \| "process_crash", message: string }` | PTY failure requiring user attention |
| `ai:stream-chunk` | `{ request_id: string, text: string }` | Incremental AI response text |
| `ai:stream-end` | `{ request_id: string, model: string, input_tokens: number, output_tokens: number, stop_reason: string }` | AI response complete |
| `ai:tool-use` | `{ request_id: string, tool_name: string, input: object }` | AI wants to use a tool (e.g., suggest_command) |
| `ai:error` | `{ request_id: string, error_type: string, message: string }` | API or processing error |

### 5.3 ShellContext Contract

Shared between frontend and backend. Frontend gathers some context (from block state), backend enriches it.

```typescript
interface ShellContext {
  shell_type: "bash" | "zsh" | "fish" | "unknown";
  os: string;                    // e.g., "macOS 15.2"
  cwd: string;                   // current working directory
  recent_commands: CommandEntry[]; // last 5 commands with exit codes
  env_snippet?: string;          // selected safe env vars (PATH, SHELL, TERM)
}

interface CommandEntry {
  command: string;
  exit_code: number;
  output_preview?: string;       // first 20 lines, redacted
}
```

---

## 6. Cross-Cutting Concerns

### 6.1 Error Handling

- Backend: Rust Result types propagate through ports. Tauri commands return serialized errors to frontend.
- Frontend: Each hook handles errors and updates error state in stores. ErrorBanner component displays recoverable errors.
- AI errors (rate limit, network, auth): Emit `ai:error` event with actionable error type.

**PTY Error Recovery:**

PTY sessions can fail in three ways, each handled explicitly:

| Error Type | Cause | Recovery |
|-----------|-------|----------|
| `spawn_failure` | Shell binary not found, permission denied, resource exhaustion | `create_terminal_session` returns error immediately. Frontend shows error in new tab with retry option. |
| `read_io_error` | PTY master fd read fails mid-session (I/O error, fd closed unexpectedly) | Tokio read loop catches error, emits `pty:error` event, attempts to kill child process, marks session as failed. Frontend shows error banner with "Close" and "Reopen" actions. |
| `process_crash` | Child shell process terminated by signal (SIGSEGV, SIGKILL, etc.) | Detected by Tokio read loop when read returns EOF + non-zero wait status. Emits `pty:exit` with exit code, then `pty:error` with crash details. Frontend shows "Process crashed" banner. |

In all failure cases:
1. Pending AI requests for the session are cancelled
2. Incomplete blocks are finalized with an error state
3. Command history is flushed to SQLite
4. Session resources (PTY fd, Tokio task) are cleaned up

### 6.2 Logging

- Backend: Rust `log` crate with `env_logger`. Log levels configurable.
- Frontend: Console logging with structured messages.
- Sensitive data never logged (API keys, redacted content).

EVOLUTION: Structured logging to file for debugging user-reported issues.

### 6.3 Concurrency

- PTY I/O: Dedicated Tokio task per terminal session. Reads from PTY master in a loop, emits events.
- AI streaming: Dedicated Tokio task per AI request. Reads SSE stream, emits chunk events.
- SQLite: Single connection with WAL mode. Access serialized through a Tokio mutex or dedicated thread.
- Frontend: Single-threaded (JavaScript). State updates via Zustand subscriptions trigger React re-renders.
