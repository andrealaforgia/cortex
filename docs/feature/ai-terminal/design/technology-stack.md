# AI Terminal - Technology Stack

**Status:** Draft
**Date:** 2026-03-19

---

## 1. Stack Overview

| Layer | Technology | Version | License |
|-------|-----------|---------|---------|
| Application Framework | Tauri | v2.x | MIT/Apache-2.0 |
| Backend Language | Rust | stable (1.80+) | MIT/Apache-2.0 |
| Frontend Language | TypeScript | 5.x | Apache-2.0 |
| Frontend Framework | React | 19.x | MIT |
| Terminal Emulation | xterm.js | 5.x | MIT |
| Terminal WebGL Renderer | @xterm/addon-webgl | 0.18.x | MIT |
| Terminal Fit Addon | @xterm/addon-fit | 0.10.x | MIT |
| Terminal Web Links | @xterm/addon-web-links | 0.11.x | MIT |
| Terminal Search | @xterm/addon-search | 0.15.x | MIT |
| PTY Management | portable-pty | 0.8.x | MIT |
| AI API Client | reqwest | 0.12.x | MIT/Apache-2.0 |
| JSON Serialization | serde + serde_json | 1.x | MIT/Apache-2.0 |
| Database | rusqlite | 0.31.x | MIT |
| API Key Storage | keyring | 3.x | MIT/Apache-2.0 |
| CSS Framework | Tailwind CSS | 4.x | MIT |
| Frontend State | Zustand | 5.x | MIT |
| Frontend Build | Vite | 6.x | MIT |
| Backend Async Runtime | Tokio | 1.x | MIT |
| Streaming/SSE Parsing | eventsource-client | 0.13.x | MIT/Apache-2.0 |
| Regex (Redaction) | regex | 1.x | MIT/Apache-2.0 |

All technologies are open source with permissive licenses (MIT or Apache-2.0).

---

## 2. Technology Decisions

### 2.1 Application Framework: Tauri v2

**Decision:** Tauri v2

**Rationale:**
- ~10MB binary vs ~150MB for Electron
- Rust backend provides native-speed PTY management and HTTP streaming
- IPC supports both synchronous commands and async events (needed for our hybrid communication pattern)
- Cross-platform (macOS, Linux, Windows) from single codebase
- Active development, strong community (85k+ GitHub stars)

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Electron | 15x larger binary, higher memory overhead, Node.js intermediary adds complexity for PTY and API calls |
| Native Swift (macOS only) | Not cross-platform. Limits future audience. |
| Full Rust + WGPU | 12-24+ months estimated effort for custom UI framework and terminal renderer. Unacceptable for MVP timeline. |

### 2.2 Terminal Emulation: xterm.js

**Decision:** xterm.js with WebGL renderer addon

**Rationale:**
- Battle-tested: powers VS Code terminal, Tabby, Hyper, and hundreds of web terminals
- Full VT100/ANSI escape sequence handling including Unicode, IME, and accessibility
- WebGL renderer addon provides GPU-accelerated text rendering within the webview
- Rich addon ecosystem (fit, search, web-links, image support)
- Active maintenance by the xtermjs organization

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Custom WGPU renderer | Multi-year effort. Must handle escape sequences, Unicode, font shaping, cursor, selection. Not viable for MVP. |
| alacritty_terminal crate (as library) | Designed for native rendering, not web. Would require building a custom bridge to Tauri webview. Higher integration complexity than xterm.js. |
| Termwiz (WezTerm's lib) | Less documented, tighter coupling to WezTerm's architecture. xterm.js has larger ecosystem. |

### 2.3 PTY Management: portable-pty

**Decision:** portable-pty

**Rationale:**
- Created by the WezTerm author, production-proven
- Cross-platform: Unix PTY on macOS/Linux, ConPTY on Windows
- 902K+ monthly downloads on crates.io
- Clean API: spawn shell, read/write master fd, resize
- Integrates naturally with Rust backend

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| tauri-plugin-pty | Less mature, fewer users. Adds a plugin dependency layer. portable-pty gives more control and is directly usable. |
| node-pty | Node.js library. Would require running Node alongside Rust backend or using NAPI bridges. Unnecessary complexity in a Tauri app. |
| Raw libc PTY calls | Cross-platform handling (especially Windows ConPTY) is significant work. portable-pty already solves this. |

### 2.4 AI API Client: reqwest + Manual SSE Parsing

**Decision:** Use reqwest for HTTP and parse Anthropic's SSE streaming manually (or via eventsource-client crate)

**Rationale:**
- No official Anthropic Rust SDK exists. reqwest is the standard Rust HTTP client.
- The Anthropic Messages API uses server-sent events (SSE) for streaming. Parsing SSE is straightforward.
- Direct HTTP gives full control over timeouts, retries, and error handling.
- Avoids depending on unofficial/unmaintained third-party SDKs.

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| anthropic-rs (unofficial) | Unmaintained or low adoption third-party crates. Risk of breaking changes or abandonment. |
| Calling TypeScript SDK from frontend | API key would be exposed to webview JavaScript context. Security violation. |

EVOLUTION: If Anthropic releases an official Rust SDK, evaluate migration.

### 2.5 Database: rusqlite (SQLite)

**Decision:** rusqlite with SQLite

**Rationale:**
- Zero-configuration embedded database. Single file.
- Handles all persistence needs: config, command history, AI response cache.
- WAL mode provides crash safety and concurrent read/write.
- No external database process to manage.
- rusqlite is the most mature SQLite binding for Rust (MIT license).

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| sqlx | Async-first, adds compile-time query checking complexity. Overkill for our simple schema. We don't need async DB access -- queries are fast and local. |
| sled (embedded KV store) | Less mature, limited query capabilities compared to SQL. Our data has relational aspects (blocks -> commands -> AI responses). |
| Plain JSON files | No query capability. No transactional safety. Doesn't scale with history growth. |

### 2.6 API Key Storage: keyring

**Decision:** keyring crate

**Rationale:**
- Cross-platform access to OS credential stores: macOS Keychain, Windows Credential Manager, Linux Secret Service
- Simple API: set_password / get_password / delete_password
- Well-maintained, 1M+ downloads

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Encrypted config file | Must manage encryption key. Key storage problem is recursive. OS keychain solves this natively. |
| Environment variable only | Not user-friendly for desktop app. No GUI-based key management. Supported as fallback, not primary. |

### 2.7 Frontend Framework: React

**Decision:** React 19 with TypeScript

**Rationale:**
- Largest ecosystem for UI components, state management, and developer tooling
- TypeScript provides type safety across the frontend
- Strong Tauri integration and community examples
- xterm.js has React integration patterns and wrappers

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Svelte | Smaller ecosystem. Fewer Tauri examples. Less familiar to most developers. Viable but React has more momentum. |
| Vue | Similar trade-off to Svelte. React's ecosystem edge is decisive for rapid MVP development. |
| Solid | Very small community. Risk of insufficient ecosystem support. |

### 2.8 CSS: Tailwind CSS

**Decision:** Tailwind CSS v4

**Rationale:**
- Utility-first approach enables rapid UI development
- No custom CSS files to manage for MVP
- Consistent design system out of the box
- v4 has CSS-first configuration (no tailwind.config.js needed)

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Plain CSS / CSS Modules | Slower development for UI-heavy features like AI panel, settings, block rendering. |
| Styled Components | Runtime CSS-in-JS adds overhead. Tailwind is build-time. |

### 2.9 Frontend State: Zustand

**Decision:** Zustand

**Rationale:**
- Minimal boilerplate compared to Redux
- No providers or context wrappers needed
- Supports subscriptions (useful for streaming AI state updates)
- ~2KB bundle size
- TypeScript-first

**Alternatives Rejected:**

| Alternative | Rejection Reason |
|-------------|-----------------|
| Redux Toolkit | More boilerplate, larger bundle. Overkill for our state complexity. |
| Jotai | Atomic model is elegant but less intuitive for grouped state (terminal sessions, AI conversations). |
| React Context | Re-render issues at scale. Not suited for high-frequency terminal state updates. |

---

## 3. Dependency Summary

### 3.1 Rust Backend (Cargo.toml)

```
tauri = "2.x"
portable-pty = "0.8.x"
reqwest = { version = "0.12.x", features = ["stream", "json"] }
tokio = { version = "1.x", features = ["full"] }
serde = { version = "1.x", features = ["derive"] }
serde_json = "1.x"
rusqlite = { version = "0.31.x", features = ["bundled"] }
keyring = "3.x"
regex = "1.x"
eventsource-client = "0.13.x"
uuid = { version = "1.x", features = ["v4"] }
chrono = "0.4.x"
```

### 3.2 Frontend (package.json)

```
react: "^19.0.0"
react-dom: "^19.0.0"
typescript: "^5.0.0"
@tauri-apps/api: "^2.0.0"
@xterm/xterm: "^5.0.0"
@xterm/addon-webgl: "^0.18.0"
@xterm/addon-fit: "^0.10.0"
@xterm/addon-web-links: "^0.11.0"
@xterm/addon-search: "^0.15.0"
zustand: "^5.0.0"
tailwindcss: "^4.0.0"
vite: "^6.0.0"
@tauri-apps/cli: "^2.0.0"
```

---

## 4. Build Toolchain

| Tool | Purpose |
|------|---------|
| Cargo | Rust compilation, dependency management |
| Vite | Frontend bundling, dev server, HMR |
| Tauri CLI | Application packaging, build orchestration |
| pnpm (or npm) | Frontend package management |

Build command: `tauri build` orchestrates both Cargo (backend) and Vite (frontend) into a single distributable binary.

EVOLUTION: CI/CD pipeline with GitHub Actions for automated builds across macOS, Linux, Windows.
