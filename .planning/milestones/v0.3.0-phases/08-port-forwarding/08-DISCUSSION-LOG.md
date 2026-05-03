# Phase 8: Port Forwarding - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 08-port-forwarding
**Areas discussed:** Data Model & Persistence, Backend Tunnel Architecture, Forward UI Placement, Port Conflict & Lifecycle

---

## Data Model & Persistence

| Option | Description | Selected |
|--------|-------------|----------|
| Persistent (SQLite) | Rules stored in SQLite, survive restart | |
| Session-only (in-memory) | Rules exist only while server running | |
| Hybrid | Rules in SQLite, active status in-memory, tunnels need re-activation after restart | ✓ |

**User's choice:** Hybrid — "konfigurasi ada di sqlite. tapi ada status on / off nya yang itu in memory. jadi ketika server crash or something ya harus on off kan lagi"
**Notes:** Rules are config, tunnels are runtime state.

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal fields | name, connection_id, local_port, remote_port | ✓ |
| Extended fields | Above + bind_address, protocol, description | |

**User's choice:** Minimal fields

| Option | Description | Selected |
|--------|-------------|----------|
| localhost only | Remote host always localhost (relative to SSH server) | ✓ |
| Arbitrary remote host | User can specify any remote_host:remote_port | |

**User's choice:** localhost only

---

## Backend Tunnel Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Separate SSH connection | Each forward dials its own SSH connection, decoupled from terminal | ✓ |
| Reuse terminal connection | Forward uses existing ssh.Client from terminal session | |

**User's choice:** Separate SSH connection

| Option | Description | Selected |
|--------|-------------|----------|
| REST API | /api/forwards endpoints for CRUD + start/stop | ✓ |
| Extend WebSocket | New message types in existing WebSocket protocol | |

**User's choice:** REST API

| Option | Description | Selected |
|--------|-------------|----------|
| net.Listen + ssh.Dial | Backend opens listener, forwards via SSH client | ✓ |
| ssh.DirectForward | Use ssh.Client.Listen built-in | |

**User's choice:** net.Listen + ssh.Dial

---

## Forward UI Placement

| Option | Description | Selected |
|--------|-------------|----------|
| New sidebar page | Third tab "Port Forwards" alongside Hosts and SSH Keys | ✓ |
| Per-connection in HostsPage | Forward action per host card | |
| TabBar dropdown | Global button in TabBar area | |

**User's choice:** New sidebar page

| Option | Description | Selected |
|--------|-------------|----------|
| Card list with toggle | Each rule is a card with on/off toggle, kebab menu | ✓ |
| Table layout | Tabular format with columns | |

**User's choice:** Card list with toggle

| Option | Description | Selected |
|--------|-------------|----------|
| Dropdown selector | Sheet form with dropdown listing saved connections | ✓ |
| Auto from active session | Pre-select from active terminal tab | |

**User's choice:** Dropdown selector

---

## Port Conflict & Lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Backend net.Listen check | Try binding port on start, EADDRINUSE returns error | ✓ |
| Frontend tracking only | Track assigned ports in frontend state | |

**User's choice:** Backend net.Listen check

| Option | Description | Selected |
|--------|-------------|----------|
| Keep alive | Forward stays up until explicitly stopped, independent of terminal | ✓ |
| Auto-stop | Stop forward when terminal sessions for connection close | |

**User's choice:** Keep alive (separate SSH connection)

| Option | Description | Selected |
|--------|-------------|----------|
| Poll on page load | GET /api/forwards returns status per rule | ✓ |
| WebSocket status stream | Real-time push updates | |

**User's choice:** Poll on page load

---

## the agent's Discretion

- PortForward model struct details
- Auto-suggest next available local port
- Tunnel goroutine lifecycle management
- Error message wording
- Card styling, FAB placement, empty state design
- Edit behavior for active vs inactive rules
- Local port range validation

## Deferred Ideas

None — discussion stayed within phase scope.
