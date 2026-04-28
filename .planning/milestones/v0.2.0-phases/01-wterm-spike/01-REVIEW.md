---
phase: 01-wterm-spike
reviewed: 2026-04-27T12:30:00Z
depth: deep
files_reviewed: 5
files_reviewed_list:
  - be/main.go
  - be/go.mod
  - fe/src/App.tsx
  - fe/src/App.css
  - fe/vite.config.ts
findings:
  critical: 4
  warning: 4
  info: 2
  total: 10
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-27T12:30:00Z
**Depth:** deep
**Files Reviewed:** 5
**Status:** issues_found

## Summary

Reviewed the WebTerm spike: a Go WebSocket-to-SSH proxy backend and React terminal frontend. The spike validates the architecture but has **4 critical security issues** that must be addressed before any broader exposure. SSH credentials are transmitted in cleartext over unencrypted WebSockets, host key verification is disabled, the WebSocket endpoint has no origin checking or authentication (enabling SSRF), and error messages from SSH are injected into JSON via a broken manual escaper that can produce invalid JSON. The goroutine lifecycle relies on deferred cleanup to unblock stranded reads — fragile but functionally correct in practice.

## Critical Issues

### CR-01: SSH credentials transmitted over cleartext WebSocket

**File:** `fe/src/App.tsx:27`
**Issue:** The WebSocket URL uses `ws://` (not `wss://`), and the SSH password is sent as plaintext JSON in the initial connect message (lines 34-41). Anyone on the network path can capture SSH credentials via packet sniffing. This is the most severe security vulnerability — it defeats the purpose of SSH encryption by exposing credentials before the SSH tunnel is established.

**Fix:**
```typescript
// Use wss:// in production, detect protocol from page
const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
const socket = new WebSocket(`${protocol}//${wsHost}:8080/ws`);
```
Additionally, configure TLS on the Go backend (`ListenAndServeTLS`) or terminate TLS at a reverse proxy. For the spike, document this as a known limitation and ensure it's addressed before any non-localhost deployment.

---

### CR-02: No WebSocket origin check — any website can connect

**File:** `be/main.go:24-26`
**Issue:** `CheckOrigin` unconditionally returns `true`, allowing any origin to open a WebSocket to the proxy. A malicious website can initiate SSH connections through the user's browser (CSRF on WebSocket), potentially scanning internal networks or connecting to hosts the user has access to.

**Fix:**
```go
var upgrader = websocket.Upgrader{
    ReadBufferSize:  4096,
    WriteBufferSize: 4096,
    CheckOrigin: func(r *http.Request) bool {
        origin := r.Header.Get("Origin")
        // Allow only expected origins
        return origin == "http://localhost:5173" || origin == "http://localhost:8080"
    },
}
```

---

### CR-03: Unauthenticated SSRF — proxy can reach any internal host

**File:** `be/main.go:90-113`
**Issue:** The `/ws` endpoint accepts arbitrary `host` and `port` values with no authentication, no rate limiting, and no allowlist. An attacker can use this as an SSRF vector to probe internal networks, connect to cloud metadata endpoints (e.g., `169.254.169.254`), or brute-force SSH credentials on internal hosts. The server effectively acts as an open SSH relay.

**Fix:** At minimum for production:
1. Add authentication to the WebSocket endpoint (session token, API key)
2. Restrict target hosts to an allowlist or private network ranges
3. Add rate limiting per source IP
4. Validate port is in range 1-65535

For the spike, document this as accepted risk.

---

### CR-04: `jsonEscape` produces invalid JSON for many inputs

**File:** `be/main.go:294-299`
**Issue:** The manual `jsonEscape` function only escapes `\`, `"`, and `\n`. It misses tabs (`\t`), carriage returns (`\r`), backspace (`\b`), form feeds (`\f`), and all other control characters (U+0000–U+001F). SSH error messages frequently contain these characters. If a tab or control char appears in an SSH error, the resulting JSON string will be malformed, causing `JSON.parse` to throw on the client — the error message is lost and the user sees no feedback.

Example: An SSH error like `"connection\trefused"` produces `{"type":"error","message":"connection\trefused"}` — the unescaped `\t` makes this invalid JSON.

**Fix:**
```go
func jsonEscape(s string) string {
    var buf strings.Builder
    buf.Grow(len(s))
    for _, r := range s {
        switch {
        case r == '\\':
            buf.WriteString(`\\`)
        case r == '"':
            buf.WriteString(`\"`)
        case r == '\n':
            buf.WriteString(`\n`)
        case r == '\r':
            buf.WriteString(`\r`)
        case r == '\t':
            buf.WriteString(`\t`)
        case r < 0x20:
            fmt.Fprintf(&buf, `\u%04x`, r)
        default:
            buf.WriteRune(r)
        }
    }
    return buf.String()
}
```
Or better: use `json.Marshal` for the entire error message structure instead of hand-building JSON strings.

---

## Warnings

### WR-01: SSH host key verification disabled

**File:** `be/main.go:112`
**Issue:** `ssh.InsecureIgnoreHostKey()` disables host key verification, making the SSH connection vulnerable to man-in-the-middle attacks. An attacker on the network path between the proxy and the SSH server can impersonate the target server and capture the password.

**Fix:** For production, implement a `HostKeyCallback` that verifies against a known_hosts file or accepts on first use (TOFU). For the spike, document this as a known limitation.

---

### WR-02: No port bounds validation on backend

**File:** `be/main.go:95-97`
**Issue:** The Go code defaults port to 22 if 0, but does not validate that the port is in the valid range 1-65535. A negative port or port > 65535 will produce a malformed address string for `ssh.Dial`. The frontend (line 23) also allows `parseInt` to produce NaN (which falls back to 22) but doesn't validate the upper bound.

**Fix:**
```go
if connectMsg.Port <= 0 || connectMsg.Port > 65535 {
    connectMsg.Port = 22
}
```

---

### WR-03: Goroutine cleanup relies on deferred resource closes, not context cancellation

**File:** `be/main.go:181-286`
**Issue:** The three goroutines use a `select { case <-ctx.Done(): return; default: }` pattern before each blocking read. This only checks context cancellation at the top of the loop — once the goroutine enters `conn.ReadMessage()` or `pipe.Read()`, it blocks until data arrives or the underlying connection is closed. Context cancellation alone does NOT unblock these reads. The goroutines ultimately exit because `handleWS` defers `conn.Close()` and `session.Close()`, but this is fragile and undocumented. If the defers were reordered or the close logic changed, goroutines would leak.

**Fix:** Use `conn.SetReadDeadline()` with periodic refresh, or use `conn.ReadDeadline` in a loop. Alternatively, close the SSH session explicitly before returning rather than relying solely on defers:
```go
// After <-done:
cancel()
session.Close()  // Force-close pipes to unblock readers
conn.Close()     // Force-close WS to unblock ReadMessage
```

---

### WR-04: No WebSocket cleanup on component unmount

**File:** `fe/src/App.tsx:28`
**Issue:** When the `App` component unmounts while connected (e.g., during React strict-mode double-render, or navigation away), the WebSocket connection stored in `wsRef.current` is never closed. There's no `useEffect` cleanup function and no `beforeunload` handler. This leaks WebSocket connections and the associated SSH sessions on the backend.

**Fix:**
```typescript
useEffect(() => {
    return () => {
        if (wsRef.current) {
            wsRef.current.close();
            wsRef.current = null;
        }
    };
}, []);
```

---

## Info

### IN-01: Vite proxy config is unused for WebSocket

**File:** `fe/vite.config.ts:9-12` and `fe/src/App.tsx:27`
**Issue:** The Vite config defines a `/ws` proxy to `ws://127.0.0.1:8080`, but the frontend connects directly to `ws://${wsHost}:8080/ws` instead of using the relative path `/ws`. The proxy configuration is dead code. Either the frontend should use `ws://${window.location.host}/ws` (which goes through the proxy in dev), or the proxy config should be removed.

**Fix:** Use the proxy in development:
```typescript
const socket = new WebSocket(
    `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`
);
```

---

### IN-02: Password persists in React state after connection

**File:** `fe/src/App.tsx:16`
**Issue:** The password is stored in React state (`useState`) and remains in memory for the entire component lifecycle. Even after the connection is established and the password is no longer needed, it stays accessible via React DevTools. For a spike this is acceptable, but for production, consider clearing the password from state after sending the connect message.

**Fix:**
```typescript
socket.onopen = () => {
    const connectMsg = JSON.stringify({ type: 'connect', host, port: portNum, user, password });
    socket.send(connectMsg);
    setPassword(''); // Clear password from memory after use
};
```

---

_Reviewed: 2026-04-27T12:30:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
