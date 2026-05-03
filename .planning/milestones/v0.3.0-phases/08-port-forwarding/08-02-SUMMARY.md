---
phase: 08-port-forwarding
plan: 02
subsystem: frontend
tags: [port-forwarding, react, zustand, react-query, shadcn, sonner, sheet]
dependency_graph:
  requires: [PortForward backend API (08-01), existing Connection model, Zustand store]
  provides: [PortForwardsPage, ForwardFormSheet, forwardsApi, useForwards hooks, sidebar tab]
  affects: [fe/src/lib/api.ts, fe/src/stores/app-store.ts, fe/src/App.tsx, fe/src/features/forwards/]
tech_stack:
  added: [sonner, @base-ui/react Select, shadcn Switch]
  patterns: [React Query mutations with error toast, Sheet form, card list with toggle]
key_files:
  created:
    - fe/src/features/forwards/hooks/useForwards.ts
    - fe/src/features/forwards/components/PortForwardsPage.tsx
    - fe/src/features/forwards/components/ForwardFormSheet.tsx
    - fe/src/components/ui/sonner.tsx
    - fe/src/components/ui/switch.tsx
    - fe/src/components/ui/select.tsx
  modified:
    - fe/src/lib/api.ts
    - fe/src/stores/app-store.ts
    - fe/src/App.tsx
    - fe/package.json
decisions:
  - Used sonner for toast notifications (shadcn toast not available for base-nova style)
  - Cards displayed as vertical list (not grid) — better readability for port mapping info
  - Switch component uses emerald color for active state (distinct from primary accent)
  - Fixed sonner.tsx to use project's own useTheme hook instead of next-themes
  - Base-ui Select onValueChange handles null (not just string) — added null guard
metrics:
  duration: 5 min
  completed: 2026-04-28
  tasks: 3
  files: 11
---

# Phase 8 Plan 2: Port Forwarding Frontend Summary

Port forwards page with card list, toggle switches, create form sheet, sidebar tab, and sonner toast for port conflicts.

## What Was Built

**PortForward Type + API Client** (`fe/src/lib/api.ts`)
- `PortForward` interface: id, name, connection_id, local_port, remote_port, active, error, timestamps
- `forwardsApi` with list, create, delete, start, stop methods
- create/start reject with server error JSON for toast display

**React Query Hooks** (`fe/src/features/forwards/hooks/useForwards.ts`)
- `useForwards()` — query with ['forwards'] key
- `useCreateForward()` — mutation with cache invalidation
- `useDeleteForward()` — mutation with cache invalidation
- `useStartForward()` — mutation with cache invalidation (status changes)
- `useStopForward()` — mutation with cache invalidation

**PortForwardsPage** (`fe/src/features/forwards/components/PortForwardsPage.tsx`)
- Card list layout (vertical) with toggle switches per rule
- Each card shows: name, connection label, `localhost:local_port → :remote_port`, active badge, error text
- Toggle calls start/stop API with sonner toast on failure
- Active cards highlighted with emerald border/background
- Kebab menu with delete option + confirmation dialog
- Empty state with create CTA
- Connection label resolution via useConnections hook + Map lookup

**ForwardFormSheet** (`fe/src/features/forwards/components/ForwardFormSheet.tsx`)
- Right-side Sheet form with name, connection dropdown, local port, remote port
- Connection dropdown lists all saved connections (label + host)
- Port validation: 1-65535 on blur and submit
- Port conflict error shows sonner toast, keeps sheet open
- Success toast and auto-close on create

**Sidebar Integration** (`fe/src/stores/app-store.ts`, `fe/src/App.tsx`)
- `sidebarPage` union extended with 'forwards'
- ArrowLeftRight icon + "Port Forwards" sidebar button
- PortForwardsPage rendered when sidebarPage === 'forwards'
- Toaster component added to App

**UI Components Installed**
- `sonner` — toast notifications (fixed to use project theme hook)
- `switch` — shadcn switch component for toggles
- `select` — base-ui select component for connection dropdown

## Commits

| Commit | Message |
|--------|---------|
| 608cf9e | feat(08-02): add port forwards frontend — page, form sheet, sidebar integration |

## Must-Have Verification

| # | Truth | Status |
|---|-------|--------|
| 1 | User sees Port Forwards as sidebar tab | ✓ ArrowLeftRight icon + button in nav |
| 2 | User can create forward via sheet form with connection dropdown, port inputs | ✓ ForwardFormSheet with all fields + validation |
| 3 | Saved rules appear as cards with name, connection, port mapping, toggle | ✓ Card list with all info displayed |
| 4 | User can toggle forward on/off (start/stop) | ✓ Switch calls start/stop mutations |
| 5 | User can delete forward via kebab menu | ✓ DropdownMenu + AlertDialog confirmation |
| 6 | Port conflict shows toast error | ✓ sonner toast on create/start failure |
| 7 | Forward status visible on page load | ✓ useForwards query fetches active state from GET /api/forwards |

## Threat Mitigation Verification

| Threat ID | Mitigation | Implemented |
|-----------|-----------|-------------|
| T-08-06 | Client-side port range validation (1-65535) | ✓ ForwardFormSheet validates on blur + submit |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Installed missing shadcn components**
- **Found during:** Task 1 setup
- **Issue:** Project had no toast, switch, or select components; shadcn toast not available for base-nova style
- **Fix:** Installed sonner, switch, select via shadcn CLI. Fixed sonner.tsx to use project's own useTheme hook (not next-themes)
- **Files modified:** fe/src/components/ui/sonner.tsx, fe/package.json
- **Commit:** 608cf9e

**2. [Rule 1 - Bug] Fixed base-ui Select onValueChange type**
- **Found during:** Task 3 build verification
- **Issue:** base-ui Select onValueChange passes `string | null`, not just `string`
- **Fix:** Added null guard: `(v) => v !== null && setConnectionId(v)`
- **Files modified:** fe/src/features/forwards/components/ForwardFormSheet.tsx
- **Commit:** 608cf9e

## Self-Check: PASSED

All 6 key files exist, commit 608cf9e verified, TypeScript compiles cleanly, Vite build succeeds.
