# Phase 6: Frontend UI & Navigation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 06-frontend-ui-navigation
**Areas discussed:** Card layout & design, SSH Keys page layout, Auth toggle in connection form, Sidebar tab navigation

---

## Card Layout & Design

| Option | Description | Selected |
|--------|-------------|----------|
| Cards in main area | Sidebar = nav only, main area shows host cards | ✓ |
| Cards stay in sidebar | Sidebar contains cards in tabs | |

**User's choice:** Cards in main area
**Notes:** Sidebar hanya isi pilihan menu hosts dan ssh keys

| Option | Description | Selected |
|--------|-------------|----------|
| Icon sidebar | ~60px icon-only nav strip | |
| Text sidebar | ~200px with text labels | ✓ |

**User's choice:** Text sidebar

| Option | Description | Selected |
|--------|-------------|----------|
| Grid layout | 2-3 columns of cards | ✓ |
| List layout | Single column wider cards | |
| You decide | Planner decides | |

**User's choice:** Grid layout

| Option | Description | Selected |
|--------|-------------|----------|
| Kebab menu | ⋯ menu with Connect/Edit/Duplicate/Delete | ✓ |
| Hover action buttons | Icon buttons on hover | |

**User's choice:** Kebab menu

| Option | Description | Selected |
|--------|-------------|----------|
| Standard card | Icon, label, user@host, tags, auth badge, kebab | |
| Minimal card | Just label and host | ✓ |
| You decide | Planner decides | |

**User's choice:** Minimal card
**Notes:** Just label + host:port always visible, tags always visible too

| Option | Description | Selected |
|--------|-------------|----------|
| Click = Connect | Opens SSH session immediately | ✓ |
| Click = Detail view | Shows connection info first | |

**User's choice:** Click = Connect
**Notes:** "Klik ke tab paling akhir dari koneksi itu. Kalau belum ada bikin tab baru"

| Option | Description | Selected |
|--------|-------------|----------|
| CTA empty state | Icon + text + Create button | ✓ |
| You decide | Planner decides | |

**User's choice:** CTA empty state

| Option | Description | Selected |
|--------|-------------|----------|
| Highlight + status dot | Active cards get highlight and green dot | ✓ |
| No card indication | No visual difference for active sessions | |

**User's choice:** Highlight + status dot

| Option | Description | Selected |
|--------|-------------|----------|
| FAB + button | Floating action button in grid area | ✓ |
| Toolbar button | Top toolbar with New Connection button | |
| Quick-connect at top | Quick-connect bar at top of grid | |

**User's choice:** FAB + button

| Option | Description | Selected |
|--------|-------------|----------|
| Tags on hover | Tags shown on hover only | |
| No tags on cards | Remove tags from cards | |
| Tags always visible | Tags always visible as badges | ✓ |

**User's choice:** Tags always visible

---

## SSH Keys Page Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Card grid | Keys as cards in a grid | ✓ |
| Table layout | Table with columns for each metadata | |
| You decide | Planner decides | |

**User's choice:** Card grid

| Option | Description | Selected |
|--------|-------------|----------|
| Drag & drop + picker | File drop zone and file picker | |
| Form upload only | Traditional form upload | |

**User's choice:** Both file and paste text
**Notes:** "Bisa dalam bentuk file, bisa juga copas isi teksnya"

| Option | Description | Selected |
|--------|-------------|----------|
| Name + type + fingerprint | Full metadata on card | |
| Name + type only | Minimal, fingerprint on hover | ✓ |
| You decide | Planner decides | |

**User's choice:** Name + type only

| Option | Description | Selected |
|--------|-------------|----------|
| Kebab: Rename + Delete | Kebab with inline rename + delete confirm | ✓ |
| You decide | Planner decides | |

**User's choice:** Kebab: Rename + Delete

| Option | Description | Selected |
|--------|-------------|----------|
| Always-visible upload | Upload area always at top of page | |
| Right-side Sheet | Sheet form like ConnectionForm | ✓ |
| Left-side Sheet | Sheet opens from left | |

**User's choice:** Right-side Sheet
**Notes:** "Pakek sheet di sebelah kanan. Termasuk disitu bisa milik plain teks atau file untuk ssh key nya"

| Option | Description | Selected |
|--------|-------------|----------|
| CTA empty state | Icon + text + Upload button | ✓ |
| You decide | Planner decides | |

**User's choice:** CTA empty state

---

## Auth Toggle in Connection Form

| Option | Description | Selected |
|--------|-------------|----------|
| Toggle replaces password | Toggle between password/key modes | |
| Both visible, radio | Both fields, radio to select | |
| Three-way toggle | Password / Key / Key+Password | |
| Both fields, no toggle | Password + key dropdown both always visible | ✓ |

**User's choice:** Both fields, no toggle
**Notes:** Connection stores both password AND SSH key reference. "Bukan passphrase, tapi password user di target machine" — password stored alongside key for sudo/other purposes.

| Option | Description | Selected |
|--------|-------------|----------|
| Dropdown select | Select with key names + types | ✓ |
| You decide | Planner decides | |

**User's choice:** Dropdown select

| Option | Description | Selected |
|--------|-------------|----------|
| Include 'Manage Keys' link | Link to SSH Keys page in dropdown | ✓ |
| Dropdown only | Just the selector | |

**User's choice:** Include 'Manage Keys' link

---

## Sidebar Tab Navigation

| Option | Description | Selected |
|--------|-------------|----------|
| Vertical text nav | Two stacked text items with icons | ✓ |
| You decide | Planner decides | |

**User's choice:** Vertical text nav

| Option | Description | Selected |
|--------|-------------|----------|
| Hosts page only | Quick-connect on Hosts page | |
| Always in sidebar | Always visible | |
| Remove it | Only via card click | |

**User's choice:** Quick-connect stays in NewTabView only
**Notes:** "Quick connect hanya ada di new tab default aja"

| Option | Description | Selected |
|--------|-------------|----------|
| Theme in sidebar header | Move theme toggle to sidebar | |
| You decide | Planner decides | |

**User's choice:** Theme toggle stays in TabBar
**Notes:** "Theme toggle tetep di baris tab paling kanan"

---

## the agent's Discretion

- Grid column count and responsive breakpoints
- Card styling details (shadows, borders, spacing)
- Sidebar transition animations
- Tag filter placement on Hosts page
- FAB button exact position
- Key card fingerprint display on hover
- Upload sheet field layout and validation
- Error states for upload failures
- "Manage SSH Keys" navigation mechanism

## Deferred Ideas

None — discussion stayed within phase scope.
