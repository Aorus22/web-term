# Feature Research

**Domain:** Native desktop SSH client (GPUI frontend over existing Go backend)
**Researched:** 2026-09-04
**Confidence:** MEDIUM-HIGH (competitor landscape well documented; parity baseline is our own validated requirements)
**Note:** Produced inline by the orchestrator (generic inline workaround — no subagent runtime in this session).

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = the desktop app feels incomplete vs both the web app and competing clients.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Tabbed terminal sessions | Every client (Termius, Tabby, Warp, Zed) has tabs; web app already does | MEDIUM | GPUI tab bar or gpui-component tabs; status dots parity |
| SSH connect with password + key auth | Core purpose of the product; both auth modes already exist backend-side | LOW | Reuses backend REST/WS as-is |
| Copy/paste + text selection in terminal | Without selection, a terminal is unusable for real work | MEDIUM-HIGH | gpui-terminal marks selection "planned" — likely the decisive spike item |
| Terminal scrollback | Long command output must be reviewable | MEDIUM | gpui-terminal scrollback "planned" — same spike decision |
| Resize-aware rendering (grid reflow) | Windows are resized constantly; PTY size must track | LOW | Resize callback pattern is documented in gpui-terminal |
| Connection manager (CRUD, tags, search) | Table stakes for any SSH client; already validated in web app | LOW | Backend API exists; UI is cards + kebab menus in GPUI |
| Dark/light theme | Web app parity; desktop users expect OS-aware theming | LOW | gpui-component theming + terminal palette sync |
| Keyboard shortcuts (new tab, close tab, switch) | Terminal users live on the keyboard (web: Ctrl+T/W/Tab) | LOW | GPUI key bindings; intercept via key handler |
| Reconnection UX on WS drop | Web app validated this; desktop adds process-restart scenarios | MEDIUM | Detect child-process death separately from SSH-session drop |
| Settings persistence | Window state, theme, backend path | LOW | Native config dir via dirs; SSH data stays backend-side |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Native GPU-rendered terminal (alacritty core) | Smoother than any webview terminal; the Zed engine users trust | MEDIUM | The milestone's headline |
| Dual-pane SFTP with drag-and-drop | Termius-grade file management; already designed in web app | HIGH | Port interaction model to GPUI; drag-and-drop across panes is the cost center |
| One-click app (no server deploy) | Desktop spawns the backend locally — zero-install feel vs web deployment | MEDIUM | Backend-as-child-process with health polling |
| Local terminal tab | Shell on the desktop host itself, alongside SSH tabs | MEDIUM (POSIX) / HIGH (Windows) | Windows needs a ConPTY decision — see ARCHITECTURE.md |
| Command palette | Termius X made this a headline feature; natural in GPUI | MEDIUM | gpui-component has palette primitives; nice stretch goal, not parity-required |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Terminal engine selector (xterm.js vs @wterm/react) | Exists in the web app | Meaningless on desktop — engine is fixed to alacritty_terminal in GPUI | Settings page simply omits the selector |
| Built-in SSH implementation toggle (russh) | "Native feels purer" | Discards the proven, encrypted, session-persistent backend; doubles security surface | Keep Go backend as single SSH/SFTP implementation |
| Auto-update framework | Desktop convention | Scope creep for parity milestone; GPUI ecosystem has no standard updater yet | Ship installers; add updater as v0.6 candidate |
| System tray / global hotkey | Desktop-native polish | User explicitly deferred extras; GPUI tray support is immature cross-platform | Park as future milestone items |

## Feature Dependencies

```
Backend-as-child-process
    └──requires--> WS/REST client (backend-client crate)
                       └──requires--> Auth surface (hosts, keys) parity
                                          └──requires--> Terminal tab (SSH)
                                                             └──requires--> Terminal view (gpui-terminal or vendored)
                                                                                └──requires--> gpui app shell (window, tabs, theme)

Local terminal tab ──requires--> PTY path (backend POSIX / ConPTY decision)
SFTP dual-pane ──requires--> WS/REST client + terminal-adjacent session state
Port forwarding UI ──requires--> backend-client (forward CRUD already in API)
Settings ──requires--> app shell; ──enhances──> theme sync
```

### Dependency Notes

- **Terminal tab requires terminal view:** the alacritty/GPUI spike is the root risk; run it first.
- **SFTP requires backend-client but not the terminal view:** SFTP can proceed in parallel after the client crate exists, though sharing connection state is simpler sequentially.
- **Theme sync requires app shell + terminal palette:** do theming after the first terminal renders, not before.

## MVP Definition

### Launch With (v0.5.0 — parity baseline)

- [ ] GPUI app shell — window, tabs, nav (Hosts/SSH Keys/SFTP/Settings), theming — the desktop's frame
- [ ] Backend lifecycle — spawn/health/kill the Go backend; connection with reconnect UX
- [ ] Terminal view — alacritty_terminal rendering in GPUI incl. selection + scrollback (spike-gated)
- [ ] SSH + local terminal tabs — connect, interact, resize, status dots, shortcuts
- [ ] Hosts management — cards, kebab menus, tags, search, quick-connect, import/export
- [ ] SSH keys — pool, upload, passphrase flow, per-connection auth method
- [ ] SFTP dual-pane — browse, transfer ops, drag-and-drop, local/remote panes
- [ ] Port forwarding UI + settings (desktop-scope settings; engine selector intentionally absent)

### Add After Validation (v0.6 candidates)

- [ ] Command palette — once core flows feel native
- [ ] System tray / global hotkey — user deferred by choice
- [ ] Auto-updater — after installers stabilize

### Future Consideration (v2+)

- [ ] macOS target — deferred by user this milestone
- [ ] Team/vault features — still out of scope per PROJECT.md
- [ ] Mobile — unchanged, out of scope

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Terminal view (alacritty in GPUI) | HIGH | HIGH | P1 |
| Backend-as-child-process | HIGH | MEDIUM | P1 |
| App shell (tabs/nav/theme) | HIGH | MEDIUM | P1 |
| SSH terminal tabs | HIGH | MEDIUM | P1 |
| Hosts management | HIGH | LOW | P1 |
| SSH keys management | HIGH | LOW | P1 |
| Local terminal | MEDIUM | MEDIUM-HIGH | P2 |
| SFTP dual-pane | HIGH | HIGH | P1 (can start parallel to terminal once client exists) |
| Port forwarding UI | MEDIUM | LOW | P2 |
| Settings page | MEDIUM | LOW | P2 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | Termius | Tabby | Zed terminal | Our Approach |
|---------|---------|-------|--------------|--------------|
| Terminal engine | Proprietary/web | Web-based (xterm.js) | alacritty_terminal in GPUI | Same engine as Zed via alacritty_terminal |
| Tabs + status | Yes (Termius X nav) | Yes | Yes | GPUI tabs with status dots, parity with web app |
| SFTP | Yes (paid tiers) | Yes (plugin) | No | First-class dual-pane, parity with web v0.4 |
| Connection mgmt | Vaults, sync | Yes | N/A | Local SQLite via backend, cards + tags + quick-connect |
| Port forwarding | Yes | Yes | N/A | Surface existing backend capability |
| Native perf | Mixed (Electron) | Electron | GPU-native | GPU-native is our differentiator |

## Sources

- Termius feature pages + Termius X announcement (nav, palette, vaults) (HIGH)
- Tabby feature listings (SSH/SFTP/port-forwarding/key management) (HIGH)
- zed.dev/docs/terminal — Zed terminal engine and features (HIGH)
- WebTerm PROJECT.md v0.2–v0.4 validated requirements — parity baseline (HIGH, internal)
- reddit selfhosted client comparisons (MEDIUM — perception of table stakes)

---
*Feature research for: WebTerm Desktop (GPUI client over Go backend)*
*Researched: 2026-09-04*
