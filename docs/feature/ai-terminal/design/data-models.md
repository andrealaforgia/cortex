# AI Terminal - Data Models

**Status:** Draft
**Date:** 2026-03-19

---

## 1. SQLite Schema

Single database file located at the OS-appropriate app data directory (e.g., `~/Library/Application Support/ai-terminal/data.db` on macOS).

### 1.1 Configuration Table

```sql
CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Stores key-value pairs for application settings. Values are JSON-encoded for complex types.

**Example rows:**

| key | value |
|-----|-------|
| `default_shell` | `"/bin/zsh"` |
| `default_model` | `"claude-sonnet-4-20250514"` |
| `scrollback_lines` | `10000` |
| `theme` | `"dark"` |
| `ai_context_lines` | `20` |
| `redaction_enabled` | `true` |
| `custom_redaction_patterns` | `["my-internal-host\\.corp\\.com"]` |

EVOLUTION: Typed settings table with validation, per-profile configs.

### 1.2 Command History Table

```sql
CREATE TABLE command_history (
    id          TEXT PRIMARY KEY,  -- UUID v4
    session_id  TEXT NOT NULL,
    command     TEXT NOT NULL,
    output      TEXT,              -- truncated to first/last N lines
    exit_code   INTEGER,
    cwd         TEXT,
    shell_type  TEXT,
    started_at  TEXT NOT NULL,     -- ISO 8601
    duration_ms INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_history_session ON command_history(session_id);
CREATE INDEX idx_history_created ON command_history(created_at);
CREATE INDEX idx_history_command ON command_history(command);
```

**Storage policy (MVP):**
- Store last 10,000 command entries.
- Output truncated to 200 lines max (first 100 + last 100).
- Prune entries older than 90 days on app startup.

EVOLUTION: Full-text search on commands and output. Export/import history.

### 1.3 AI Response Cache Table

```sql
CREATE TABLE ai_cache (
    prompt_hash   TEXT PRIMARY KEY,  -- SHA-256 of the normalized prompt
    model         TEXT NOT NULL,
    response      TEXT NOT NULL,
    tokens_in     INTEGER,
    tokens_out    INTEGER,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at  TEXT NOT NULL DEFAULT (datetime('now')),
    use_count     INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_cache_last_used ON ai_cache(last_used_at);
```

**Caching rules (MVP):**
- Cache `explain_command` responses (commands are deterministic).
- Do NOT cache `translate_command` or `chat` responses (context-dependent).
- Do NOT cache `explain_error` responses (output varies).
- LRU eviction: prune when cache exceeds 1,000 entries, remove least recently used.

EVOLUTION: Cache invalidation by model version. Fuzzy prompt matching for near-miss cache hits.

### 1.4 AI Conversation Table

```sql
CREATE TABLE ai_conversations (
    id          TEXT PRIMARY KEY,  -- UUID v4
    title       TEXT,              -- auto-generated from first message
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE ai_messages (
    id              TEXT PRIMARY KEY,  -- UUID v4
    conversation_id TEXT NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,     -- 'user' or 'assistant'
    content         TEXT NOT NULL,
    model           TEXT,             -- null for user messages
    tokens_in       INTEGER,
    tokens_out      INTEGER,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_messages_conversation ON ai_messages(conversation_id, created_at);
```

**Purpose:** Persist chat sidebar conversations across sessions. Users can resume conversations.

EVOLUTION: Conversation branching, conversation search, conversation export.

### 1.5 Schema Migrations

```sql
CREATE TABLE schema_version (
    version    INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Migrations run on app startup. Each migration is a SQL script with a version number. The app checks `schema_version` and applies any unapplied migrations in order.

---

## 2. Domain Data Structures

### 2.1 Block

Represents a single command execution cycle, detected via OSC 133 sequences.

```
Block {
    id: string (UUID)
    session_id: string
    prompt: string             -- prompt text before command
    command: string            -- the command the user typed
    output: string             -- command output (may be large, streamed)
    exit_code: number | null   -- null while command is running
    cwd: string                -- working directory when command ran
    started_at: timestamp
    finished_at: timestamp | null
    duration_ms: number | null
    state: BlockState          -- enum: prompting | running | completed
}
```

**BlockState transitions:**
```
prompting --> running --> completed
   (OSC 133;A/B)  (OSC 133;C)  (OSC 133;D)
```

Blocks live in frontend memory (Zustand store) during a session. Completed blocks are persisted to `command_history` for cross-session recall.

### 2.2 AIConversation

Represents a chat sidebar conversation.

```
AIConversation {
    id: string (UUID)
    title: string
    messages: AIMessage[]
    created_at: timestamp
    updated_at: timestamp
}

AIMessage {
    id: string (UUID)
    role: "user" | "assistant"
    content: string
    model: string | null        -- e.g., "claude-sonnet-4-20250514"
    tokens_in: number | null
    tokens_out: number | null
    created_at: timestamp
}
```

### 2.3 CommandSuggestion

Returned by Claude when using the `suggest_command` tool in translate_command flow.

```
CommandSuggestion {
    command: string             -- the shell command to execute
    explanation: string         -- what the command does
    risk_level: "safe" | "moderate" | "dangerous"
}
```

### 2.4 ShellContext

Assembled by the context builder before each AI request.

```
ShellContext {
    shell_type: "bash" | "zsh" | "fish" | "unknown"
    os: string                  -- e.g., "macOS 15.2", "Ubuntu 24.04"
    cwd: string
    recent_commands: CommandEntry[]  -- last 5 commands
    env_snippet: string | null  -- safe env vars (PATH, SHELL, TERM)
}

CommandEntry {
    command: string
    exit_code: number
    output_preview: string | null  -- first 20 lines, post-redaction
}
```

### 2.5 AppConfig

Application-wide settings.

```
AppConfig {
    default_shell: string       -- path to shell binary
    default_model: string       -- Claude model ID
    scrollback_lines: number    -- max scrollback buffer size
    theme: "dark" | "light"
    font_size: number
    font_family: string
    ai_context_lines: number    -- how many output lines to include in AI context
    redaction_enabled: boolean
    custom_redaction_patterns: string[]  -- user-defined regex patterns
}
```

### 2.6 TerminalSession

In-memory representation of an active terminal session.

```
TerminalSession {
    id: string (UUID)
    shell_path: string
    cwd: string
    pid: number                 -- shell process ID
    rows: number
    cols: number
    blocks: Block[]             -- ordered list of blocks in this session
    created_at: timestamp
    state: SessionState         -- enum: active | closed
}
```

EVOLUTION: Multiple concurrent sessions (tabs), session restore on app restart.

---

## 3. Data Flow Between Components

### 3.1 Terminal I/O Data Flow

```
Frontend (xterm.js)
    |
    | write_to_pty(session_id, bytes)     -- user keystroke
    v
Tauri Command Handler
    |
    | PtyPort.write_input(bytes)
    v
PortablePtyAdapter
    |
    | writes to PTY master fd
    v
Shell Process (bash/zsh/fish)
    |
    | writes output to PTY slave
    v
PortablePtyAdapter (read loop)
    |
    | PtyPort.read_output() -> bytes
    v
Tauri Event: pty:data(session_id, bytes)
    |
    v
Frontend: xterm.js.write(bytes)
    |
    v
Frontend: OSC 133 parser scans bytes
    |
    | if OSC 133 sequence detected
    v
Frontend: Block state updated in terminalStore
```

### 3.2 AI Request Data Flow

```
Frontend: user triggers AI action
    |
    | invoke ai_translate_command(query, shell_context)
    v
Tauri Command Handler (ai_commands.rs)
    |
    | Spawns async task, returns request_id immediately
    v
Domain: Context Builder
    |
    | Assembles full prompt from shell_context + system prompt template
    v
Domain: Redaction Engine
    |
    | Applies regex patterns to strip secrets from assembled context
    v
ClaudeApiPort.send_message_with_tools(prompt, tools)
    |
    v
AnthropicHttpAdapter
    |
    | HTTPS POST to api.anthropic.com/v1/messages (streaming)
    v
SSE Stream processing loop:
    |
    | For each text chunk:
    |   emit Tauri event: ai:stream-chunk(request_id, text)
    |
    | On tool_use block:
    |   emit Tauri event: ai:tool-use(request_id, tool_name, input)
    |
    | On stream end:
    |   emit Tauri event: ai:stream-end(request_id, metadata)
    |   StoragePort.save_ai_cache_entry() if cacheable
    v
Frontend: aiStore updated, UI re-renders
```

### 3.3 Configuration Data Flow

```
App Startup
    |
    v
StoragePort.load_config()
    |
    | Returns key-value pairs from config table
    v
Domain: AppConfig constructed with defaults for missing keys
    |
    v
KeychainPort.retrieve_secret("anthropic_api_key")
    |
    | API key loaded into memory (not stored in AppConfig)
    v
Tauri State: AppConfig + api_key held in Tauri managed state
    |
    v
Frontend: get_config() -> configStore initialized

--- On config change ---

Frontend: set_config(partial_config)
    |
    v
Tauri Command Handler
    |
    | Merge partial config into current config
    v
StoragePort.save_config(updated_key_values)
    |
    v
Tauri State: updated in memory
```

### 3.4 Block Persistence Data Flow

```
Frontend: Block reaches "completed" state (OSC 133;D received)
    |
    | Block data passed to backend
    v
Tauri Command Handler (or automatic on block completion event)
    |
    v
StoragePort.save_command_history(block_data)
    |
    | Truncate output if > 200 lines
    v
SQLite: INSERT into command_history
```

---

## 4. Storage Location

| Data | Location | Format |
|------|----------|--------|
| SQLite database | `{app_data_dir}/data.db` | SQLite 3 |
| API key | OS keychain | Encrypted by OS |
| Shell integration scripts | Bundled in app binary | Plain text (resources) |
| Logs | `{app_data_dir}/logs/` | Plain text |

`app_data_dir` resolved by Tauri:
- macOS: `~/Library/Application Support/ai-terminal/`
- Linux: `~/.local/share/ai-terminal/`
- Windows: `%APPDATA%/ai-terminal/`

EVOLUTION: Data export/import, database backup before migrations, optional cloud sync.
