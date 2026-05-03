# Phase 4: Multi-Tab & Polish - Context

## Phase Boundary
Users can manage multiple SSH sessions with a polished, logical UI. This phase focuses on removing redundancy (sidebar cleanup), enhancing the "New Tab" experience, and fixing UI/UX polish issues in forms and tab management.

**Requirements covered:** TAB-01, TAB-02, TAB-03, TAB-04, UI-02, UI-03

## Implementation Decisions

### 1. New Tab / "Selection" View (TAB-01)
- **Goal:** Instead of an empty state, show a rich selection view when no session is active or a new tab is opened.
- **Components:**
    - **Grid of Connections:** Large, clickable cards for saved connections.
    - **Quick Connect:** Prominent host/user/port input at the top of this view (moved from sidebar).
- **Behavior:** Clicking a connection transforms the current "New Tab" into a `TerminalPane`.

### 2. Sidebar Cleanup & Redundancy (UI-04)
- **D-01: Remove Quick Connect from Sidebar.** It is redundant and cramped.
- **D-02: Sidebar Header.** Top of sidebar should only have a "+ New Connection" button.
- **D-03: Sidebar Content.** Only show the list of saved connections and tag filters.
- **D-04: Theme Toggle.** Move theme toggle to the header (top right) to save sidebar space.

### 3. Connection Form Polish (UI-01)
- **D-05: Padding & Spacing.** Fix the `Sheet` content in `ConnectionForm.tsx`. Add significant padding (`p-6` or `p-8`) and vertical spacing (`space-y-4` or `space-y-6`) between input groups.
- **D-06: Layout.** Ensure the sheet opens from the right (`side="right"`) and feels like a professional drawer, not a cramped popup.

### 4. Tab Bar & Lifecycle (TAB-03, TAB-04)
- **D-07: Add Tab Button.** A permanent `+` button at the right of the TabBar to open the "Selection View".
- **D-08: Status Dots.** 6px colored dots (Green=Connected, Yellow=Connecting, Red=Error/Disconnected) next to the tab label.
- **D-09: Close Confirmation.** Use an `AlertDialog` for connected sessions; immediate close for disconnected/new tabs.
- **D-10: Unmount Cleanup.** Ensure `useSSHSession` calls `disconnect()` on unmount to prevent zombie WebSockets.

## Canonical References
- `fe/src/App.tsx` — Main layout logic to be updated for New Tab view.
- `fe/src/components/TabBar.tsx` — Needs `+` button and status dots.
- `fe/src/features/connections/components/ConnectionForm.tsx` — Needs padding/spacing fix.
- `fe/src/stores/app-store.ts` — Already supports multiple sessions; no major changes needed.

## Deferred Ideas
- SSH Key Auth (v2)
- Session Recording (v2)
- Advanced Keyboard Mapping (v2)
