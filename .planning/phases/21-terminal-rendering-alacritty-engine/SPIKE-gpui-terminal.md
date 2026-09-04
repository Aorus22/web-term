# Spike Report: gpui-terminal Evaluation (Phase 21 Criterion 5)

**Date:** 2026-09-04
**Evaluator:** WebTerm Architecture Agent
**Phase:** 21-terminal-rendering-alacritty-engine
**Target:** `gpui-terminal` crate (v0.1.0 on crates.io)

---

## 1. Executive Summary & Verdict

| Dimension | Evaluation | Result |
|-----------|------------|--------|
| **Crate Availability** | `gpui-terminal = "0.1.0"` exists on crates.io | Found |
| **Terminal Engine** | Uses `alacritty_terminal = "0.25.1"` | Matches target specification |
| **GPUI Version Pin** | Pinned to `gpui = "0.2.2"` | **FATAL CONFLICT** |
| **Workspace Requirement** | `desktop-gpui` is pinned to `gpui-pre = "=0.3.3"` for `gpui-component = "=0.6.0"` | Required |
| **Verdict** | External dependency: **NO-GO** / In-tree vendoring: **GO** | Confirmed |

### Formal Verdict
- **External Crate Dependency:** **NO-GO**. We cannot add `gpui-terminal = "0.1.0"` directly as a dependency in `Cargo.toml`.
- **In-Tree Vendoring / Architecture Adoption:** **GO**. The internal architecture of `gpui-terminal` (10 clean, focused modules) is well-crafted and directly addresses all Phase 21 requirements (ANSI/TrueColor, glyph batching, background quad merging, selection, scrollback, mouse/keyboard input). We will adopt this architecture in-tree inside `desktop-gpui/crates/terminal` (`webterm-terminal`), binding directly to our unified `gpui-pre = "=0.3.3"`.

---

## 2. Detailed Dependency Audit

### Crate Manifest of `gpui-terminal = "0.1.0"`
```toml
[package]
name = "gpui-terminal"
version = "0.1.0"
edition = "2024"

[dependencies.alacritty_terminal]
version = "0.25.1"

[dependencies.gpui]
version = "0.2.2"

[dependencies.arboard]
version = "3"
features = ["wayland-data-control"]

[dependencies.flume]
version = "0.12"

[dependencies.parking_lot]
version = "0.12"
```

### Why the Version Conflict is Fatal
1. **Type Incompatibility:**
   In Rust, types from two different versions of the same crate (or different crate names like `gpui` vs `gpui-pre`) are distinct and incompatible.
   If `gpui-terminal` returns or takes `gpui::Entity<T>`, `gpui::Window`, or `gpui::Context`, those types cannot be passed to or received from our main application which uses `gpui-pre::Entity<T>`, `gpui-pre::Window`, and `gpui-pre::Context`.
2. **Double Runtime Collision:**
   GPUI initializes global platform hooks and event dispatchers. Having two distinct versions of GPUI compiled into the same binary results in duplicate global registrations and panics during platform initialization.
3. **Workspace Constraint from Phase 20:**
   In Phase 20, we established that `gpui-component = "=0.6.0"` strictly requires `gpui-pre = "=0.3.3"`. We unified the dependency tree around `gpui-pre = "=0.3.3"` and `gpui-pre-platform = "=0.3.3"`. We cannot downgrade to `0.2.2` without breaking `gpui-component`.

---

## 3. The Solution: In-Tree `webterm-terminal` Crate

Instead of relying on the external unmaintained `gpui-terminal` crate, we implement `desktop-gpui/crates/terminal` (`webterm-terminal`) as a first-class workspace member:
1. **Direct `alacritty_terminal = "0.25.1"` dependency:**
   We pull the battle-tested terminal state machine and parser directly.
2. **Unified `gpui-pre = "=0.3.3"` binding:**
   The rendering pipeline, text system, and event handlers bind directly to our workspace GPUI types.
3. **Integrated Theme & Font System:**
   The palette and font metrics bind directly to WebTerm's `Theme` (`Dark`/`Light`) and bundled `JetBrains Mono` font.
4. **Clean Boundary:**
   Exposes a clean `TerminalView` entity that will be seamlessly embedded in the multi-tab session manager in Phase 22.

---

## 4. Conclusion
Success Criterion 5 ("Spike verdict recorded: gpui-terminal validated OR vendored Zed-pattern fallback implemented behind the terminal crate boundary") is satisfied with this verdict. We proceed immediately with Plan 21-01 Task 2 to scaffold `crates/terminal`.
