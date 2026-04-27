# Project Research Summary

**Project:** WebTerm
**Domain:** Web-based SSH terminal client (self-hosted, single-user)
**Researched:** 2026-04-27
**Confidence:** HIGH

## Executive Summary

WebTerm is a self-hosted web-based SSH client that follows a well-established architecture pattern: browser-based terminal emulator connected to a Go backend via WebSocket, which proxies SSH connections to target servers. This exact pattern is used by production tools like Coder (coder/websocket), WebSSH2, and wterm's own SSH example. The domain is well-understood — there are no novel technical challenges, only execution quality matters.

The recommended approach is a Go backend (`golang.org/x/crypto/ssh` + `coder/websocket` + `modernc.org/sqlite`) paired with a React frontend (`@wterm/react` terminal + Vite + shadcn/ui). The critical path is the WebSocket-to-SSH proxy — get bidirectional I/O piping, PTY resize, and session cleanup right, and everything else is UI work. The architecture naturally decomposes into: (1) backend foundation with REST CRUD, (2) frontend shell with connection management, (3) the WebSocket↔SSH proxy (the hardest phase), and (4) multi-tab + polish.

The primary risk is **@wterm/react library maturity** (v0.2.0, published 2026-04-26, only 52 commits). It must be spiked first — if it can't handle vim/htop/tmux correctly, the fallback is xterm.js with minimal architectural disruption (the WebSocket protocol should be terminal-agnostic). Secondary risks are SSH session zombie leaks (prevent with `context.Context`-based cancellation) and password storage security (don't store passwords in v1; prompt each time).

## Key Findings

### Recommended Stack

Go backend with three core dependencies (SSH, WebSocket, SQLite) — no framework, no ORM, no router beyond stdlib `net/http`. React frontend with Vite build, shadcn/ui components, and the @wterm terminal emulator. All choices favor simplicity and single-binary deployment.

**Core technologies:**
- `golang.org/x/crypto/ssh` — SSH client connections — the standard Go SSH library, full protocol support, no wrapper needed for client-side use
- `github.com/coder/websocket` — WebSocket server — successor to nhooyr.io/websocket, concurrent read/write safe, first-class context support, maintained by Coder (who build web terminals professionally)
- `modernc.org/sqlite` — SQLite driver — pure Go, no CGO, trivial cross-compilation, single-binary deployment
- `@wterm/react` (v0.2.0) — Terminal emulator — Zig+WASM core (~12KB), DOM rendering, native copy/paste, built-in themes, WebSocket transport
- `shadcn/ui` + Tailwind CSS v4 — UI components — copy-paste pattern, Vercel aesthetic, Vite template available
- React 19 + Vite 6 + TypeScript — Frontend framework — modern, fast HMR, proxy config for Go backend

### Expected Features

The feature landscape is well-mapped across 8 competitors (Termius, WebSSH2, ttyd, Next Terminal, Apache Guacamole, Wetty). WebTerm's positioning: self-hosted single-binary alternative to Termius SaaS, with modern UI.

**Must have (table stakes):**
- SSH connection via WebSocket proxy — core value proposition, browser→Go→SSH target
- Password authentication — most basic auth method, every SSH client supports it
- Save connections — CRUD to SQLite, list in sidebar
- Multi-tab terminal — independent sessions, each with own WebSocket
- Terminal emulator fidelity — vim, htop, tmux, curses apps must work
- Copy/paste — native browser clipboard via DOM rendering
- Auto-resize — terminal resizes with window, propagated to remote PTY
- Connection status indicator — visual connected/connecting/disconnected state

**Should have (differentiators):**
- Quick-connect bar — `user@host:port` one-shot connections
- Dark/light theme — system preference detection + manual toggle
- Keyboard shortcuts — Ctrl+T/W/Tab for tab management
- Connection grouping/tags — organize by project/environment
- Export/import connections — JSON backup

**Defer (v2+):**
- Authentication system, SFTP/file transfer, SSH key management, port forwarding, mobile responsive, session recording, team features

### Architecture Approach

Standard WebSocket-to-SSH proxy with one WebSocket per SSH session (no multiplexing). First WebSocket message is JSON with connection params; all subsequent messages are raw terminal data. Connection profiles stored server-side in SQLite. React state via built-in hooks + Context (no external state management needed for this scope).

**Major components:**
1. **WebSocket Handler** (Go) — HTTP upgrade to WS, parse connect message, hand off to SSH session manager
2. **SSH Session Manager** (Go) — Dial target, open PTY shell, bidirectional I/O proxy with context-based cancellation
3. **Connection Store** (Go) — SQLite CRUD for connection profiles, WAL mode for concurrent reads
4. **Terminal View** (React) — `@wterm/react` wrapper with WebSocket integration via `useSSHSession` hook
5. **Tab System** (React) — Session state management, per-tab WebSocket lifecycle
6. **Connection List/Form** (React) — Sidebar CRUD UI wired to REST API

### Critical Pitfalls

1. **@wterm library maturity** (v0.2.0, days old) — Spike before building anything. Test vim/htop/tmux. Have xterm.js fallback ready.
2. **SSH session zombie leaks** — Use `context.Context` for cancellation. Deferred cleanup on both WS close and SSH close. Global session registry with TTL.
3. **Terminal resize not propagating end-to-end** — Typed JSON protocol `{"type":"resize","cols":N,"rows":N}`, debounce 100ms, initial dimensions in PTY request, note `WindowChange(rows, cols)` parameter order.
4. **WebSocket concurrent write panic** — Use `coder/websocket` (not gorilla) which supports concurrent writes natively.
5. **No authentication = open proxy** — Bind to `127.0.0.1` only. Startup warning if exposed. Document clearly.
6. **Password storage in plaintext** — Don't store passwords in v1. Prompt each time. Store only host/port/username.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 0: wterm Spike
**Rationale:** The terminal library is the highest-risk dependency (v0.2.0, 52 commits, no prior experience). Must validate before committing to architecture.
**Delivers:** Minimal prototype: `@wterm/react` → WebSocket → Go → SSH to localhost. Confirms vim/htop/resize work.
**Addresses:** Pitfall #1 (wterm maturity)
**Avoids:** Building entire product on a library that can't handle SSH terminal workloads

### Phase 1: Backend Foundation
**Rationale:** REST CRUD and SQLite store have no frontend dependencies. Can verify with curl. Establishes data model and API contract.
**Delivers:** Go HTTP server with connection CRUD endpoints, SQLite with migrations, connection model
**Addresses:** Save connections (table stake)
**Uses:** `modernc.org/sqlite`, Go stdlib `net/http`, `coder/websocket`
**Avoids:** Binding to `0.0.0.0` (pitfall #5)

### Phase 2: Frontend Foundation
**Rationale:** Needs Phase 1's REST API. Establishes UI shell, shadcn setup, connection management UI.
**Delivers:** Vite+React+shadcn app, connection list sidebar, connection form, basic layout
**Addresses:** Save connections UI, clean modern UI (table stakes)
**Uses:** Vite 6, React 19, shadcn/ui, Tailwind CSS v4

### Phase 3: WebSocket SSH Proxy (Core Feature)
**Rationale:** This is the product's core value. Most complex phase — both Go (SSH+WebSocket) and React (terminal integration) must work together. Depends on Phase 1 (connection data) and Phase 2 (UI shell).
**Delivers:** Working SSH terminal in browser — connect to saved server, interactive shell, vim/htop work, resize propagates, copy/paste works
**Addresses:** SSH connection, password auth, terminal fidelity, copy/paste, auto-resize (5 table stakes)
**Avoids:** SSH zombie leaks (pitfall #2), resize issues (pitfall #3), concurrent write panics (pitfall #4), HostKeyCallback mishandling (pitfall #6)
**Implements:** WebSocket Handler, SSH Session Manager, Terminal View, useSSHSession hook

### Phase 4: Multi-Tab + Polish
**Rationale:** Needs Phase 3's single-session working. Multi-tab is N × single session. Polish features are independent.
**Delivers:** Multiple simultaneous SSH sessions in tabs, connection status indicators, error handling, keyboard shortcuts, theme switching, quick-connect bar
**Addresses:** Multi-tab (table stake), connection status (table stake), keyboard shortcuts, dark/light theme, quick-connect (differentiators)
**Implements:** Tab System component, session state management

### Phase Ordering Rationale

- **Spike first** because wterm is the #1 risk — if it fails, everything changes
- **Backend before frontend** because REST API is independently testable and establishes data contracts
- **Single session before multi-tab** because multi-tab = N × single session (confirmed by feature dependency analysis)
- **SSH proxy after UI shell** because the proxy needs the connection form and sidebar to be interactive
- **Polish last** because it's independent features that don't affect core architecture

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 0 (wterm spike):** Must verify `@wterm/core` WebSocket transport compatibility with Go backend protocol. The official SSH example uses Node.js — no Go reference exists.
- **Phase 3 (SSH proxy):** `coder/websocket` API for bidirectional proxy pattern with context cancellation. Need to verify exact `Read()`/`Write()` semantics for long-lived connections.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Backend):** Standard Go HTTP + SQLite CRUD — well-documented, trivial
- **Phase 2 (Frontend):** Standard Vite+React+shadcn setup — shadcn has Vite template
- **Phase 4 (Multi-tab):** Standard React state management — tabs, context, no novel patterns

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All technologies verified against official docs and npm/pkg.go.dev. Alternative analysis thorough. |
| Features | HIGH | 8 competitors analyzed. Feature matrix complete. MVP scope well-defined. |
| Architecture | HIGH | Standard WebSocket↔SSH proxy pattern. Reference implementation exists (wterm SSH example). Data flow fully mapped. |
| Pitfalls | HIGH | SSH+WebSocket pitfalls well-documented across gorilla/coder/xterm.js ecosystems. All pitfalls have concrete prevention strategies. |

**Overall confidence:** HIGH

### Gaps to Address

- **@wterm/react stability:** Cannot be fully assessed until spike. Library was published 1 day before research. DOM-based terminal rendering is unique (advantage) but unproven at scale. Must validate during Phase 0.
- **Go SSH ↔ coder/websocket integration pattern:** No reference implementation exists combining these two specific libraries. The pattern is straightforward (both are idiomatic Go) but integration details (message framing, error handling, cleanup) need to be discovered during implementation.
- **@wterm/core WebSocketTransport protocol:** May have its own message format expectations that conflict with our Go backend protocol. Needs verification during spike.

## Sources

### Primary (HIGH confidence)
- `@wterm/react` npm registry (v0.2.0) + https://wterm.dev/react + https://github.com/vercel-labs/wterm
- `golang.org/x/crypto/ssh` — pkg.go.dev official docs
- `github.com/coder/websocket` — GitHub README, Context7 verified
- `modernc.org/sqlite` — pkg.go.dev, Context7 verified
- shadcn/ui Vite installation — https://ui.shadcn.com/docs/installation/vite, Context7 verified

### Secondary (MEDIUM confidence)
- wterm SSH example (vercel-labs/wterm/examples/ssh) — Node.js reference, Go adaptation needed
- Termius, WebSSH2, ttyd, Next Terminal, Guacamole, Wetty — competitor feature analysis
- gorilla/websocket — Context7 docs (used for comparison only, not recommended)

---
*Research completed: 2026-04-27*
*Ready for roadmap: yes*
