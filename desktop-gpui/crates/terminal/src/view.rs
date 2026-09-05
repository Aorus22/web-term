//! Interactive GPUI TerminalView component.
//!
//! Manages terminal canvas rendering, keyboard translation, mouse selection,
//! clipboard operations, scrollback navigation, and grid resize.

use crate::colors::ColorPalette;
use crate::event::TerminalEvent;
use crate::input::keystroke_to_bytes;
use crate::mouse::{
    modifiers_to_mouse_code, mouse_button_report, pixel_to_cell, scroll_report,
    selection_type_from_clicks,
};
use crate::render::TerminalRenderer;
use crate::terminal::Terminal;
use alacritty_terminal::index::{Column, Line, Point as AlacPoint};
use gpui::*;
use parking_lot::Mutex;
use std::sync::Arc;

pub type InputCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;
pub type ResizeCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;
pub type TitleCallback = Arc<dyn Fn(&str) + Send + Sync>;
pub type BellCallback = Arc<dyn Fn() + Send + Sync>;

/// Core GPUI view wrapping the terminal emulator.
pub struct TerminalView {
    terminal: Arc<Mutex<Terminal>>,
    renderer: TerminalRenderer,
    focus_handle: FocusHandle,
    padding: Edges<Pixels>,
    is_selecting: bool,
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
                    let cont = cx_handle
                        .update(|cx: &mut App| {
                            if let Some(entity) = view_weak.upgrade() {
                                entity.update(cx, |this, cx| {
                                    match &event {
                                        TerminalEvent::Title(title) => {
                                            if let Some(ref cb) = this.title_callback {
                                                cb(title);
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

        Self {
            terminal: term_arc,
            renderer,
            focus_handle,
            padding: Edges::all(px(4.0)),
            is_selecting: false,
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
    pub fn with_resize_callback<F: Fn(usize, usize) + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.resize_callback = Some(Arc::new(f));
        self
    }

    /// Set callback invoked when terminal title changes.
    pub fn with_title_callback<F: Fn(&str) + Send + Sync + 'static>(mut self, f: F) -> Self {
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
    pub fn set_title_callback<F: Fn(&str) + Send + Sync + 'static>(&mut self, f: F) {
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
    pub fn set_font_size(&mut self, size: Pixels, cx: &mut Context<Self>) {
        self.renderer.font_size = size;
        self.renderer.cell_width = size * 0.6;
        self.renderer.cell_height = size * self.renderer.line_height_multiplier;
        cx.notify();
    }

    /// Dynamically update font family and font size used by this terminal view.
    pub fn set_font(&mut self, family: String, size: Pixels, cx: &mut Context<Self>) {
        self.renderer.font_family = family;
        self.renderer.font_size = size;
        self.renderer.cell_width = size * 0.6;
        self.renderer.cell_height = size * self.renderer.line_height_multiplier;
        cx.notify();
    }

    /// Focus handle for keyboard input routing.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
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
        let bounds = *self.last_bounds.lock();

        if let Some(bounds) = bounds {
            let origin = Point {
                x: bounds.origin.x + self.padding.left,
                y: bounds.origin.y + self.padding.top,
            };
            let mut term = self.terminal.lock();
            let pt = pixel_to_cell(
                event.position,
                origin,
                self.renderer.cell_width,
                self.renderer.cell_height,
                term.cols(),
                term.rows(),
            );

            let mode = term.mode();
            let mouse_mods = modifiers_to_mouse_code(&event.modifiers);

            if let Some(bytes) = mouse_button_report(event.button, true, pt, mouse_mods, mode) {
                drop(term);
                self.write_to_pty(&bytes);
            } else if event.button == MouseButton::Left {
                let sel_type = selection_type_from_clicks(event.click_count);
                term.start_selection(pt, sel_type);
                self.is_selecting = true;
            }
        }

        cx.notify();
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = *self.last_bounds.lock();
        if let Some(bounds) = bounds {
            let origin = Point {
                x: bounds.origin.x + self.padding.left,
                y: bounds.origin.y + self.padding.top,
            };
            let mut term = self.terminal.lock();
            let pt = pixel_to_cell(
                event.position,
                origin,
                self.renderer.cell_width,
                self.renderer.cell_height,
                term.cols(),
                term.rows(),
            );

            if self.is_selecting {
                term.update_selection(pt);
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

    fn on_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let bounds = *self.last_bounds.lock();
        if let Some(bounds) = bounds {
            let origin = Point {
                x: bounds.origin.x + self.padding.left,
                y: bounds.origin.y + self.padding.top,
            };
            let term = self.terminal.lock();
            let pt = pixel_to_cell(
                event.position,
                origin,
                self.renderer.cell_width,
                self.renderer.cell_height,
                term.cols(),
                term.rows(),
            );

            let mode = term.mode();
            let mouse_mods = modifiers_to_mouse_code(&event.modifiers);
            if let Some(bytes) = mouse_button_report(event.button, false, pt, mouse_mods, mode) {
                drop(term);
                self.write_to_pty(&bytes);
            }
        }

        if event.button == MouseButton::Left {
            self.is_selecting = false;
        }

        cx.notify();
    }

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta_lines = match event.delta {
            ScrollDelta::Lines(delta) => delta.y.round() as i32,
            ScrollDelta::Pixels(delta) => {
                let dy: f32 = delta.y.into();
                let ch: f32 = self.renderer.cell_height.into();
                if ch > 0.0 {
                    (dy / ch).round() as i32
                } else {
                    0
                }
            }
        };

        if delta_lines == 0 {
            return;
        }

        let bounds = *self.last_bounds.lock();
        let pt = if let Some(bounds) = bounds {
            let origin = Point {
                x: bounds.origin.x + self.padding.left,
                y: bounds.origin.y + self.padding.top,
            };
            let term = self.terminal.lock();
            pixel_to_cell(
                event.position,
                origin,
                self.renderer.cell_width,
                self.renderer.cell_height,
                term.cols(),
                term.rows(),
            )
        } else {
            AlacPoint::new(Line(0), Column(0))
        };

        let mode = self.terminal.lock().mode();
        let mouse_mods = modifiers_to_mouse_code(&event.modifiers);

        if let Some(bytes) = scroll_report(delta_lines, pt, mouse_mods, mode) {
            self.write_to_pty(&bytes);
        } else {
            let mut term = self.terminal.lock();
            term.scroll_display(delta_lines);
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

        div()
            .size_full()
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
                .size_full(),
            )
    }
}
