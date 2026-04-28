---
phase: 03-ssh-terminal
plan: 01
subsystem: backend-ssh-proxy
tags: [ssh, websocket, keyboard-interactive, session-management, connection-id]
dependency_graph:
  requires:
    - "02-01 (Connection CRUD + encryption for DB password fetch)"
    - "01-01 (WebSocket SSH proxy foundation)"
  provides:
    - "Enhanced SSH proxy with connection_id protocol, keyboard-interactive auth, session tracking"
  affects:
    - "be/internal/ssh/proxy.go"
    - "be/internal/ssh/types.go"
tech_stack:
  added:
    - "ssh.KeyboardInteractive auth method"
    - "context.Context-based session cleanup"
    - "uuid-based session_id tracking"
  patterns:
    - "Dual auth flow: connection_id (saved) + host/user/password (quick-connect)"
    - "Server-side password decryption from encrypted storage"
key_files:
  created:
    - "be/internal/ssh/types.go"
  modified:
    - "be/internal/ssh/proxy.go"
decisions:
  - "Keyboard-interactive returns user password for all challenges — covers 99% of real-world KBI scenarios (D-04)"
  - "SSRF check applied to DB-fetched hosts, not just client-provided hosts (T-03-05)"
  - "Session ID generated before SSH dial for logging even on connection failure"
metrics:
  duration: 4 min
  completed: 2026-04-27
---

# Phase 03 Plan 01: Enhanced SSH Proxy Summary

WebSocket SSH proxy with dual connect flow, keyboard-interactive auth, and context-based session cleanup.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extract types + add connection_id protocol with server-side password fetch | e9b536d | be/internal/ssh/types.go (new), be/internal/ssh/proxy.go |
| 2 | Add keyboard-interactive SSH auth + context-based session cleanup | e856bdb | be/internal/ssh/proxy.go |

## What Changed

### Task 1: Extract types + connection_id protocol
- Created `types.go` with extracted `ConnectMessage`, `ResizeMessage`, and new `ServerMessage` types
- `ConnectMessage` now has optional `ConnectionID` field for saved connection flow
- When `connection_id` is provided: backend queries SQLite for the connection, decrypts password via `config.Decrypt`, populates SSH credentials
- SSRF protection applies to DB-fetched hosts (mitigates T-03-05)
- Connected response changed from `{"type":"connected"}` to `{"type":"connected","session_id":"<uuid>"}`

### Task 2: Keyboard-interactive auth + session cleanup
- Added `ssh.KeyboardInteractive` auth method alongside `ssh.Password` — returns password for all challenges
- Replaced channel-based `done` pattern with `context.Context` cancellation
- All 3 forwarding goroutines check `ctx.Done()` for clean shutdown
- Sends `{"type":"disconnected","session_id":"<uuid>"}` on clean WebSocket close

## Deviations from Plan

None — plan executed exactly as written.

## Verification Results

```
✓ go build ./cmd/server/ — clean
✓ go vet ./internal/ssh/ — clean
✓ 6 type references in types.go (ConnectMessage, ResizeMessage, ServerMessage)
✓ 5 connection_id references in proxy.go
✓ 1 KeyboardInteractive auth method
✓ 1 context.WithCancel
✓ 5 disconnected message references
```

## Threat Model Compliance

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-03-01 | Validate connection_id exists in DB | ✓ Implemented — `database.First()` returns error for unknown IDs |
| T-03-02 | Strict JSON unmarshal | ✓ Existing pattern preserved |
| T-03-03 | Password in memory | ✓ Accept — cleared on goroutine exit |
| T-03-04 | Concurrent connections | ✓ Accept — single user per PROJECT.md |
| T-03-05 | SSRF via connection_id | ✓ SSRF check applied to DB-fetched host |

## Self-Check: PASSED

- [x] `be/internal/ssh/types.go` exists
- [x] `be/internal/ssh/proxy.go` exists and builds
- [x] Commit e9b536d exists
- [x] Commit e856bdb exists
