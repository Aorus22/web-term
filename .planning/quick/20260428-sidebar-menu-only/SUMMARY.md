---
status: complete
date: 2026-04-28
---
# Summary: Sidebar Simplified to Menu Only

Removed the connection list from the sidebar to focus strictly on navigation menus.

## Changes
- Removed `ConnectionList` and `ScrollArea` from the sidebar in `App.tsx`.
- Cleaned up unused imports in `App.tsx`.
- Narrowed the sidebar to 200px for a more compact navigation feel.
- Ensured the sidebar only contains the "WebTerm" brand and the navigation buttons for "Hosts" and "SSH Keys".

## Verification
- Code structure is simplified and build-ready.
