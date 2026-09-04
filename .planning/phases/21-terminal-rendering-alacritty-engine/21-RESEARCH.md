# Phase 21: Terminal Rendering (Alacritty Engine) - Research

**Date:** 2026-09-04
**Phase:** 21-terminal-rendering-alacritty-engine
**Objective:** Evaluate terminal emulation architecture for GPUI, conduct spike on `gpui-terminal`, and define integration plan for `alacritty_terminal`.

---

## 1. Executive Summary & Spike Verdict

### The Question
Can `gpui-terminal = "0.1.0"` be pulled directly from crates.io into `desktop-gpui`?

### The Finding
- `gpui-terminal = "0.1.0"` depends hardcoded on `gpui = "0.2.2"`.
- Our desktop client workspace is pinned to `gpui-pre = "=0.3.3"` (and `gpui-pre-platform = "=0.3.3"`) to guarantee binary compatibility with `gpui-component = "=0.6.0"`.
- Cargo cannot unify `gpui = "0.2.2"` and `gpui-pre = "0.3.3"` — attempting to use `gpui-terminal` directly from crates.io results in duplicated GPUI runtime crates, symbol collisions, and incompatible `Entity`, `Window`, `App`, and `Context` types.

### The Verdict: NO-GO for external crate; GO for in-tree vendored crate
- **Direct dependency verdict:** **NO-GO**. Do not add `gpui-terminal = "0.1.0"` to `desktop-gpui/Cargo.toml`.
- **In-tree crate verdict:** **GO**. The architecture of `gpui-terminal` (10 clean files: `colors.rs`, `event.rs`, `input.rs`, `mouse.rs`, `render.rs`, `terminal.rs`, `view.rs`, `clipboard.rs`) provides the exact production-grade `alacritty_terminal` (v0.25.1) integration needed.
- By vendoring this architecture directly into `desktop-gpui/crates/terminal` (`webterm-terminal`), we bind it to our workspace `gpui-pre 0.3.3`, our bundled `JetBrains Mono` font, and our application theme system without external version drift.

---

## 2. Technical Analysis of `alacritty_terminal` (0.25.1)

### Key Structures & Traits
1. **`Term<EventProxy>`**:
   - Manages grid state, active screen vs alternate screen (for vim/htop/tmux), scrollback history, selection ranges, and cursor positions.
   - `Term::new(config, &dimensions, event_proxy)` initializes the terminal grid.
   - `Term::resize(dimensions)` resizes cols/rows.
2. **`Processor` / `vte` parser**:
   - `Processor::process_input(&mut term, bytes)` feeds raw UTF-8 / ANSI escape sequences from the PTY/SSH channel into the terminal state machine.
3. **`Grid` & `Cell`**:
   - Cells carry character (`c: char`), flags (`Flags::BOLD`, `ITALIC`, `UNDERLINE`, `INVERSE`, `DIM`), foreground and background colors (`Color::Spec(Rgb)` or indexed ANSI 0..255).
4. **`Selection`**:
   - Supports simple range selection and block selection.
   - Text can be extracted directly across lines via `term.selection_to_string()`.

---

## 3. GPUI Rendering Pipeline

### Cell Metrics & Glyph Batching
- Cell dimensions are derived from `cx.text_system()` font metrics using the standard character `'M'` (widest monospace glyph) and `(ascent + descent) * line_height_multiplier` (default `1.2`).
- To achieve 60+ FPS under heavy streaming output, the renderer:
  1. **Merges Background Quads:** Adjacent cells on the same row with identical background colors are coalesced into a single quad. Cells with default background are omitted entirely.
  2. **Batches Text Runs:** Consecutive cells with identical text styles (color, bold, italic) are grouped into `BatchedTextRun` structs for collective text shaping and painting.
  3. **Renders Cursor:** Paints block, beam, or underline at the cursor coordinates, adapting color and opacity to focus state.

---

## 4. Requirement Verification Mapping

| Requirement | Description | Implementation Strategy |
|-------------|-------------|-------------------------|
| **TERM-01** | vim, htop, tmux, curses, truecolor | `alacritty_terminal` alternate screen + 24-bit RGB TrueColor ANSI parsing (`\x1b[38;2;r;g;bm`). |
| **TERM-02** | Mouse selection + copy/paste | Mouse drag listener updates `term.selection`, right-click or shortcut reads/writes GPUI clipboard (`cx.write_to_clipboard`). |
| **TERM-03** | Scrollback history | Mouse wheel increments/decrements scrollback display offset (`term.scroll_display(Scroll::Delta(n))`). Keyboard input resets offset to 0. |
| **TERM-04** | Resize sync | Element layout bounds division by cell width/height computes `(cols, rows)`. Calls `term.resize` and triggers PTY resize callback. |

---

## 5. Phase Plan Structure

1. **Plan 21-01: gpui-terminal Spike & Crate Scaffold**
   - Create `desktop-gpui/crates/terminal` (`webterm-terminal`).
   - Add `alacritty_terminal = "0.25.1"` to workspace dependencies.
   - Record formal spike report `SPIKE-gpui-terminal.md` capturing the go/no-go verdict and dependency compatibility analysis.
2. **Plan 21-02: Terminal State & Event Loop**
   - Port terminal state machine, event proxy, I/O thread, and parser into `crates/terminal`.
   - Provide unit tests verifying ANSI escape sequence processing, alternate screen switching, and scrollback accumulation.
3. **Plan 21-03: GPUI Renderer, Selection & Input Mapping**
   - Implement `TerminalRenderer` and `TerminalView` with batched glyph/quad drawing.
   - Implement mouse selection, clipboard copy/paste, and keystroke-to-escape-sequence mapping.
   - Add mock PTY stream test fixture and verify TERM-01 through TERM-04.
