# Phase 2: Connection Management - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 02-connection-mgmt
**Areas discussed:** Sidebar & Layout, Connection Form & Validation UX, Quick-Connect Bar, Data Model & API, shadcn Setup & Components, Security Fixes, Go Backend Structure, React Frontend Architecture

---

## Sidebar & Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Always-visible left sidebar | Fixed left panel (~260px), main area for terminal. Standard for dev tools. | |
| Collapsible sidebar | Left sidebar that can be toggled open/closed. Toggle button in tab bar. | ✓ |
| Drawer/overlay panel | Sidebar slides over content on demand. Maximum terminal space. | |

**User's choice:** Collapsible sidebar with tab bar at top. Toggle button at the left edge of the tab bar area.
**Notes:** User described "ada tab diatas, nah tombol untuk buka tutup sidebar ada di pojok kiri tempat tab itu" — tab bar across top, sidebar toggle at far left of tab area.

| Option | Description | Selected |
|--------|-------------|----------|
| Flat list + tag filters | All connections flat, tags as badges, filter row at top. | ✓ |
| Grouped by tags (accordion) | Connections grouped under collapsible tag sections. | |
| Search + flat list | Flat list with search input. No tag grouping. | |

**User's choice:** Flat list + tag filters

| Option | Description | Selected |
|--------|-------------|----------|
| Click to connect + context menu | Single click connects, right-click/"..." for actions. | ✓ |
| Click to select + action buttons | Click selects, action buttons appear inline. | |
| Click to expand + inline actions | Click expands item, shows details + actions. | |

**User's choice:** Click to connect + context menu

| Option | Description | Selected |
|--------|-------------|----------|
| Just tabs + toggle icon | Minimal: tab bar shows open tabs, sidebar toggle at far left. | ✓ |
| Tabs + quick-connect input | Tab bar includes quick-connect input field. | |

**User's choice:** Just tabs + toggle icon when collapsed

---

## Connection Form & Validation UX

| Option | Description | Selected |
|--------|-------------|----------|
| Slide-over panel from right | Form slides in from right, overlays content. Modern pattern. | ✓ |
| Modal dialog (centered) | Centered overlay. Classic, focused. | |
| Inline in sidebar | Form replaces sidebar content. Limited space. | |

**User's choice:** Slide-over panel from right

| Option | Description | Selected |
|--------|-------------|----------|
| Essential + tags | Label, Host, Port, Username, Password (opt), Tags. | ✓ |
| Essential only | Label, Host, Port, Username. Tags separate. | |
| Full with groups | Label, Host, Port, Username, Password, Tags, Group/Folder. | |

**User's choice:** Essential + tags

| Option | Description | Selected |
|--------|-------------|----------|
| Inline validation on blur | Validate on tab-out, red border + error, submit disabled. | ✓ |
| Validate on submit | All validation on Save. Scroll to first error. | |
| Progressive — validate as you type | Real-time validation on every keystroke. | |

**User's choice:** Inline validation on blur

| Option | Description | Selected |
|--------|-------------|----------|
| Confirmation dialog | "Are you sure?" dialog before delete. | ✓ |
| Undo toast | Delete immediately, toast with Undo for 5 seconds. | |

**User's choice:** Confirmation dialog

---

## Quick-Connect Bar

| Option | Description | Selected |
|--------|-------------|----------|
| Top of sidebar | Search-like input at top of sidebar. Always visible when open. | ✓ |
| Command palette (Cmd+K) | Keyboard-triggered overlay. Power-user, hidden by default. | |
| Header bar input | Always-visible input in top tab/header area. | |

**User's choice:** Top of sidebar

| Option | Description | Selected |
|--------|-------------|----------|
| Don't save, one-time | Ephemeral — connects without saving. Can save later via context menu. | ✓ |
| Auto-save with label as host | Auto-saves using host as label. | |

**User's choice:** Don't save, one-time

| Option | Description | Selected |
|--------|-------------|----------|
| Connect-only, no search | Pure quick-connect. Searching via sidebar list. | |
| Dual: connect + search connections | Type to search existing (autocomplete) OR type new address. | ✓ |

**User's choice:** Dual-purpose — search + connect

---

## Data Model & API

| Option | Description | Selected |
|--------|-------------|----------|
| JSON array in connections table | `tags` TEXT column with JSON array. Simple, SQLite JSON functions. | ✓ |
| Join table (normalized) | Separate `tags` and `connection_tags` tables. | |
| Comma-separated string | TEXT column with CSV. No query support. | |

**User's choice:** JSON array in connections table

| Option | Description | Selected |
|--------|-------------|----------|
| REST JSON API | GET/POST/PUT/DELETE /api/connections. Standard. | ✓ |
| RPC-style JSON API | POST /api/connections.create, etc. | |

**User's choice:** REST JSON API

| Option | Description | Selected |
|--------|-------------|----------|
| Full dump, import merges | Export all as JSON, import merges (skip duplicates). | ✓ |
| Selective export + full replace | Choose which to export, import replaces all. | |

**User's choice:** Full dump export, merge-on import

| Option | Description | Selected |
|--------|-------------|----------|
| Don't store passwords | User enters at connect time. Most secure. | |
| Store encrypted passwords | AES-256-GCM with server-side key. | ✓ |
| Store plaintext | Simplest, insecure. | |

**User's choice:** Store encrypted passwords (AES-256-GCM with server-side key)

---

## shadcn/ui Setup & Components

| Option | Description | Selected |
|--------|-------------|----------|
| Full shadcn with Tailwind v4 | shadcn CLI + Tailwind v4. Latest, matches Vercel aesthetic. | ✓ |
| Full shadcn with Tailwind v3 | Stable Tailwind v3. More docs available. | |
| Manual shadcn (copy components) | Manually copy only needed components. | |

**User's choice:** Full shadcn with Tailwind v4

| Option | Description | Selected |
|--------|-------------|----------|
| Core set (Button, Input, Dialog, Sheet, Dropdown, ScrollArea, Badge, Tooltip) | Minimal set for sidebar, forms, menus. | |
| Extended (also Card, Command, Popover, Separator) | More components upfront including Command for quick-connect. | ✓ |

**User's choice:** Extended set — also include Card, Command, Popover, Separator

---

## Security Fixes

| Option | Description | Selected |
|--------|-------------|----------|
| Fix all 4 critical issues | CR-01 through CR-04. Backend being restructured anyway. | ✓ |
| Fix CR-02, CR-03, CR-04 only | Skip TLS/wss. Handle via reverse proxy later. | |
| Fix CR-02 and CR-04 only | Minimal fixes. Leaves significant gaps. | |

**User's choice:** Fix all 4

| Option | Description | Selected |
|--------|-------------|----------|
| Self-signed TLS in Go backend | Go serves HTTPS/WSS with auto-generated cert. | |
| Reverse proxy handles TLS | nginx/caddy in front. Go stays HTTP on localhost. | ✓ |
| Auto-detect TLS | Check for certs at startup. Flexible. | |

**User's choice:** Reverse proxy handles TLS

| Option | Description | Selected |
|--------|-------------|----------|
| Allowlist of allowed hosts | Configurable patterns (*.internal, 10.0.0.*). | ✓ |
| Block dangerous ranges only | Block metadata, loopback, link-local. | |
| No restriction, accepted risk | v1 is single-user self-hosted. | |

**User's choice:** Allowlist of allowed hosts

| Option | Description | Selected |
|--------|-------------|----------|
| TOFU (Trust On First Use) | Save key on first connect, verify after. Standard SSH pattern. | |
| Skip for v1, document limitation | Keep InsecureIgnoreHostKey. Fix later. | ✓ |

**User's choice:** Skip for v1, document as known limitation

| Option | Description | Selected |
|--------|-------------|----------|
| AES-256-GCM with server-side key | Standard encryption, env var for key. | ✓ |
| Per-connection key derivation | Derive from connection data + master key. More complex. | |

**User's choice:** AES-256-GCM with server-side key

---

## Go Backend Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Standard Go layout | cmd/server, internal/api, internal/db, internal/ssh, internal/config. | ✓ |
| Flat package with main.go at root | Separate files, no package hierarchy. | |
| Domain-driven (feature folders) | internal/connections/, internal/terminal/. | |

**User's choice:** Standard Go layout

| Option | Description | Selected |
|--------|-------------|----------|
| Single server, route-based | /api/* for REST, /ws for WebSocket. Single port. | ✓ |
| Separate ports for API and WS | REST on one port, WebSocket on another. | |

**User's choice:** Single server, route-based

| Option | Description | Selected |
|--------|-------------|----------|
| Embedded SQL migration files | goose/golang-migrate with versioned SQL files. | |
| Auto-migrate with Go code | Use GORM with auto-migration from Go structs. | ✓ |
| Manual schema in init function | CREATE TABLE IF NOT EXISTS on startup. | |

**User's choice:** Auto-migrate with Go code (GORM)

| Option | Description | Selected |
|--------|-------------|----------|
| GORM | Most popular Go ORM. Auto-migration, query builder. | ✓ |
| sqlc with manual migrations | Write SQL, generate type-safe Go code. | |
| Raw database/sql + helpers | Standard library, maximum control. | |

**User's choice:** GORM

---

## React Frontend Architecture

| Option | Description | Selected |
|--------|-------------|----------|
| Feature-based folders | src/features/connections/, src/features/terminal/, src/components/ui/. | ✓ |
| Layer-based folders | src/components/, src/hooks/, src/types/. | |
| Flat src/ with colocation | Everything in src/ with descriptive names. | |

**User's choice:** Feature-based folders

| Option | Description | Selected |
|--------|-------------|----------|
| React state + fetch | useState/useReducer + custom fetch hooks. No library. | |
| TanStack Query | Server state with caching, refetching. | |
| Zustand | Lightweight global state store. Client state only. | ✓ |

**User's choice:** Zustand

| Option | Description | Selected |
|--------|-------------|----------|
| Custom fetch wrapper + hooks | Thin fetch wrapper + custom hooks per feature. | |
| Axios + interceptors | axios with base URL, error handling. | |
| TanStack Query alongside Zustand | TanStack for server state, Zustand for client state. | ✓ |

**User's choice:** TanStack Query for server state + Zustand for client state

| Option | Description | Selected |
|--------|-------------|----------|
| No router needed yet | Single-page layout. No page navigation. | ✓ |
| React Router from the start | Set up routing now for future-proofing. | |

**User's choice:** No router needed yet

| Option | Description | Selected |
|--------|-------------|----------|
| Custom useWebSocket hook | Hook manages WS lifecycle, returns send + state. | ✓ |
| Global WebSocket service class | Singleton class, imperative control. | |

**User's choice:** Custom useWebSocket hook

---

## the agent's Discretion

- Exact visual styling within Vercel-inspired aesthetic
- Component internal implementation details
- Error message wording
- Tags input UX (multi-select vs comma-separated)
- Configuration file format for SSRF allowlist and encryption key

## Deferred Ideas

None — discussion stayed within phase scope
