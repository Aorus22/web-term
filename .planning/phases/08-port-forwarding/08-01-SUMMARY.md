---
phase: 08-port-forwarding
plan: 01
subsystem: backend
tags: [port-forwarding, ssh, tunnel, rest-api, gorm]
dependency_graph:
  requires: [existing Connection model, existing SSH auth patterns]
  provides: [PortForward model, /api/forwards REST API, TunnelManager]
  affects: [be/internal/db/models.go, be/internal/db/db.go, be/internal/api/forwards.go, be/internal/api/routes.go, be/internal/ssh/tunnel.go]
tech_stack:
  added: [net.Listen, ssh.Client.Dial, ssh.PublicKeys, ssh.KeyboardInteractive]
  patterns: [GORM model, REST CRUD, goroutine lifecycle, sync.Mutex]
key_files:
  created:
    - be/internal/api/forwards.go
    - be/internal/ssh/tunnel.go
  modified:
    - be/internal/db/models.go
    - be/internal/db/db.go
    - be/internal/api/routes.go
decisions:
  - TunnelManager created inside SetupRoutes for clean dependency injection
  - Port conflict detected via net.Listen bind failure (not separate registry)
  - Each forward has its own SSH connection, decoupled from terminal sessions
  - 127.0.0.1-only binding for security (no 0.0.0.0 exposure)
metrics:
  duration: 3 min
  completed: 2026-04-28
  tasks: 2
  files: 5
---

# Phase 8 Plan 1: Port Forwarding Backend Summary

Port forward CRUD API and SSH tunnel manager with net.Listen-based port conflict detection.

## What Was Built

**PortForward GORM Model** (`be/internal/db/models.go`)
- UUID primary key, name, connection_id (FK), local_port, remote_port, timestamps
- BeforeCreate hook for UUID generation, AutoMigrate registered

**REST API** (`be/internal/api/forwards.go`, `be/internal/api/routes.go`)
- `GET /api/forwards` — List all rules with active status and error info
- `POST /api/forwards` — Create rule (validates name, connection_id existence, port ranges 1-65535)
- `DELETE /api/forwards/{id}` — Delete rule (stops active tunnel first)
- `POST /api/forwards/{id}/start` — Activate tunnel (returns error on port conflict)
- `POST /api/forwards/{id}/stop` — Deactivate tunnel

**TunnelManager** (`be/internal/ssh/tunnel.go`)
- Manages active tunnels in-memory with sync.Mutex
- `Start()`: Binds local port via net.Listen on 127.0.0.1, dials SSH with key or password auth, launches accept loop goroutine
- `Stop()`: Cancels context, closes listener + SSH client, removes from map
- `IsActive()` / `GetError()`: Query active state and stored errors
- `StopAll()`: Graceful shutdown of all tunnels
- Accept loop: Each incoming connection gets its own ssh.Client.Dial + bidirectional io.Copy

## Commits

| Commit | Message |
|--------|---------|
| b7d167c | feat(08-01): add PortForward model, CRUD API, and SSH tunnel manager |

## Must-Have Verification

| # | Truth | Status |
|---|-------|--------|
| 1 | Save port forwarding rule with name, connection_id, local_port, remote_port | ✓ CreateForward validates all fields |
| 2 | Activate saved rule establishes SSH tunnel on localhost:local_port | ✓ TunnelManager.Start dials SSH + binds listener |
| 3 | Deactivate active tunnel releases local port | ✓ TunnelManager.Stop closes listener + client |
| 4 | Starting forward on occupied port returns clear error | ✓ net.Listen failure → "Port N is already in use" |
| 5 | Deleting rule also stops active tunnel | ✓ DeleteForward checks IsActive then Stop |
| 6 | GET /api/forwards returns each rule with active status and error | ✓ ListForwards enriches with IsActive/GetError |

## Threat Mitigation Verification

| Threat ID | Mitigation | Implemented |
|-----------|-----------|-------------|
| T-08-01 | Input validation (name, connection_id, ports) | ✓ CreateForward validates all fields |
| T-08-02 | Bind to 127.0.0.1 only | ✓ net.Listen("tcp", "127.0.0.1:N") |
| T-08-04 | Decrypt creds only at Start time | ✓ Decrypt in TunnelManager.Start, never stored |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

All files exist, commit verified, build compiles cleanly, go vet passes.
