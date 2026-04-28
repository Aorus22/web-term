# Phase 6: Frontend UI & Navigation - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Frontend-only phase delivering two-page sidebar navigation + card-based host management + SSH key pool management UI + connection form with key selector. Sidebar becomes navigation-only (text labels: Hosts, SSH Keys). Main content area renders the active page. Connection form gains SSH key selector alongside password field. No backend changes — consumes existing /api/keys and /api/connections endpoints.

</domain>

<decisions>
## Implementation Decisions

### Sidebar Navigation
- **D-01:** Sidebar becomes navigation-only with ~200px width, two vertical text items: "Hosts" (server icon) and "SSH Keys" (key icon). Active item highlighted.
- **D-02:** Zustand `sidebarPage` state (already decided in prior context) controls which page shows in main area. No router.
- **D-03:** Quick-connect bar stays only in NewTabView (default empty state) — NOT moved to sidebar or Hosts page.
- **D-04:** Theme toggle and other controls remain in TabBar area (top-right of main content, same as current).
- **D-05:** Sidebar does NOT contain export/import or quick-connect — those stay in their current locations or move to Hosts page toolbar.

### Hosts Page (Card Layout)
- **D-06:** Hosts displayed as cards in a grid layout (2-3 columns) in the main content area — NOT inside the sidebar.
- **D-07:** Minimal card content: label (bold) + host:port (small text) always visible. Tags always visible as small badges.
- **D-08:** Each card has kebab menu (⋯) with actions: Connect, Edit, Duplicate, Delete.
- **D-09:** Clicking a card body = connect to host. If tab already exists for that connection, switch to it. Otherwise create new session tab. Same behavior as current ConnectionList click.
- **D-10:** Active session indicator: cards with active SSH sessions get highlight border/background + status dot.
- **D-11:** FAB + button (floating action button) in the grid area to create new connection. Opens right-side ConnectionForm sheet.
- **D-12:** Empty state: centered icon + "No connections yet" + "Create Connection" button (CTA pattern).
- **D-13:** Tag filter remains available — accessible from Hosts page (exact placement at planner's discretion).

### SSH Keys Page
- **D-14:** Keys displayed as cards in a grid layout. Each card shows: key name (bold) + key type badge (RSA/Ed25519/ECDSA).
- **D-15:** Each card has kebab menu with: Rename (inline edit) and Delete (with confirmation dialog showing warning if connections reference the key).
- **D-16:** Upload via right-side Sheet (same pattern as ConnectionForm). Sheet contains: name input + key input area with toggle between paste-text and file-picker.
- **D-17:** Empty state: centered icon + "No SSH keys yet" + "Upload Key" button (CTA pattern).
- **D-18:** FAB + button for uploading new keys, opens the upload sheet.

### Connection Form Auth Integration
- **D-19:** NO toggle between password and key auth. Both fields always visible in the form.
- **D-20:** Password field stays as-is (optional). Below it, SSH key selector dropdown (optional). Connection can store both password AND a key reference.
- **D-21:** Key selector is a dropdown listing all uploaded keys by name + type. Default selection: "None".
- **D-22:** Key selector includes "Manage SSH Keys" link/button that navigates to SSH Keys page.
- **D-23:** When editing an existing connection, pre-select the currently assigned key in the dropdown.

### Connection Interface Update
- **D-24:** Frontend Connection type in api.ts must add `auth_method` ("password" | "key") and `ssh_key_id` (string | null) fields to match backend model from Phase 5.

### the agent's Discretion
- Exact grid column count and responsive breakpoints
- Card component styling details (shadows, borders, spacing)
- Sidebar transition animation details
- Tag filter placement on Hosts page
- FAB button exact position (corner, offset)
- Whether to add a search/filter bar to Hosts page
- Key card fingerprint display on hover (tooltip vs inline)
- Upload sheet exact field layout and validation messages
- Error states for key upload failures
- How "Manage SSH Keys" link navigates (sidebar state change vs direct)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` § "UI Navigation" (UI-01 through UI-05)
- `.planning/REQUIREMENTS.md` § "Connection Auth" (AUTH-01, AUTH-02)

### Prior Phase Context
- `.planning/phases/05-backend-ssh-key-storage/05-CONTEXT.md` — Backend SSH key API endpoints, SSHKey model structure, connection schema migration with auth_method and ssh_key_id fields

### Existing Frontend Patterns
- `fe/src/stores/app-store.ts` — Zustand store pattern, will need sidebarPage state added
- `fe/src/features/connections/components/ConnectionForm.tsx` — Right-side Sheet pattern for add/edit forms
- `fe/src/features/connections/components/ConnectionList.tsx` — Current list rendering with kebab menu, click-to-connect, duplicate, delete confirmation
- `fe/src/features/connections/hooks/useConnections.ts` — React Query hooks pattern (useQuery/useMutation) to replicate for SSH keys
- `fe/src/lib/api.ts` — API client pattern, needs keysApi added alongside connectionsApi
- `fe/src/components/ui/card.tsx` — shadcn Card component available for reuse
- `fe/src/components/ui/dropdown-menu.tsx` — shadcn DropdownMenu for kebab menus
- `fe/src/components/ui/sheet.tsx` — shadcn Sheet for upload form

### Backend API (built in Phase 5)
- `be/internal/api/keys.go` — SSH key CRUD API (List, Get, Create, Update, Delete)
- `be/internal/api/routes.go` — Route registration with /api/keys endpoints
- `be/internal/db/models.go` — SSHKey model and Connection model with auth_method/ssh_key_id fields

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **shadcn Card component**: Already installed at `fe/src/components/ui/card.tsx` — use for both host cards and key cards
- **shadcn DropdownMenu**: Used in ConnectionList for kebab menus — same pattern for card actions
- **shadcn Sheet**: Used by ConnectionForm — same component for SSH key upload sheet
- **AlertDialog**: Used in ConnectionList for delete confirmation — same pattern for key delete
- **useConnections hooks**: React Query pattern with invalidateQueries — replicate for useSSHKeys hooks
- **connectionsApi**: Fetch-based API client pattern — add keysApi following same structure
- **ConnectionList**: Click-to-connect logic, duplicate, delete — directly reusable for card click handlers

### Established Patterns
- **Zustand store**: `useAppStore` with setters — add `sidebarPage` state here
- **React Query**: `useQuery`/`useMutation` with `queryClient.invalidateQueries` — follow for SSH keys
- **Feature-based structure**: `fe/src/features/connections/` — create `fe/src/features/ssh-keys/` with same structure
- **Right-side Sheet forms**: ConnectionForm pattern — SSHKeyUploadForm follows same layout
- **Base64 API encoding**: Backend expects key_base64 field for key upload

### Integration Points
- Main App.tsx layout: `<aside>` sidebar + `<main>` content area — needs restructuring to add page routing
- `app-store.ts`: Add `sidebarPage: 'hosts' | 'keys'` state and setter
- `api.ts`: Add `SSHKey` interface and `keysApi` object with CRUD methods
- `Connection` interface: Add `auth_method`, `ssh_key_id` fields
- New feature directory: `fe/src/features/ssh-keys/` with components and hooks
- ConnectionForm: Add SSH key selector dropdown below password field

</code_context>

<specifics>
## Specific Ideas

- "Sidebar cuma isi pilihan menu hosts dan ssh keys" — sidebar is navigation only, content in main area
- "Klik ke tab paling akhir dari koneksi itu. Kalau belum ada bikin tab baru" — click card switches to existing tab or creates new
- "Bisa dalam bentuk file, bisa juga copas isi teksnya" — SSH key upload supports both file picker and paste text
- "Pakek sheet di sebelah kanan. Termasuk disitu bisa milik plain teks atau file untuk ssh key nya" — right-side sheet for key upload with toggle between paste/file
- "Bukan passphrase, tapi password user di target machine" — connection stores both SSH key AND user password (key for SSH auth, password for other purposes)
- "Quick connect hanya ada di new tab default aja" — quick-connect bar stays in NewTabView only
- "Theme toggle tetep di baris tab paling kanan" — controls stay in TabBar, not moved to sidebar

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-frontend-ui-navigation*
*Context gathered: 2026-04-28*
