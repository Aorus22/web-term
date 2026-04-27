# Architecture Research

**Domain:** Web-based SSH terminal client (WebSocket-to-SSH proxy)
**Researched:** 2026-04-27
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Browser (React SPA)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ TerminalView │  │ ConnectionMgr│  │   TabSystem  │  │ SettingsUI │  │
│  │ (@wterm/react│  │  (CRUD form) │  │  (tab state) │  │ (shadcn)   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────────────┘  │
│         │                 │                  │                          │
│         └─────────────────┴──────────────────┘                          │
│                           │                                             │
│              ┌────────────┴────────────┐                                │
│              │   WebSocket Client      │                                │
│              │   (per-session WS conn) │                                │
│              └────────────┬────────────┘                                │
└───────────────────────────┼─────────────────────────────────────────────┘
                            │ WebSocket (ws:// / wss://)
                            │ JSON connect msg → binary stream
┌───────────────────────────┼─────────────────────────────────────────────┐
│                       Go Backend                                         │
│              ┌────────────┴────────────┐                                │
│              │   WebSocket Handler     │  ← HTTP upgrade → WS           │
│              │   (gorilla/websocket)   │                                │
│              └────────────┬────────────┘                                │
│                           │                                             │
│         ┌─────────────────┴──────────────────┐                          │
│         │         SSH Session Manager         │                          │
│         │   (golang.org/x/crypto/ssh)         │                          │
│         │   - Dial target host                 │                         │
│         │   - Open shell/PTY                   │                         │
│         │   - Proxy data WS ↔ SSH stream       │                        │
│         └─────────────────┬──────────────────┘                          │
│                           │                                             │
│  ┌──────────────┐  ┌──────┴───────┐  ┌──────────────┐                  │
│  │  REST API    │  │  Connection  │  │   Session    │                  │
│  │  (CRUD conn) │  │   Store      │  │   Registry   │                  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘                  │
│         │                 │                                             │
│         └─────────────────┘                                             │
│              SQLite (connections.db)                                     │
└─────────────────────────────────────────────────────────────────────────┘
                            │ TCP/SSH
                            ▼
                 ┌─────────────────────┐
                 │   Target SSH Server  │
                 └─────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Implementation |
|-----------|----------------|----------------|
| **TerminalView** | Renders terminal emulator, handles user input/output | `@wterm/react` `<Terminal>` + `useTerminal()` hook |
| **ConnectionMgr** | CRUD forms for SSH connection profiles (host, port, user, password) | React components + shadcn form inputs |
| **TabSystem** | Multiple simultaneous SSH sessions in browser tabs | React state managing array of active sessions |
| **WebSocket Client** | Bidirectional data channel per SSH session | Browser `WebSocket` API, one WS per session |
| **WebSocket Handler** | Upgrade HTTP→WS, parse connect message, hand off to session manager | `gorilla/websocket` Upgrader |
| **SSH Session Manager** | Establish SSH connection, open PTY shell, proxy data | `golang.org/x/crypto/ssh` Client + Session |
| **Session Registry** | Track active sessions, enable cleanup on disconnect | In-memory map of session ID → session state |
| **REST API** | CRUD endpoints for saved connections | Go `net/http` handlers |
| **Connection Store** | Persist SSH connection profiles | SQLite via `modernc.org/sqlite` |
| **Vite Dev Server** | HMR, TypeScript, bundling for frontend | Vite with React plugin + Tailwind |

## Recommended Project Structure

```
web-term/
├── be/                              # Go backend
│   ├── cmd/
│   │   └── webterm/
│   │       └── main.go              # Entry point: HTTP server + routes
│   ├── internal/
│   │   ├── handler/
│   │   │   ├── connection.go        # REST CRUD handlers for connections
│   │   │   └── websocket.go         # WebSocket upgrade + SSH proxy handler
│   │   ├── ssh/
│   │   │   ├── session.go           # SSH session lifecycle (dial, shell, proxy)
│   │   │   └── registry.go          # Active session tracking + cleanup
│   │   ├── store/
│   │   │   ├── sqlite.go            # SQLite init + migrations
│   │   │   └── connection.go        # Connection CRUD queries
│   │   └── model/
│   │       └── connection.go        # Connection struct + types
│   ├── go.mod
│   ├── go.sum
│   └── webterm.db                   # SQLite database (runtime, gitignored)
│
├── fe/                              # React frontend (Vite)
│   ├── src/
│   │   ├── components/
│   │   │   ├── ui/                  # shadcn/ui components (auto-generated)
│   │   │   ├── terminal-view.tsx    # @wterm/react wrapper with WS integration
│   │   │   ├── connection-form.tsx  # New/edit connection form
│   │   │   ├── connection-list.tsx  # Sidebar list of saved connections
│   │   │   ├── tab-bar.tsx          # Tab navigation for sessions
│   │   │   └── layout.tsx           # App shell: sidebar + tabs + terminal
│   │   ├── hooks/
│   │   │   ├── use-ssh-session.ts   # WebSocket connect/disconnect + data flow
│   │   │   └── use-connections.ts   # Fetch/CRUD saved connections
│   │   ├── lib/
│   │   │   ├── api.ts               # REST client for backend
│   │   │   └── utils.ts             # shadcn cn() utility
│   │   ├── types/
│   │   │   └── connection.ts        # Connection type definitions
│   │   ├── App.tsx                  # Root component
│   │   ├── main.tsx                 # React entry point
│   │   └── index.css                # Tailwind + wterm styles
│   ├── components.json              # shadcn/ui config
│   ├── vite.config.ts               # Vite + Tailwind + path alias config
│   ├── tsconfig.json
│   ├── package.json
│   └── public/
│       └── wterm.wasm               # WASM binary (optional, can be embedded)
│
└── .planning/                       # GSD planning artifacts
```

### Structure Rationale

- **`be/cmd/webterm/`:** Standard Go convention — main.go is lean, imports from internal packages
- **`be/internal/handler/`:** HTTP/WebSocket handlers separated by concern (REST vs WS)
- **`be/internal/ssh/`:** SSH logic isolated — session.go handles dial/shell/proxy, registry.go tracks active sessions in memory
- **`be/internal/store/`:** Database layer behind an interface — could swap SQLite later without touching handlers
- **`be/internal/model/`:** Pure data structs, no dependencies on DB or HTTP layers
- **`fe/src/components/ui/`:** shadcn convention — auto-generated, never edit directly
- **`fe/src/hooks/`:** Custom hooks own WebSocket lifecycle and API calls — components stay declarative
- **`fe/src/hooks/use-ssh-session.ts`:** Central hook that wires `@wterm/react`'s `write()` to WebSocket `onmessage` and `onData` to WebSocket `send()`

## Architectural Patterns

### Pattern 1: First-Message Connection Negotiation

**What:** The WebSocket connection is established empty. The first message from the client is a JSON object with SSH connection parameters. All subsequent messages are raw binary/text data for the terminal stream.

**When to use:** This is the standard pattern for SSH-over-WebSocket proxies. The wterm SSH example from Vercel Labs uses exactly this approach.

**Trade-offs:**
- Pro: Single WS endpoint, no URL-encoded credentials, clean separation of "connect" from "data"
- Pro: Easy to add reconnect with same socket
- Con: Slightly more complex message handling (first message vs rest)

**Example (server-side, Go):**
```go
func handleSSH(ws *websocket.Conn) {
    // Step 1: Read connect params (first message only)
    _, msg, err := ws.ReadMessage()
    if err != nil { return }
    
    var params ConnectParams
    json.Unmarshal(msg, &params)
    
    // Step 2: Dial SSH
    config := &ssh.ClientConfig{
        User: params.Username,
        Auth: []ssh.AuthMethod{ssh.Password(params.Password)},
        HostKeyCallback: ssh.InsecureIgnoreHostKey(), // v1 only
    }
    client, err := ssh.Dial("tcp", fmt.Sprintf("%s:%d", params.Host, params.Port), config)
    
    // Step 3: Open shell session
    session, _ := client.NewSession()
    session.RequestPty("xterm-256color", 24, 80, ssh.TerminalModes{})
    stdin, _ := session.StdinPipe()
    stdout, _ := session.StdoutPipe()
    session.Shell()
    
    // Step 4: Proxy data bidirectionally
    go io.Copy(stdin, wsReader)  // WS → SSH stdin
    io.Copy(wsWriter, stdout)    // SSH stdout → WS
}
```

### Pattern 2: One WebSocket Per SSH Session

**What:** Each open terminal tab gets its own dedicated WebSocket connection to the backend. The backend creates one SSH client per WebSocket.

**When to use:** Multi-tab terminal applications where each tab is an independent SSH session.

**Trade-offs:**
- Pro: Simple mental model — close tab = close WS = close SSH
- Pro: No multiplexing complexity, no session ID routing
- Pro: Natural backpressure — if SSH is slow, WS backpressure handles it
- Con: One goroutine pair per session (acceptable for individual developer use)

**Example (frontend):**
```tsx
// Per-tab hook
function useSSHSession(params: ConnectParams) {
  const ws = useRef<WebSocket | null>(null);
  const { ref, write } = useTerminal();

  const connect = () => {
    const socket = new WebSocket(`ws://${host}/api/ssh`);
    socket.binaryType = "arraybuffer";

    socket.onopen = () => {
      socket.send(JSON.stringify(params)); // First message: connect params
    };

    socket.onmessage = (event) => {
      const text = new TextDecoder("latin1").decode(event.data);
      write(text); // Write SSH output to terminal
    };

    ws.current = socket;
  };

  const handleData = useCallback((data: string) => {
    ws.current?.send(data); // Send terminal input to SSH
  }, []);

  return { ref, handleData, connect, disconnect };
}
```

### Pattern 3: Store-First for Connection Profiles

**What:** Connection profiles (host, port, username) are stored in SQLite on the backend, not in browser localStorage. Frontend fetches them via REST API.

**When to use:** Self-hosted tools where data should persist across browsers/devices.

**Trade-offs:**
- Pro: Data survives browser clear, works across devices
- Pro: Single source of truth
- Con: Requires backend CRUD API (small overhead)
- Note: Passwords stored in plaintext for v1 (no auth = no security boundary anyway)

## Data Flow

### SSH Session Lifecycle

```
[User clicks "Connect"]
    ↓
ConnectionForm → useSSHSession.connect(params)
    ↓
POST /api/ssh (WebSocket upgrade)
    ↓
Browser: WS.onopen → WS.send(JSON{host, port, user, pass})
    ↓
Go Backend: WS handler reads first message
    ↓
ssh.Dial("tcp", "host:port", config)
    ↓
session.RequestPty("xterm-256color", rows, cols)
    ↓
session.Shell()
    ↓
┌─────────────────────────────────────────────────┐
│ Bidirectional Proxy Loop:                        │
│                                                  │
│  Terminal.onData → WS.send(data) → stdin.Write() │
│                                                  │
│  stdout.Read() → WS.send(data) → write(data)    │
│                                                  │
│  Terminal.onResize → WS.send(JSON{type:"resize"})│
│       → session.WindowChange(rows, cols)          │
└─────────────────────────────────────────────────┘
    ↓
[User closes tab / clicks disconnect]
    ↓
WS.close() → session.Close() → client.Close()
    ↓
Registry removes session
```

### Connection CRUD Flow

```
[User saves connection]
    ↓
ConnectionForm → POST /api/connections
    ↓
Handler → Store.Insert(connection)
    ↓
SQLite INSERT → return connection with ID
    ↓
[User views connections]
    ↓
GET /api/connections → Store.List() → SQLite SELECT
    ↓
ConnectionList component renders sidebar
```

### State Management

```
React State (no global store needed for v1):
├── App
│   ├── connections: Connection[]          # Fetched from REST API
│   ├── sessions: Session[]               # Active SSH sessions
│   │   ├── id: string                    # Unique session ID
│   │   ├── connectionId: string          # Which saved connection
│   │   ├── ws: WebSocket                 # Live WS connection
│   │   └── title: string                 # Tab title
│   └── activeTab: string | null          # Currently viewed session
│
└── useSSHSession (per-tab)
    ├── ws: WebSocket | null
    ├── state: "disconnected" | "connecting" | "connected" | "error"
    └── error: string
```

### Key Data Flows

1. **Terminal Input Flow:** User types → `@wterm/react` fires `onData(data)` → `ws.send(data)` → Go reads WS message → writes to SSH `stdin` pipe
2. **Terminal Output Flow:** SSH `stdout` pipe has data → Go reads → `ws.send(data)` → `ws.onmessage` → `write(text)` to terminal
3. **Resize Flow:** Browser resize → `@wterm/react` fires `onResize(cols, rows)` → client sends JSON `{type:"resize", cols, rows}` → Go parses → `session.WindowChange(rows, cols)`
4. **Disconnect Flow:** Tab close / disconnect button → `ws.close()` → Go detects WS close → closes SSH session → removes from registry

## WebSocket Protocol Design

### Message Format

The protocol uses a **first-message-is-JSON, subsequent-messages-are-binary** pattern, matching the wterm SSH reference implementation.

#### Client → Server Messages

**Message 1 (Connect):** JSON string
```json
{
  "host": "192.168.1.100",
  "port": 22,
  "username": "root",
  "password": "secret"
}
```

**Message 2+ (Terminal input):** Raw UTF-8 string (text frames)
```
"ls -la\n"
```

**Resize message:** JSON string (distinguished by `{` prefix)
```json
{
  "type": "resize",
  "cols": 120,
  "rows": 40
}
```

#### Server → Client Messages

**Error response:** JSON string
```json
{
  "error": "SSH connection failed: connection refused"
}
```

**Terminal output:** Binary/text frames
```
Raw bytes from SSH stdout (ANSI/VT100 escape sequences included)
```

### Message Routing Logic (Client-side)

```typescript
// Sending
function sendToServer(data: string) {
  if (ws.readyState !== WebSocket.OPEN) return;
  ws.send(data); // Terminal input as raw string
}

// Resize
function sendResize(cols: number, rows: number) {
  ws.send(JSON.stringify({ type: "resize", cols, rows }));
}

// Receiving
ws.onmessage = (event) => {
  if (event.data instanceof ArrayBuffer) {
    const text = new TextDecoder("latin1").decode(event.data);
    write(text); // Write to @wterm/react terminal
  } else {
    const data = event.data as string;
    if (data.startsWith("{")) {
      const msg = JSON.parse(data);
      if (msg.error) { /* show error */ return; }
    }
    write(data); // Terminal output as string
  }
};
```

## SQLite Schema

### connections table

```sql
CREATE TABLE IF NOT EXISTS connections (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL,          -- User-friendly label (e.g., "Production Web")
    host       TEXT    NOT NULL,          -- Target hostname or IP
    port       INTEGER NOT NULL DEFAULT 22,
    username   TEXT    NOT NULL,
    password   TEXT    NOT NULL DEFAULT '',-- v1: plaintext (no auth boundary anyway)
    created_at TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_connections_name ON connections(name);
```

### Schema Notes

- **No auth/users table** — v1 has no authentication; single-user assumption
- **Plaintext passwords** — acceptable because there's no auth boundary to protect. If the server is accessible, the DB is accessible. Will revisit in v2 with auth.
- **INTEGER PRIMARY KEY** — SQLite auto-increment pattern
- **ISO 8601 timestamps as TEXT** — standard SQLite convention, sortable
- **`updated_at`** — maintained in application code on updates

### Future Tables (v2+)

```sql
-- v2: When auth is added
CREATE TABLE users (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    -- password hash, etc.
);

-- v2: Connections become per-user
ALTER TABLE connections ADD COLUMN user_id INTEGER REFERENCES users(id);
```

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| 1 user (v1 target) | Single Go binary, in-memory session map, SQLite file. No optimization needed. |
| 10 concurrent users | Same architecture. Go handles thousands of goroutines trivially. SQLite WAL mode for concurrent reads. |
| 100+ users | Add PostgreSQL, session store in Redis, rate limiting on WS connections. This is out of scope for v1. |

### Scaling Priorities

1. **First bottleneck:** SQLite write contention — unlikely for individual use. If it hits, switch to WAL mode (`PRAGMA journal_mode=WAL`).
2. **Second bottleneck:** Memory per SSH session — each session holds ~2 goroutines + SSH buffers. At ~100 sessions this is still <100MB. Not a concern for v1.

**v1 is intentionally single-user.** Do not over-engineer for scale.

## Anti-Patterns

### Anti-Pattern 1: URL-Encoded SSH Credentials

**What people do:** Pass host/port/username in the WebSocket URL path or query string: `ws://host/api/ssh?host=1.2.3.4&user=root&pass=secret`
**Why it's wrong:** Credentials appear in server logs, browser history, proxy logs. Even without auth, this leaks sensitive data.
**Do this instead:** First-message JSON pattern — credentials travel in the WebSocket data channel, not the URL. This matches the wterm SSH reference implementation.

### Anti-Pattern 2: Multiplexing Multiple Sessions Over One WebSocket

**What people do:** Send all SSH sessions through a single WebSocket with session IDs in each message.
**Why it's wrong:** Adds complexity (framing, routing, partial message handling). One session's slow SSH connection blocks others. Hard to handle partial failures.
**Do this instead:** One WebSocket per SSH session. Goroutines are cheap. Close tab = close WS = close SSH. Simple and reliable.

### Anti-Pattern 3: Storing Connection Passwords in localStorage

**What people do:** Save SSH passwords in `localStorage` for auto-fill.
**Why it's wrong:** Any XSS on any page on the same origin can read localStorage. Survives browser restart = bigger attack surface.
**Do this instead:** Store connection profiles server-side in SQLite. Frontend fetches on demand. Passwords sent only at connect time over WebSocket.

### Anti-Pattern 4: Using io.Copy Directly Between WebSocket and SSH

**What people do:** Use `websocket.NetConn()` to get a `net.Conn` and `io.Copy` between that and SSH pipes.
**Why it's wrong:** Loses message boundaries, can't intercept resize messages, can't detect and handle JSON control messages separately.
**Do this instead:** Manual read loop that distinguishes control messages (JSON starting with `{`) from terminal data. Use goroutines for bidirectional proxy with proper error handling and cleanup.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Target SSH servers | `golang.org/x/crypto/ssh` client | Dial over TCP, PTY shell session |
| WASM binary | Embedded in `@wterm/react` package by default | Can serve separately via `wasmUrl` prop if needed |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Frontend ↔ Backend (REST) | HTTP JSON API | CORS needed for Vite dev server (port 5173 → Go port 8080) |
| Frontend ↔ Backend (WS) | WebSocket `/api/ssh` | Binary + text frames, first-message JSON handshake |
| Go Handler ↔ SSH Manager | Direct function calls | In-process, no serialization |
| Go Handler ↔ Store | Go interfaces | `Store` interface enables testing with mocks |
| Go SSH Manager ↔ Target | TCP SSH protocol | Standard SSH-2, password auth for v1 |

### Vite Dev Proxy Configuration

During development, Vite proxies API and WS requests to the Go backend:

```typescript
// fe/vite.config.ts
export default defineConfig({
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        ws: true, // Proxy WebSocket connections
      },
    },
  },
});
```

This avoids CORS issues during development. In production, Go serves the built frontend static files directly.

## Build Order with Dependency Rationale

The following build order respects dependency chains — each phase only depends on previously built components:

```
Phase 1: Backend Foundation
├── Go module init + SQLite store + connection model
├── REST CRUD endpoints for connections
└── Basic HTTP server with routing
    Reason: No frontend needed to test. Verify with curl.
    Depends on: Nothing

Phase 2: Frontend Foundation
├── Vite + React + shadcn/ui setup
├── Connection list + form UI (wired to REST API)
└── Basic layout (sidebar + content area)
    Reason: Needs Phase 1's REST API to be functional.
    Depends on: Phase 1

Phase 3: WebSocket SSH Proxy (Core Feature)
├── Go WebSocket handler + SSH session manager
├── Terminal view component with @wterm/react
├── useSSHSession hook wiring terminal ↔ WebSocket
└── Connect flow: select connection → open terminal
    Reason: Needs @wterm/react (first-time library) AND Go WebSocket+SSH.
    Most complex phase — both sides must work together.
    Depends on: Phase 1 (connection data), Phase 2 (UI shell)

Phase 4: Multi-Tab + Polish
├── Tab system for multiple simultaneous sessions
├── Session state management (active sessions array)
├── Resize handling (terminal.onResize → WS → SSH)
├── Error handling + reconnection UX
└── Visual polish (loading states, transitions)
    Reason: Needs Phase 3's single-session working first.
    Depends on: Phase 3
```

## Sources

- **@wterm/react documentation** — https://wterm.dev/react (verified 2026-04-27, official Vercel Labs)
- **@wterm/react npm** — Version 0.2.0, published 2026-04-26
- **wterm SSH example** — https://github.com/vercel-labs/wterm/tree/main/examples/ssh (reference implementation using ws + ssh2)
- **wterm SSH page.tsx** — Shows exact `<Terminal>` + `useTerminal()` + WebSocket integration pattern
- **wterm SSH server.ts** — Shows first-message JSON negotiation pattern, binary stream proxy
- **gorilla/websocket** — https://github.com/gorilla/websocket (Go WebSocket library, Context7 verified)
- **coder/websocket** — https://github.com/coder/websocket (modern alternative, NetConn wrapper)
- **golang.org/x/crypto/ssh** — https://pkg.go.dev/golang.org/x/crypto/ssh (Go SSH client, standard library)
- **shadcn/ui Vite installation** — https://ui.shadcn.com/docs/installation/vite (Context7 verified)
- **modernc.org/sqlite** — Pure Go SQLite driver, no CGO needed
- **Standard Go Project Layout** — https://github.com/golang-standards/project-layout (Context7 verified)

---
*Architecture research for: web-based SSH terminal client*
*Researched: 2026-04-27*
