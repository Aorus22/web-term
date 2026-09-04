---
gsd_state_version: 1.0
milestone: v0.5.0
milestone_name: Desktop GPUI Client
status: executing
last_updated: "2026-09-05T00:10:00.000Z"
last_activity: 2026-09-05
progress:
  total_phases: 8
  completed_phases: 7
  total_plans: 26
  completed_plans: 23
  percent: 88
---

performance_metrics:
  velocity:
    total_plans_completed: 65
    average_duration_min: 14
  by_phase:
    - { phase: "12-local-terminal-foundation", plans: 2, total_min: 45, avg_min: 22.5 }
    - { phase: "13-sftp-backend-core", plans: 1, total_min: 15, avg_min: 15 }
    - { phase: "14-sftp-frontend-ui", plans: 2, total_min: 4, avg_min: 2 }
    - { phase: "15-sftp-operations-dnd", plans: 2, total_min: 20, avg_min: 10 }
    - { phase: "16-polish-ui-themes", plans: 3, total_min: 30, avg_min: 10 }
    - { phase: "17-terminal-theme-sync", plans: 3, total_min: 45, avg_min: 15 }
    - { phase: "18-terminal-engine-selector", plans: 3, total_min: 45, avg_min: 15 }
    - { phase: "19-review-fixes", plans: 1, total_min: 20, avg_min: 20 }
    - { phase: "20-desktop-foundation-backend-integration", plans: 4, total_min: 60, avg_min: 15 }
    - { phase: "21-terminal-rendering-alacritty-engine", plans: 3, total_min: 40, avg_min: 13.3 }
    - { phase: "22-ssh-terminal-sessions-tabs", plans: 4, total_min: 50, avg_min: 12.5 }
    - { phase: "23-hosts-ssh-keys-management", plans: 4, total_min: 50, avg_min: 12.5 }
    - { phase: "24-local-terminal-cross-platform-pty", plans: 3, total_min: 35, avg_min: 11.7 }
    - { phase: "25-sftp-dual-pane-manager", plans: 3, total_min: 45, avg_min: 15 }
    - { phase: "26-port-forwarding-settings-theme-polish", plans: 2, total_min: 30, avg_min: 15 }

quick_tasks:
  completed:
    - { slug: "20260505-fix-xterm-theme-sync", date: "2026-05-05", description: "Fixed XTerm engine theme synchronization to follow application theme presets." }
    - { slug: "20260505-fix-wterm-full-size", date: "2026-05-05", description: "Fixed wterm sizing and tmux status bar visibility issues." }
    - { slug: "20260503-sftp-context-dnd", date: "2026-05-03", description: "Context Menu & Drag-and-Drop" }

accumulated_context:
  decisions:
    - "@wterm/react handles SSH workloads cleanly"
    - "WebSocket proxy pattern is stable for bidirectional I/O"
    - "SQLite with AES-256-GCM is sufficient for single-user credential storage"
    - "No-auth v1 is appropriate for self-hosted use cases"
    - "Theme preferences synced with terminal via localStorage"
    - "Session re-attachment works via unique session IDs mapped to active SSH connections"
    - "Local terminal access added via creack/pty"
    - "D-01: Standalone SSH connections for SFTP"
    - "D-06: Streaming for file uploads/downloads"
    - "Phase 14: Use shadcn/ui Resizable for dual-pane SFTP layout"
    - "Used internal path resolution (getParentPath) for `..` navigation to avoid unnecessary backend calls."
    - "Prepended `..` virtual entry in frontend logic to ensure consistency across connections."
    - "v0.5.0: Desktop client uses GPUI + alacritty_terminal (Zed-proven engine) frontend over the existing Go backend spawned as a local child process"
    - "Phase 20: Pinned gpui-pre 0.3.3 and gpui-pre-platform 0.3.3 to guarantee exact binary compatibility with gpui-component 0.6.0"
    - "Phase 20: Decoupled Tokio supervisor runner from GPUI async executor using an unbounded MPSC channel (SupervisorEvent)"
    - "Phase 20: Stored window bounds dividing Pixels by px(1.0) and persisting on close with degenerate geometry protection"
    - "Phase 21: Recorded explicit NO-GO on crates.io gpui-terminal 0.1.0 due to unresolvable gpui 0.2.2 vs gpui-pre 0.3.3 dependency conflict; vendored clean in-tree engine in desktop-gpui/crates/terminal"
    - "Phase 21: Used native GPUI clipboard API (cx.write_to_clipboard / cx.read_from_clipboard) for seamless terminal copy/paste"
    - "Phase 21: Implemented background quad coalescing and text run batching for 60+ FPS GPUI rendering"
    - "Phase 21: Mapped mouse click count (1 = simple, 2 = semantic word, 3+ = line) and keystroke escape sequence translation with APP_CURSOR support"
    - "Phase 22: Pinned tokio-tungstenite = 0.26.2 and futures-util = 0.3.32 to match gpui-pre 0.3.3 futures tree"
    - "Phase 22: TerminalWsHandle separates binary PTY frames from JSON control frames (ready, resize, get-cwd, disconnect)"
    - "Phase 22: Exponential backoff reconnect loop (2s, 4s, 6s, 8s, 16s; max 5) paired with GPUI amber countdown banner and re-attachment"
    - "Phase 22: App restart session persistence in settings.json with detached session re-attach discovery"
    - "Phase 22: First-class Local Shell launcher modal with single-click start and Quick SSH connection option"
    - "Phase 23: Mirrored backend connection and key models with serde defaults matching Go backend"
    - "Phase 23: Persistent Hosts Catalog tab in tab strip with reactive switching to/from terminal sessions"
    - "Phase 23: Session-scoped SSH key passphrase caching in memory strictly excluding sensitive passphrases from settings disk storage"
    - "Phase 23: Self-contained RFC 4648 Base64 encoder avoiding external dependency version churn"
    - "Phase 24: Formalized architecture decision (CONPTY-DECISION.md) standardizing on Go backend ConPTY path over portable-pty to preserve unified WebSocket pipeline and cross-restart durability"
    - "Phase 24: Extended WsConnectRequest::for_local_with_cwd supporting custom/inherited working directories across Windows ConPTY and Linux POSIX PTY"
    - "Phase 24: Elevated Local Shell to prominent first position in New Tab modal with OS-specific PTY badge (ConPTY / POSIX PTY)"
    - "Phase 24: Automated CI smoke test exercising real backend supervisor, local PTY connect, echo I/O, and resize on Windows and Linux"
    - "Phase 25: Pinned reqwest multipart feature in desktop-gpui Cargo.toml for streaming SFTP file upload"
    - "Phase 25: Implemented dual-pane manager with independent source selector dropdowns (Local Filesystem vs saved SSH connections)"
    - "Phase 25: Added interactive New Folder, Rename, and Delete modals with presets, input validation, and permanent deletion confirmation"
    - "Phase 25: Collapsible bottom transfer drawer displaying streaming byte counts, transfer direction, percentages, and status tags"
    - "Phase 25: Wired inter-pane drag-and-drop (.on_drag + .on_drop::<SftpDraggedItem>) and host OS file manager drop (.on_drop::<ExternalPaths>)"
    - "Phase 25: Right-click context menu and desktop keyboard shortcuts (Enter, Backspace, Alt+Up, F2, Delete, F5)"
    - "Phase 25: Automated end-to-end integration test exercising real supervisor backend local filesystem lifecycle"
    - "Phase 26: Added port forwarding REST endpoints and GPUI management UI with live start/stop, presets, and directional port mapping"
    - "Phase 26: Enforced SET-01 architectural rule: Desktop settings features a fixed Alacritty Terminal card with strictly NO engine selector dropdown"
    - "Phase 26: Added TerminalView::set_palette for mid-session live color palette synchronization across all active terminal sessions"
  roadmap_evolution:
    - "v0.2.0: MVP Terminal & Connections"
    - "v0.3.0: SSH Key Auth & UI Redesign"
    - "v0.4.0: Local Terminal & SFTP"
    - "v0.5.0: Desktop GPUI Client — roadmap created with Phases 20-27"
    - "Phase 17 added: Sync terminal theme selection to overall application theme"
    - "Phase 18 added: Terminal Engine Selector: Allow users to switch between @wterm/react and xterm.js in settings"
  pending_todos: []
  blockers_concerns: []
deferred_items: []
session_continuity:
  last_session: 2026-09-05
  stopped_at: Completed Phase 26
  resume_file: null
  next_step: Plan Phase 27 — Parity Audit & Desktop Hardening

## Current Position

Phase: Phase 27 (Parity Audit & Desktop Hardening)
Plan: Ready to plan
Status: Advancing to Phase 27
Last activity: 2026-09-05 — Completed Phase 26 (Port Forwarding, Settings & Theme Polish)

