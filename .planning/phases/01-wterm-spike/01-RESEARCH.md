# Phase 1 Research: wterm Spike

## Research Date: 2026-04-27

## Standard Stack

| Component | Library | Version | Notes |
|-----------|---------|---------|-------|
| Terminal (React) | `@wterm/react` | 0.2.0 | Vercel Labs, WASM-based DOM renderer |
| Terminal (DOM) | `@wterm/dom` | 0.2.0 | Peer dependency of @wterm/react |
| Terminal (Core) | `@wterm/core` | 0.2.0 | Headless WASM bridge + WebSocket transport |
| WebSocket (Go) | `gorilla/websocket` | latest | Mature, well-documented WebSocket library |
| SSH (Go) | `golang.org/x/crypto/ssh` | latest | Standard Go SSH client library |
| Frontend | Vite + React | 19 | Already decided |
| CSS | `@wterm/react/css` | — | Built-in terminal styles + themes |

## @wterm/react API

### `<Terminal>` Component Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `cols` | `number` | 80 | Initial column count |
| `rows` | `number` | 24 | Initial row count |
| `autoResize` | `boolean` | false | Auto-resize based on container via ResizeObserver |
| `theme` | `string` | — | `"solarized-dark"`, `"monokai"`, `"light"` |
| `cursorBlink` | `boolean` | false | Cursor blinking animation |
| `wasmUrl` | `string` | — | Serve WASM separately (embedded by default) |
| `debug` | `boolean` | false | Debug mode with DebugAdapter |
| `onData` | `(data: string) => void` | — | User input callback. When omitted, input echoes automatically |
| `onResize` | `(cols: number, rows: number) => void` | — | Resize callback |
| `onReady` | `(wt: WTerm) => void` | — | Called after WASM init |
| `onTitle` | `(title: string) => void` | — | Terminal title change |

Standard `div` props (`className`, `style`) are forwarded.

### `useTerminal()` Hook

```tsx
const { ref, write, resize, focus } = useTerminal();
```

- `ref` → pass to `<Terminal ref={ref}>`
- `write(data: string | Uint8Array)` → write data to terminal
- `resize(cols, rows)` → resize the terminal
- `focus()` → focus the terminal

### Basic Usage Pattern

```tsx
import { Terminal, useTerminal } from "@wterm/react";
import "@wterm/react/css";

function App() {
  const { ref, write } = useTerminal();

  return (
    <Terminal
      ref={ref}
      autoResize
      onData={(data) => socket.send(data)}
    />
  );
}

// WebSocket onmessage → write(data)
// Terminal onData → socket.send(data)
// Terminal onResize → socket.send(JSON.stringify({type:"resize", cols, rows}))
```

### Key Architecture Points

1. **WASM binary embedded** in the package — no extra setup
2. **DOM rendering** — native text selection, clipboard, browser find, accessibility
3. **Alternate screen buffer** — vim, less, htop work correctly
4. **WebSocket transport** built into @wterm/core with reconnection support
5. **Themes via CSS custom properties** — built-in Default, Solarized Dark, Monokai, Light

## Go Backend Architecture

### WebSocket SSH Proxy Pattern

```
Browser ←→ WebSocket ←→ Go Backend ←→ SSH ←→ Remote Server
```

### Required Go Libraries

```go
import (
    "github.com/gorilla/websocket"
    "golang.org/x/crypto/ssh"
)
```

### WebSocket Server Pattern (gorilla/websocket)

```go
var upgrader = websocket.Upgrader{
    ReadBufferSize:  4096,
    WriteBufferSize: 4096,
    CheckOrigin: func(r *http.Request) bool { return true },
}

func handleWS(w http.ResponseWriter, r *http.Request) {
    conn, err := upgrader.Upgrade(w, r, nil)
    // ... bidirectional pipe with SSH
}
```

### SSH Client Pattern (golang.org/x/crypto/ssh)

```go
config := &ssh.ClientConfig{
    User: username,
    Auth: []ssh.AuthMethod{ssh.Password(password)},
    HostKeyCallback: ssh.InsecureIgnoreHostKey(), // spike only
}
client, err := ssh.Dial("tcp", host+":22", config)
session, err := client.NewSession()
```

### Protocol Design (WebSocket ↔ SSH)

**Binary messages:**
- Client → Server: raw terminal input bytes (user keystrokes)
- Server → Client: raw terminal output bytes (SSH stdout)

**JSON control messages** (text frames):
- Client → Server: `{"type":"resize","cols":80,"rows":24}`
- Client → Server: `{"type":"connect","host":"...","port":22,"user":"...","password":"..."}`

### Resize Propagation

1. Browser ResizeObserver → wterm onResize callback
2. Frontend sends `{"type":"resize","cols":N,"rows":N}` via WebSocket
3. Go backend parses → `session.WindowChange(rows, cols)`
4. SSH PTY resize → remote process redraws

## SSH Example Reference

The wterm repo has an SSH example at `examples/ssh/` using Next.js + shadcn:
- Uses `app/` directory structure
- Has `components/ui/` (shadcn components)
- Has `lib/` directory

## Common Pitfalls

1. **Binary vs text WebSocket frames** — SSH data must use binary frames, not text. Use `WriteMessage(websocket.BinaryMessage, data)`.
2. **Resize timing** — Must request PTY with initial size before starting the session.
3. **Goroutine leaks** — Use context cancellation or done channels to clean up bidirectional pipe goroutines.
4. **HostKeyCallback** — Spike can use `InsecureIgnoreHostKey()`, production must verify.
5. **No auth in v1** — Bind to `127.0.0.1` only. Document security implications.
6. **wterm version** — v0.2.0, very new (52 commits). May have edge cases with complex apps.

## Don't Hand-Roll

- SSH protocol implementation → use `golang.org/x/crypto/ssh`
- WebSocket protocol → use `gorilla/websocket`
- Terminal rendering → use `@wterm/react` (WASM-based, not manual DOM)
- Terminal escape sequence parsing → built into wterm's WASM core

## Architecture Patterns

### Backend (Go)
- Single `main.go` for spike — no package splitting needed
- WebSocket upgrade handler at `/ws`
- SSH connection params from initial WebSocket message (JSON)
- Bidirectional pipe: goroutine WS→SSH, goroutine SSH→WS
- Signal handling for graceful shutdown

### Frontend (React + Vite)
- Single page app — terminal fills viewport
- Connection form: host, port, username, password fields
- WebSocket connects on form submit
- `useTerminal` hook for imperative control
- ResizeObserver → onResize → send resize to backend

## Security Considerations (Spike)

- Bind to `127.0.0.1` only — no external access
- `InsecureIgnoreHostKey()` — acceptable for spike
- Password sent via WebSocket — fine for localhost-only spike
- No CORS restrictions for spike — `CheckOrigin: return true`
