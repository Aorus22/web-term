//! Interactive GPUI TerminalView component.
//!
//! Manages terminal canvas rendering, keyboard translation, mouse selection,
//! clipboard operations, scrollback navigation, and grid resize.

use crate::colors::ColorPalette;
use crate::event::TerminalEvent;
use crate::input::keystroke_to_bytes;
use crate::mouse::{
    modifiers_to_mouse_code, mouse_button_report, pixel_to_cell_with_side, scroll_report,
    selection_type_from_clicks,
};
use crate::render::TerminalRenderer;
use crate::terminal::Terminal;
use alacritty_terminal::index::{Column, Line, Point as AlacPoint, Side};
use alacritty_terminal::vte::ansi::CursorShape;
use gpui::*;
use parking_lot::Mutex;
use std::sync::Arc;

pub type InputCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;
pub type ResizeCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;
pub type TitleCallback = Arc<dyn Fn(&str, &mut App) + Send + Sync>;
pub type BellCallback = Arc<dyn Fn() + Send + Sync>;

/// In-progress scrollbar thumb drag: pointer Y (px) where the grab started
/// plus the display offset at that moment.
struct ScrollbarDrag {
    start_y: f32,
    start_offset: usize,
}
/// Core GPUI view wrapping the terminal emulator.
pub struct TerminalView {
    terminal: Arc<Mutex<Terminal>>,
    renderer: TerminalRenderer,
    /// Whether font metrics in `renderer` have been measured from the text
    /// system. Font measurement requires a `&mut Window`, which is only
    /// available in `Render::render` — so measurement is deferred to the next
    /// frame after construction or any `set_*` mutation.
    renderer_needs_measure: bool,
    focus_handle: FocusHandle,
    padding: Edges<Pixels>,
    is_selecting: bool,
    scrollbar_drag: Option<ScrollbarDrag>,
    /// Fractional pixel remainder from smooth-scroll (touchpad) deltas that
    /// were too small to form a whole line. Without this, sub-cell-height
    /// deltas round to zero and scrolling feels dead.
    scroll_remainder_px: f32,
    last_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    input_callback: Option<InputCallback>,
    resize_callback: Option<ResizeCallback>,
    title_callback: Option<TitleCallback>,
    bell_callback: Option<BellCallback>,
}

impl TerminalView {
    /// Construct a new `TerminalView` wrapping an existing `Terminal` instance.
    pub fn new(terminal: Terminal, cx: &mut Context<Self>) -> Self {
        let event_rx = terminal.event_channel();
        let term_arc = Arc::new(Mutex::new(terminal));
        let focus_handle = cx.focus_handle();
        let renderer = TerminalRenderer::new(
            "JetBrains Mono".to_string(),
            px(14.0),
            1.2,
            ColorPalette::dark_default(),
        );

        let view_weak = cx.entity().downgrade();

        // Reactive event subscription waking GPUI on terminal events
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                while let Ok(event) = event_rx.recv_async().await {
                    let view_weak = view_weak.clone();
                    let cont = cx_handle.update(|cx: &mut App| {
                        if let Some(entity) = view_weak.upgrade() {
                            entity.update(cx, |this, cx| {
                                match &event {
                                    TerminalEvent::Title(title) => {
                                        if let Some(ref cb) = this.title_callback {
                                            let title = title.clone();
                                            cb(&title, cx);
                                        }
                                    }
                                    TerminalEvent::Bell => {
                                        if let Some(ref cb) = this.bell_callback {
                                            cb();
                                        }
                                    }
                                    _ => {}
                                }
                                cx.notify();
                            });
                            true
                        } else {
                            false
                        }
                    });
                    if !cont {
                        break;
                    }
                }
            }
        })
        .detach();

        // Session context for field diagnostics (logged once per view).
        crate::debug_log::debug_log(
            "session",
            format!(
                "TerminalView created wayland={:?} xdg_session={:?} display={:?}",
                std::env::var("WAYLAND_DISPLAY").ok(),
                std::env::var("XDG_SESSION_TYPE").ok(),
                std::env::var("DISPLAY").ok(),
            ),
        );

        Self {
            terminal: term_arc,
            renderer,
            renderer_needs_measure: true,
            focus_handle,
            padding: Edges::all(px(4.0)),
            is_selecting: false,
            scrollbar_drag: None,
            scroll_remainder_px: 0.0,
            last_bounds: Arc::new(Mutex::new(None)),
            input_callback: None,
            resize_callback: None,
            title_callback: None,
            bell_callback: None,
        }
    }

    /// Attach custom terminal renderer.
    pub fn with_renderer(mut self, renderer: TerminalRenderer) -> Self {
        self.renderer = renderer;
        self.renderer_needs_measure = true;
        self
    }

    /// Set inner canvas padding.
    pub fn with_padding(mut self, padding: Edges<Pixels>) -> Self {
        self.padding = padding;
        self
    }

    /// Set callback invoked when terminal generates input bytes.
    pub fn with_input_callback<F: Fn(&[u8]) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.input_callback = Some(Arc::new(f));
        self
    }

    /// Set callback invoked when terminal grid dimensions change.
    pub fn with_resize_callback<F: Fn(usize, usize) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.resize_callback = Some(Arc::new(f));
        self
    }

    /// Set callback invoked when terminal title changes.
    pub fn with_title_callback<F: Fn(&str, &mut App) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.title_callback = Some(Arc::new(f));
        self
    }

    /// Set callback invoked when terminal bell triggers.
    pub fn with_bell_callback<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.bell_callback = Some(Arc::new(f));
        self
    }

    /// Update callback invoked when terminal generates input bytes.
    pub fn set_input_callback<F: Fn(&[u8]) + Send + Sync + 'static>(&mut self, f: F) {
        self.input_callback = Some(Arc::new(f));
    }

    /// Update callback invoked when terminal grid dimensions change.
    pub fn set_resize_callback<F: Fn(usize, usize) + Send + Sync + 'static>(&mut self, f: F) {
        self.resize_callback = Some(Arc::new(f));
    }

    /// Update callback invoked when terminal title changes.
    pub fn set_title_callback<F: Fn(&str, &mut App) + Send + Sync + 'static>(&mut self, f: F) {
        self.title_callback = Some(Arc::new(f));
    }

    /// Update callback invoked when terminal bell triggers.
    pub fn set_bell_callback<F: Fn() + Send + Sync + 'static>(&mut self, f: F) {
        self.bell_callback = Some(Arc::new(f));
    }

    /// Access the underlying `Terminal` mutex.
    pub fn terminal(&self) -> Arc<Mutex<Terminal>> {
        Arc::clone(&self.terminal)
    }

    /// Access the renderer.
    pub fn renderer(&self) -> &TerminalRenderer {
        &self.renderer
    }

    /// Mutable access to the renderer.
    pub fn renderer_mut(&mut self) -> &mut TerminalRenderer {
        &mut self.renderer
    }

    /// Dynamically update the color palette used by this terminal view.
    pub fn set_palette(&mut self, palette: ColorPalette, cx: &mut Context<Self>) {
        self.renderer.palette = palette;
        cx.notify();
    }

    /// Return reference to the current color palette.
    pub fn palette(&self) -> &ColorPalette {
        &self.renderer.palette
    }

    /// Dynamically update the font size used by this terminal view.
    ///
    /// Cell dimensions are set to a rough estimate and re-measured from the
    /// text system on the next render pass, so mouse hit-testing and painting
    /// always share the same real metrics.
    pub fn set_font_size(&mut self, size: Pixels, cx: &mut Context<Self>) {
        self.renderer.font_size = size;
        self.renderer.cell_width = size * 0.6;
        self.renderer.cell_height = size * self.renderer.line_height_multiplier;
        self.renderer_needs_measure = true;
        cx.notify();
    }

    /// Dynamically update font family and font size used by this terminal view.
    ///
    /// Cell dimensions are set to a rough estimate and re-measured from the
    /// text system on the next render pass, so mouse hit-testing and painting
    /// always share the same real metrics.
    pub fn set_font(&mut self, family: String, size: Pixels, cx: &mut Context<Self>) {
        self.renderer.font_family = family;
        self.renderer.font_size = size;
        self.renderer.cell_width = size * 0.6;
        self.renderer.cell_height = size * self.renderer.line_height_multiplier;
        self.renderer_needs_measure = true;
        cx.notify();
    }

    /// Dynamically update the cursor shape override for this terminal view.
    pub fn set_cursor_shape(&mut self, shape: Option<CursorShape>, cx: &mut Context<Self>) {
        self.renderer.cursor_shape_override = shape;
        cx.notify();
    }

    /// Focus handle for keyboard input routing.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Convert a window pixel position into a terminal grid point.
    ///
    /// Lazily refreshes `self.renderer`'s font metrics,
    /// then maps pixels to a screen row, then to a buffer `Line` by
    /// subtracting the scrollback display offset — the exact inverse of how
    /// the renderer positions grid lines on screen.
    fn pixel_to_term_point(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
    ) -> (AlacPoint, Side) {
        if self.renderer_needs_measure {
            self.renderer.measure_cell(window);
            self.renderer_needs_measure = false;
        }
        self.pixel_to_term_point_measured(position)
    }

    /// Like [`Self::pixel_to_term_point`], but uses the renderer's current
    /// metrics without re-measuring. Safe once at least one paint pass has
    /// synced the measured metrics (the paint pass syncs them every frame).
    fn pixel_to_term_point_measured(&self, position: Point<Pixels>) -> (AlacPoint, Side) {

        let bounds = *self.last_bounds.lock();
        let Some(bounds) = bounds else {
            return (AlacPoint::new(Line(0), Column(0)), Side::Left);
        };

        let origin = Point {
            x: bounds.origin.x + self.padding.left,
            y: bounds.origin.y + self.padding.top,
        };

        let term = self.terminal.lock();
        let (screen, side) = pixel_to_cell_with_side(
            position,
            origin,
            self.renderer.cell_width,
            self.renderer.cell_height,
            term.cols(),
            term.rows(),
        );
        let display_offset = term.display_offset();
        (
            AlacPoint::new(Line(screen.line.0 - display_offset), screen.column),
            side,
        )
    }

    /// Send input bytes to the backend or input callback.
    pub fn write_to_pty(&self, bytes: &[u8]) {
        if let Some(ref cb) = self.input_callback {
            cb(bytes);
        }
    }

    /// Process output bytes received from PTY / SSH stream into terminal grid.
    pub fn process_output(&self, bytes: &[u8], cx: &mut Context<Self>) {
        self.terminal.lock().process_bytes(bytes);
        cx.notify();
    }

    /// Copy current selection text to system clipboard.
    pub fn copy_selection(&self, cx: &mut Context<Self>) -> bool {
        if let Some(text) = self.terminal.lock().selection_text() {
            if !text.is_empty() {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                return true;
            }
        }
        false
    }

    /// Paste text from system clipboard into terminal input.
    pub fn paste_clipboard(&self, cx: &mut Context<Self>) -> bool {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if !text.is_empty() {
                self.write_to_pty(text.as_bytes());
                return true;
            }
        }
        false
    }

    /// Scroll display viewport by delta lines.
    pub fn scroll_display(&self, delta: i32, cx: &mut Context<Self>) {
        self.terminal.lock().scroll_display(delta);
        cx.notify();
    }

    /// Scroll viewport to bottom (active cursor position).
    pub fn scroll_to_bottom(&self, cx: &mut Context<Self>) {
        self.terminal.lock().scroll_to_bottom();
        cx.notify();
    }

    /// Scroll viewport to top of scrollback history.
    pub fn scroll_to_top(&self, cx: &mut Context<Self>) {
        self.terminal.lock().scroll_to_top();
        cx.notify();
    }

    /// (history_lines, display_offset, visible_lines) for the scrollbar.
    /// display_offset 0 = bottom, history_lines = max offset.
    fn scroll_metrics(&self) -> (usize, usize, usize) {
        self.terminal.lock().with_term(|term| {
            use alacritty_terminal::grid::Dimensions;
            let grid = term.grid();
            (
                grid.history_size(),
                grid.display_offset(),
                grid.screen_lines(),
            )
        })
    }

    /// Begin dragging the scrollbar thumb.
    fn on_scrollbar_thumb_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (_, offset, _) = self.scroll_metrics();
        crate::debug_log::debug_log(
            "scroll",
            format!("thumb grab y={} offset={offset}", f32::from(event.position.y)),
        );
        self.scrollbar_drag = Some(ScrollbarDrag {
            start_y: event.position.y.into(),
            start_offset: offset,
        });
        // Thumb is a sibling of the canvas, but stop here so the canvas
        // selection handler further up the tree never sees this press.
        cx.stop_propagation();
        cx.notify();
    }

    /// Continue an in-progress thumb drag: map pointer travel to lines.
    /// The track spans the same height as the canvas (row layout, both full
    /// height), so canvas bounds give the track geometry.
    fn on_scrollbar_drag_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(drag) = &self.scrollbar_drag else {
            return;
        };
        let (history, _, _) = self.scroll_metrics();
        if history == 0 {
            return;
        }
        let track_h: f32 = self
            .last_bounds
            .lock()
            .map(|b| b.size.height.into())
            .unwrap_or(0.0);
        if track_h <= 0.0 {
            return;
        }
        let dy: f32 = f32::from(event.position.y) - drag.start_y;
        let lines_per_px = history as f32 / track_h;
        let target = (drag.start_offset as f32 - dy * lines_per_px)
            .round()
            .clamp(0.0, history as f32) as usize;
        let (_, current, _) = self.scroll_metrics();
        let delta = target as i32 - current as i32;
        if delta != 0 {
            self.terminal.lock().scroll_display(delta);
            crate::debug_log::debug_log(
                "scroll",
                format!("thumb drag dy={dy:.1}px target={target} current={current} delta={delta} history={history}"),
            );
            cx.notify();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        // Clipboard shortcuts:
        // Ctrl+Shift+C or Cmd+C -> Copy
        let is_copy = (event.keystroke.modifiers.control
            && event.keystroke.modifiers.shift
            && event.keystroke.key.eq_ignore_ascii_case("c"))
            || (event.keystroke.modifiers.platform
                && event.keystroke.key.eq_ignore_ascii_case("c"));

        if is_copy && self.copy_selection(cx) {
            return;
        }

        // Ctrl+Shift+V or Cmd+V -> Paste
        let is_paste = (event.keystroke.modifiers.control
            && event.keystroke.modifiers.shift
            && event.keystroke.key.eq_ignore_ascii_case("v"))
            || (event.keystroke.modifiers.platform
                && event.keystroke.key.eq_ignore_ascii_case("v"));

        if is_paste && self.paste_clipboard(cx) {
            return;
        }

        // Any key press resets scrollback offset to zero (bottom of terminal)
        {
            let mut term = self.terminal.lock();
            term.scroll_to_bottom();
        }

        let mode = self.terminal.lock().mode();
        if let Some(bytes) = keystroke_to_bytes(&event.keystroke, mode) {
            self.write_to_pty(&bytes);
        }

        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle, cx);

        if self.last_bounds.lock().is_some() {
            let (pt, side) = self.pixel_to_term_point(event.position, window);
            crate::debug_log::debug_log(
                "select",
                format!(
                    "mouse_down pos=({:.0},{:.0}) bounds={:?} cell=({},{}) side={side:?} btn={:?} clicks={}",
                    f32::from(event.position.x),
                    f32::from(event.position.y),
                    *self.last_bounds.lock(),
                    pt.line.0,
                    pt.column.0,
                    event.button,
                    event.click_count,
                ),
            );

            let mut term = self.terminal.lock();
            let mode = term.mode();
            let mouse_mods = modifiers_to_mouse_code(&event.modifiers);

            if let Some(bytes) = mouse_button_report(event.button, true, pt, mouse_mods, mode) {
                drop(term);
                self.write_to_pty(&bytes);
            } else if event.button == MouseButton::Left {
                let sel_type = selection_type_from_clicks(event.click_count);
                term.start_selection(pt, side, sel_type);
                self.is_selecting = true;
            }
        }

        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.scrollbar_drag.is_some() {
            self.on_scrollbar_drag_move(event, cx);
            return;
        }
        if self.last_bounds.lock().is_some() {
            let (pt, side) = self.pixel_to_term_point(event.position, window);

            let mut term = self.terminal.lock();
            if self.is_selecting {
                term.update_selection(pt, side);
                cx.notify();
            } else if let Some(button) = event.pressed_button {
                let mode = term.mode();
                let mouse_mods = modifiers_to_mouse_code(&event.modifiers);
                if let Some(bytes) = mouse_button_report(button, true, pt, mouse_mods, mode) {
                    drop(term);
                    self.write_to_pty(&bytes);
                }
            }
        }
    }

    fn on_mouse_up(&mut self, event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.last_bounds.lock().is_some() {
            let (pt, _) = self.pixel_to_term_point(event.position, window);

            let term = self.terminal.lock();
            let mode = term.mode();
            let mouse_mods = modifiers_to_mouse_code(&event.modifiers);
            if let Some(bytes) = mouse_button_report(event.button, false, pt, mouse_mods, mode) {
                drop(term);
                self.write_to_pty(&bytes);
            }
        }

        if event.button == MouseButton::Left {
            self.is_selecting = false;
            self.scrollbar_drag = None;
        }

        cx.notify();
    }

    /// Window-level drag continuation for text selection and scrollbar drags.
    ///
    /// Registered from the canvas paint pass via `window.on_mouse_event` while
    /// a drag is active, so pointer movement outside the terminal bounds (e.g.
    /// over the sidebar) keeps updating the drag. X11 and Wayland both deliver
    /// pointer events to the window during an implicit button-press grab —
    /// including the release — so the drag always terminates on mouse up.
    fn on_drag_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.scrollbar_drag.is_some() {
            self.on_scrollbar_drag_move(event, cx);
            return;
        }
        if self.is_selecting {
            let (pt, side) = self.pixel_to_term_point_measured(event.position);
            self.terminal.lock().update_selection(pt, side);
            cx.notify();
        }
    }

    /// Window-level counterpart of [`Self::on_drag_mouse_move`]: ends any
    /// active drag no matter where the pointer was released.
    fn on_drag_mouse_up(&mut self, event: &MouseUpEvent, cx: &mut Context<Self>) {
        if event.button == MouseButton::Left
            && (self.is_selecting || self.scrollbar_drag.is_some())
        {
            self.is_selecting = false;
            self.scrollbar_drag = None;
            cx.notify();
        }
    }

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (history, offset, _) = self.scroll_metrics();
        crate::debug_log::debug_log(
            "scroll",
            format!("wheel delta={:?} history={history} offset={offset}", event.delta),
        );
        let delta_lines = match event.delta {
            ScrollDelta::Lines(delta) => delta.y.round() as i32,
            ScrollDelta::Pixels(delta) => {
                let dy: f32 = delta.y.into();
                let ch: f32 = self.renderer.cell_height.into();
                if ch > 0.0 {
                    // Accumulate sub-line remainders so smooth-scroll deltas
                    // smaller than one cell still move after a few ticks.
                    self.scroll_remainder_px += dy;
                    let lines = (self.scroll_remainder_px / ch).trunc();
                    self.scroll_remainder_px -= lines * ch;
                    lines as i32
                } else {
                    0
                }
            }
        };

        if delta_lines == 0 {
            return;
        }

        // NOTE: GPUI reports wheel-up as positive Y on both X11 and Wayland,
        // matching scroll_report/scroll_display (positive = up / history).

        let (pt, _) = self.pixel_to_term_point(event.position, window);

        let mode = self.terminal.lock().mode();
        let mouse_mods = modifiers_to_mouse_code(&event.modifiers);

        if let Some(bytes) = scroll_report(delta_lines, pt, mouse_mods, mode) {
            crate::debug_log::debug_log(
                "scroll",
                format!("forwarded to app as escape sequence bytes={}", bytes.len()),
            );
            self.write_to_pty(&bytes);
        } else {
            let mut term = self.terminal.lock();
            term.scroll_display(delta_lines);
            let new_offset = term.display_offset();
            crate::debug_log::debug_log(
                "scroll",
                format!("applied delta={delta_lines} offset {offset}->{new_offset}"),
            );
        }

        cx.notify();
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        let is_focused = focus_handle.is_focused(window);
        let term_arc = Arc::clone(&self.terminal);
        let renderer = self.renderer.clone();
        let last_bounds = Arc::clone(&self.last_bounds);
        let padding = self.padding;
        let resize_cb = self.resize_callback.clone();
        let view_handle = cx.entity().downgrade();
        let drag_active = self.is_selecting || self.scrollbar_drag.is_some();

        div()
            .size_full()
            .flex()
            .flex_row()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Right, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Right, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Middle, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(
                canvas(
                    move |bounds, _window, _cx| {
                        *last_bounds.lock() = Some(bounds);
                        bounds
                    },
                    move |bounds, _, window, cx| {
                        let mut measured = renderer.clone();
                        measured.measure_cell(window);

                        // Keep the view's renderer in sync with the measured
                        // cell metrics so mouse hit-testing uses exactly the
                        // same geometry as this paint pass.
                        let _ = view_handle.update(cx, |this, cx| {
                            if this.renderer_needs_measure
                                || this.renderer.cell_width != measured.cell_width
                                || this.renderer.cell_height != measured.cell_height
                            {
                                this.renderer.cell_width = measured.cell_width;
                                this.renderer.cell_height = measured.cell_height;
                                this.renderer_needs_measure = false;
                                cx.notify();
                            }
                        });

                        // While a text-selection or scrollbar drag is active,
                        // listen for mouse move/up at the window level so the
                        // drag continues and terminates even when the pointer
                        // leaves the terminal bounds (e.g. over the sidebar).
                        if drag_active {
                            let drag_view = view_handle.clone();
                            window.on_mouse_event(
                                move |event: &MouseMoveEvent, _phase, _window, cx| {
                                    let _ = drag_view.update(cx, |this, cx| {
                                        this.on_drag_mouse_move(event, cx);
                                    });
                                },
                            );
                            let drag_view = view_handle.clone();
                            window.on_mouse_event(
                                move |event: &MouseUpEvent, _phase, _window, cx| {
                                    let _ = drag_view.update(cx, |this, cx| {
                                        this.on_drag_mouse_up(event, cx);
                                    });
                                },
                            );
                        }

                        let avail_w: f32 =
                            (bounds.size.width - padding.left - padding.right).into();
                        let avail_h: f32 =
                            (bounds.size.height - padding.top - padding.bottom).into();
                        let cw: f32 = measured.cell_width.into();
                        let ch: f32 = measured.cell_height.into();

                        let cols = ((avail_w / cw).floor() as usize).max(1);
                        let rows = ((avail_h / ch).floor() as usize).max(1);

                        let mut term = term_arc.lock();
                        if cols != term.cols() || rows != term.rows() {
                            term.resize(cols, rows);
                            if let Some(ref cb) = resize_cb {
                                cb(cols, rows);
                            }
                        }

                        let raw_term_arc = term.term_arc();
                        let raw_term = raw_term_arc.lock();
                        measured.paint(bounds, padding, &raw_term, is_focused, window, cx);
                    },
                )
                .flex_1()
                .min_w_0()
                .h_full(),
            )
            .child(self.render_scrollbar(cx))
    }
}

impl TerminalView {
    /// Visible scrollbar track + draggable thumb. Track width stays constant
    /// (no layout shift); thumb appears once scrollback exists. Thumb position
    /// uses flex spacers so no pixel geometry is needed at render time.
    fn render_scrollbar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let (history, offset, visible) = self.scroll_metrics();
        let total = (history + visible).max(1);
        let top_frac = (history - offset) as f32 / total as f32;
        let thumb_frac = visible as f32 / total as f32;
        let bottom_frac = offset as f32 / total as f32;

        div()
            .w(px(10.0))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py_1()
            .child(div().flex_grow(top_frac.max(0.0)).min_h_0())
            .child(
                div()
                    .w(px(6.0))
                    .flex_grow(thumb_frac.max(0.0))
                    .min_h(if history > 0 { px(20.0) } else { px(0.0) })
                    .rounded_full()
                    .bg(rgb(0x71717a))
                    .opacity(if history > 0 { 0.55 } else { 0.0 })
                    .id("view-01").hover(|s| s.opacity(0.9))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_scrollbar_thumb_down)),
            )
            .child(div().flex_grow(bottom_frac.max(0.0)).min_h_0())
    }
}
