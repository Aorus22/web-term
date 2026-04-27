# Technology Stack

**Project:** WebTerm — Web-based SSH Client
**Researched:** 2026-04-27
**Overall Confidence:** HIGH

## Recommended Stack

### Core Backend (Go)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `golang.org/x/crypto/ssh` | latest | SSH client connections to target servers | The standard SSH library for Go. Full SSH protocol support (password + key auth, sessions, PTY, shell streams). Used directly by every Go SSH project. No wrapper needed for our client-side use case. |
| `github.com/coder/websocket` | v1.x | WebSocket server for browser ↔ backend communication | Successor to `nhooyr.io/websocket` (now maintained by Coder). First-class `context.Context` support, zero dependencies, idiomatic Go API, concurrent read/write safe. More modern and better maintained than gorilla/websocket. |
| `modernc.org/sqlite` | latest | SQLite driver for connection storage | Pure Go — no CGO, no C compiler required. Cross-compilation is trivial. ~10% slower than `mattn/go-sqlite3` (CGO) but the simplicity tradeoff is worth it for a self-hosted tool. Registers as `"sqlite"` driver for `database/sql`. |
| Go standard library `net/http` | — | HTTP server + API routing | No external router needed for this scope. `http.ServeMux` in Go 1.22+ supports method-based routing (`GET /api/connections`). Keep it simple. |

### Core Frontend (React + Vite)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `@wterm/react` | 0.2.0 | Terminal emulator component | Vercel Labs library. Zig + WASM core (~12KB), DOM renderer, full VT100/VT220/xterm escape sequence support. Built-in themes (solarized-dark, monokai, light). Has `WebSocketTransport` in `@wterm/core` designed exactly for our use case. First-class React support via `<Terminal>` component + `useTerminal` hook. |
| `@wterm/dom` | latest | DOM renderer (peer dependency of @wterm/react) | Required peer dep — handles actual rendering, text selection, clipboard. |
| `vite` | 6.x | Frontend build tool | Fast HMR, native ESM, first-class React + TypeScript support. The `be/` + `fe/` split means Vite dev server proxies API calls to Go backend. |
| `react` | 19.x | UI framework | Latest stable. `@wterm/react` 0.2.0 supports React 19. |
| `shadcn` | 4.x (CLI) | UI component library | Copy-paste components (not a dep). Built on Radix UI + Tailwind CSS. Clean minimal aesthetic matching Vercel style. Vite template: `npx shadcn@latest init -t vite`. |
| `tailwindcss` | 4.x | Utility CSS framework | Required by shadcn. v4 uses `@tailwindcss/vite` plugin (no PostCSS config needed). |
| `typescript` | 5.x+ | Type safety | Required by shadcn and @wterm/react. |

### Supporting Libraries

| Library | Purpose | When to Use |
|---------|---------|-------------|
| `lucide-react` | Icon library | shadcn standard icon set. Used for UI icons (connect, disconnect, tabs, settings). |
| `class-variance-authority` (cva) | Component variant styling | Installed by shadcn. For button variants, dialog styles, etc. |
| `clsx` + `tailwind-merge` | Conditional className merging | Installed by shadcn. Used in every component via `cn()` utility. |

### Database

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `modernc.org/sqlite` | latest | SQLite storage | Pure Go driver. Single file DB for storing SSH connection profiles (host, port, username, labels). Use WAL mode for concurrent reads. |

## Detailed Technology Rationale

### Go SSH Library: `golang.org/x/crypto/ssh`

**Verdict:** Use directly. No wrapper needed.

The official `x/crypto/ssh` package provides everything needed for an SSH **client**:
- `ssh.Dial()` — connect to remote host
- `ssh.NewClient()` — create client from connection
- `client.NewSession()` — open shell session
- `session.RequestPty()` — request PTY allocation (required for interactive terminals)
- `session.Shell()` — start shell on remote
- `session.Stdout/stdin` — pipe data bidirectionally

**Why not `gliderlabs/ssh`?** Gliderlabs wraps `x/crypto/ssh` for building SSH *servers* (like `ssh-chat`). Our use case is SSH *client* — connecting TO servers. Gliderlabs adds no value for client-side and its roadmap explicitly lists "High-level client?" as a future goal. Stick with `x/crypto/ssh` directly.

**Architecture pattern (verified from wterm's own SSH example):**
```
Browser → WebSocket → Go backend → ssh.Dial() → ssh.Session → PTY shell → pipe I/O
```

### WebSocket Library: `github.com/coder/websocket`

**Verdict:** Use `coder/websocket` (successor to `nhooyr.io/websocket`).

**Comparison:**

| Criterion | gorilla/websocket | coder/websocket |
|-----------|-------------------|-----------------|
| Status | Maintained (24.6k stars) but feature-frozen | Actively maintained by Coder |
| Context support | No (uses deadlines) | First-class `context.Context` |
| Concurrent r/w | Manual mutex needed | Safe by default |
| Dependencies | Zero | Zero |
| API style | Traditional Go | Idiomatic modern Go |
| Binary messages | `WriteMessage(Binary, data)` | `c.Write(ctx, MessageBinary, data)` |
| Close handling | Manual close codes | `c.Close(statusCode, reason)` |
| Install | `gorilla/websocket` | `coder/websocket` |

**Why coder/websocket:**
1. nhooyr.io/websocket was transferred to Coder in 2024 (official author handed off)
2. First-class context support means proper cancellation and timeout handling
3. Concurrent read/write without manual synchronization
4. Better fit for long-lived terminal sessions where both read and write goroutines run continuously
5. Coder (the company) uses this in production for their own web terminal product

**For our terminal proxy pattern:**
```go
// Upgrade HTTP to WebSocket
conn, err := websocket.Accept(w, r, &websocket.AcceptOptions{
    OriginPatterns: []string{"localhost:*"},
})

// Bidirectional pipe:
// goroutine 1: WebSocket → SSH stdin
// goroutine 2: SSH stdout → WebSocket
```

### Terminal Library: `@wterm/react` (wterm)

**Verdict:** Use `@wterm/react` + `@wterm/dom` as decided in PROJECT.md.

**Key facts (verified from npm registry and official docs):**
- **npm:** `@wterm/react` v0.2.0 — "React component for wterm — a terminal emulator for the web"
- **Core:** Zig + WASM parser compiled to ~12KB binary (embedded by default)
- **Rendering:** DOM-based — native text selection, clipboard, browser find, screen reader support
- **Alternate screen buffer:** Yes — `vim`, `less`, `htop` work correctly
- **Scrollback:** Configurable ring buffer
- **24-bit color:** Full RGB SGR support
- **Auto-resize:** `ResizeObserver`-based
- **WebSocket transport:** Built-in `WebSocketTransport` class in `@wterm/core`

**React integration (verified from official docs):**
```tsx
// Basic usage
import { Terminal } from "@wterm/react";
import "@wterm/react/css";

function App() {
  return <Terminal />;
}

// SSH integration pattern (onData for sending to WebSocket)
import { Terminal, useTerminal } from "@wterm/react";
import "@wterm/react/css";

function SSHTerminal({ socket }) {
  const { ref, write } = useTerminal();

  // Write data FROM server TO terminal
  socket.onmessage = (e) => write(e.data);

  return (
    <Terminal
      ref={ref}
      onData={(data) => socket.send(data)}  // Send user input TO server
      autoResize
      theme="solarized-dark"
    />
  );
}
```

**`<Terminal>` Props (complete reference):**

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `cols` | `number` | `80` | Initial column count |
| `rows` | `number` | `24` | Initial row count |
| `wasmUrl` | `string` | embedded | Optional URL for WASM binary |
| `theme` | `string` | default | `solarized-dark`, `monokai`, `light`, or custom |
| `autoResize` | `boolean` | `false` | Auto-resize via ResizeObserver |
| `cursorBlink` | `boolean` | `false` | Cursor blinking animation |
| `debug` | `boolean` | `false` | Debug mode with escape sequence inspection |
| `onData` | `(data: string) => void` | echo | User input callback. **When omitted, input auto-echoes.** |
| `onTitle` | `(title: string) => void` | — | Terminal title change callback |
| `onResize` | `(cols, rows) => void` | — | Resize callback |
| `onReady` | `(wt: WTerm) => void` | — | Fires after WASM loads (React-only) |
| `onError` | `(error) => void` | — | WASM load failure (React-only) |

**`useTerminal` Hook returns:**
- `ref` — `React.RefObject<TerminalHandle>` — pass to `<Terminal ref={ref}>`
- `write` — `(data: string | Uint8Array) => void` — write data to terminal
- `resize` — `(cols, rows) => void` — resize terminal grid
- `focus` — `() => void` — focus the terminal

**`@wterm/core` WebSocketTransport:**
```ts
import { WebSocketTransport } from "@wterm/core";

const ws = new WebSocketTransport({
  url: "ws://localhost:8080/pty",
  onData: (data) => { /* handle received data */ },
});
ws.connect();
ws.send("ls\n");
```

**Built-in themes:** `solarized-dark`, `monokai`, `light`, default. Custom themes via CSS custom properties (`--term-fg`, `--term-bg`, `--term-color-0` through `--term-color-15`).

**Important note:** The wterm SSH example in their repo uses **Next.js + Node.js (ssh2/ws)** for the server, but our project uses **Go backend** — same pattern, different backend language. The frontend `@wterm/react` integration is identical regardless.

### SQLite Driver: `modernc.org/sqlite`

**Verdict:** Use `modernc.org/sqlite` over `mattn/go-sqlite3`.

**Comparison:**

| Criterion | mattn/go-sqlite3 | modernc.org/sqlite |
|-----------|------------------|---------------------|
| CGO required | **Yes** (needs C compiler, SQLite C lib) | **No** (pure Go) |
| Cross-compile | Complex (needs cross-compile toolchain) | Trivial (`GOOS=linux GOARCH=arm64 go build`) |
| Performance | Fastest (~10% faster) | ~10% slower |
| `database/sql` compatible | Yes | Yes |
| Driver name | `"sqlite3"` | `"sqlite"` |
| SQLite version | Latest | 3.51.2 |

**Why modernc.org/sqlite:** For a self-hosted tool targeting individual developers, the pure-Go advantage is decisive. Users can download a single binary with no system dependencies. Cross-compilation for different OS/arch combos is trivial. The ~10% performance gap is irrelevant for our use case (storing a handful of SSH connection profiles).

**Usage:**
```go
import (
    "database/sql"
    _ "modernc.org/sqlite"
)

db, err := sql.Open("sqlite", "file:webterm.db?_pragma=foreign_keys(1)&_pragma=journal_mode(WAL)")
```

### Frontend: Vite + React + shadcn

**Setup (verified from shadcn official docs):**

```bash
# 1. Create Vite project
npm create vite@latest fe -- --template react-ts

# 2. Install Tailwind CSS v4
cd fe
npm install tailwindcss @tailwindcss/vite

# 3. Configure vite.config.ts
# (add tailwindcss plugin + @ alias resolution)

# 4. Initialize shadcn (Vite template)
npx shadcn@latest init -t vite

# 5. Add components
npx shadcn@latest add button dialog input tabs
```

**vite.config.ts:**
```typescript
import path from "path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  server: {
    proxy: {
      "/api": "http://localhost:8080",  // Proxy API to Go backend
      "/ws": {
        target: "ws://localhost:8080",
        ws: true,
      },
    },
  },
})
```

### React State Management

**Verdict:** Use React's built-in state (useState/useReducer) + React Context. No external state management library needed.

**Why no Zustand/Redux/Jotai:**
- The app has minimal shared state: list of saved connections, active session tabs
- WebSocket connections are inherently local to their terminal component
- `useTerminal` hook manages terminal state internally
- React Context is sufficient for passing connection list and active tab state
- If complexity grows in v2, Zustand is the natural upgrade path (simplest API, no boilerplate)

**State structure:**
```
AppContext
├── connections: Connection[]        (saved SSH profiles from SQLite)
├── sessions: Session[]              (active SSH sessions)
├── activeSessionId: string | null   (currently focused tab)
└── actions: { addConnection, removeConnection, connect, disconnect }
```

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| SSH library | `golang.org/x/crypto/ssh` | `gliderlabs/ssh` | Server-focused wrapper, no client benefit |
| WebSocket | `coder/websocket` | `gorilla/websocket` | Gorilla is feature-frozen; coder has context support |
| WebSocket | `coder/websocket` | `nhooyr.io/websocket` | Deprecated — author transferred to Coder |
| Terminal | `@wterm/react` | `xterm.js` | PROJECT.md decided on wterm; xterm is heavier (~200KB vs ~12KB WASM) |
| SQLite | `modernc.org/sqlite` | `mattn/go-sqlite3` | CGO complexity not worth it for this use case |
| SQLite | `modernc.org/sqlite` | `zombiezen/go-sqlite` | Lower-level API, more boilerplate |
| Router | `net/http` (stdlib) | `chi`, `gin`, `echo` | Overkill for ~5 API endpoints |
| State mgmt | React Context + useState | Zustand, Redux, Jotai | Unnecessary for this scope |
| HTTP router | `net/http` | `gorilla/mux` | Go 1.22 ServeMux handles method routing natively |

## Installation

### Backend (Go)

```bash
cd be

# Initialize Go module
go mod init github.com/user/webterm

# Core dependencies
go get golang.org/x/crypto/ssh
go get github.com/coder/websocket
go get modernc.org/sqlite

# That's it — no other dependencies needed
```

### Frontend (React + Vite + shadcn)

```bash
cd fe

# Create Vite + React + TypeScript project
npm create vite@latest . -- --template react-ts

# Install Tailwind CSS v4
npm install tailwindcss @tailwindcss/vite

# Initialize shadcn with Vite template
npx shadcn@latest init -t vite

# Add needed shadcn components
npx shadcn@latest add button dialog input tabs dropdown-menu separator scroll-area tooltip

# Install terminal library
npm install @wterm/dom @wterm/react

# Icons (used by shadcn)
npm install lucide-react
```

### Project Structure

```
web-term/
├── be/                          # Go backend
│   ├── go.mod
│   ├── go.sum
│   ├── main.go                  # Entry point, HTTP server
│   ├── handler/
│   │   ├── connections.go       # CRUD for SSH connection profiles
│   │   └── ssh.go               # WebSocket → SSH proxy handler
│   ├── ssh/
│   │   └── client.go            # SSH dial, session, PTY management
│   ├── store/
│   │   ├── db.go                # SQLite init, migrations
│   │   └── connections.go       # Connection CRUD queries
│   └── model/
│       └── connection.go        # Connection struct
├── fe/                          # Vite + React frontend
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   ├── components/
│   │   │   ├── ui/              # shadcn components (auto-generated)
│   │   │   ├── TerminalPane.tsx # wterm terminal wrapper
│   │   │   ├── ConnectionList.tsx
│   │   │   ├── ConnectionForm.tsx
│   │   │   └── TabBar.tsx
│   │   ├── context/
│   │   │   └── AppContext.tsx    # App-wide state
│   │   ├── hooks/
│   │   │   └── useWebSocket.ts  # WebSocket connection management
│   │   ├── lib/
│   │   │   └── utils.ts         # cn() utility from shadcn
│   │   └── types/
│   │       └── index.ts         # TypeScript types
│   └── index.html
└── .planning/
```

## Sources

- **@wterm/react**: npm registry (`@wterm/react` v0.2.0), https://wterm.dev/react, https://github.com/vercel-labs/wterm — **HIGH confidence**
- **@wterm/core WebSocketTransport**: https://github.com/vercel-labs/wterm/blob/main/packages/@wterm/core/README.md — **HIGH confidence**
- **@wterm SSH example**: https://github.com/vercel-labs/wterm/tree/main/examples/ssh (server.ts uses ws + ssh2 Node.js pattern, our Go version mirrors this architecture) — **HIGH confidence**
- **coder/websocket**: https://github.com/coder/websocket (README, Context7 verified) — **HIGH confidence**
- **gorilla/websocket**: GitHub API (24.6k stars, not archived, last pushed 2025-03) — **HIGH confidence**
- **modernc.org/sqlite**: Context7 verified, https://pkg.go.dev/modernc.org/sqlite — **HIGH confidence**
- **golang.org/x/crypto/ssh**: https://pkg.go.dev/golang.org/x/crypto/ssh, Go standard examples — **HIGH confidence**
- **shadcn/ui Vite setup**: Context7 verified, https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/installation/vite.mdx — **HIGH confidence**
- **gliderlabs/ssh**: https://github.com/gliderlabs/ssh (README confirmed server-only focus) — **HIGH confidence**
