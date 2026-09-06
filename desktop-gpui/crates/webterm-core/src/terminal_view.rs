use gpui::*;
use gpui::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const DEFAULT_COLS: usize = 80;
const DEFAULT_ROWS: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub bold: bool,
    pub underline: bool,
    pub inverse: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: rgb(0xc9d1d9), // GitHub Dark text
            bg: rgb(0x0d1117), // GitHub Dark canvas
            bold: false,
            underline: false,
            inverse: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TerminalCell {
    pub ch: char,
    pub style: CellStyle,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: CellStyle::default(),
        }
    }
}

pub struct TerminalScreen {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Vec<TerminalCell>>,
    pub scrollback: Vec<Vec<TerminalCell>>,
    pub cursor_col: usize,
    pub cursor_row: usize,
    pub cursor_visible: bool,
    pub current_style: CellStyle,
    pub default_fg: Rgba,
    pub default_bg: Rgba,
    pub app_cursor: bool,
}

impl TerminalScreen {
    pub fn new(cols: usize, rows: usize) -> Self {
        let default_fg = rgb(0xc9d1d9);
        let default_bg = rgb(0x0d1117);
        let empty_row = vec![TerminalCell::default(); cols];
        Self {
            cols,
            rows,
            cells: vec![empty_row; rows],
            scrollback: Vec::new(),
            cursor_col: 0,
            cursor_row: 0,
            cursor_visible: true,
            current_style: CellStyle {
                fg: default_fg,
                bg: default_bg,
                bold: false,
                underline: false,
                inverse: false,
            },
            default_fg,
            default_bg,
            app_cursor: false,
        }
    }

    pub fn newline(&mut self) {
        if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        } else {
            if !self.cells.is_empty() {
                let old_row = self.cells.remove(0);
                if self.scrollback.len() > 1000 {
                    self.scrollback.remove(0);
                }
                self.scrollback.push(old_row);
            }
            self.cells.push(vec![TerminalCell::default(); self.cols]);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.cells {
            for cell in row.iter_mut() {
                *cell = TerminalCell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn clear_line_from_cursor(&mut self) {
        if self.cursor_row < self.rows {
            for c in self.cursor_col..self.cols {
                if c < self.cells[self.cursor_row].len() {
                    self.cells[self.cursor_row][c] = TerminalCell::default();
                }
            }
        }
    }

    pub fn clear_line_to_cursor(&mut self) {
        if self.cursor_row < self.rows {
            for c in 0..=self.cursor_col.min(self.cols.saturating_sub(1)) {
                if c < self.cells[self.cursor_row].len() {
                    self.cells[self.cursor_row][c] = TerminalCell::default();
                }
            }
        }
    }

    pub fn clear_entire_line(&mut self) {
        if self.cursor_row < self.rows {
            for cell in self.cells[self.cursor_row].iter_mut() {
                *cell = TerminalCell::default();
            }
        }
    }

    fn standard_color(code: u16) -> Rgba {
        match code {
            0 => rgb(0x161b22), // Black
            1 => rgb(0xf85149), // Red
            2 => rgb(0x7ee787), // Green
            3 => rgb(0xd29922), // Yellow
            4 => rgb(0x58a6ff), // Blue
            5 => rgb(0xbc8cff), // Magenta
            6 => rgb(0x39c5cf), // Cyan
            7 => rgb(0xd1d5da), // White
            8 => rgb(0x484f58), // Bright Black
            9 => rgb(0xff7b72), // Bright Red
            10 => rgb(0x56d364), // Bright Green
            11 => rgb(0xe3b341), // Bright Yellow
            12 => rgb(0x79c0ff), // Bright Blue
            13 => rgb(0xd2a8ff), // Bright Magenta
            14 => rgb(0x56d4dd), // Bright Cyan
            15 => rgb(0xf0f6fc), // Bright White
            _ => rgb(0xc9d1d9),
        }
    }

    fn color_256(idx: u16) -> Rgba {
        if idx < 16 {
            Self::standard_color(idx)
        } else if idx < 232 {
            let i = idx - 16;
            let r = ((i / 36) % 6) * 51;
            let g = ((i / 6) % 6) * 51;
            let b = (i % 6) * 51;
            rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff)
        } else {
            let gray = 8 + (idx - 232) * 10;
            let g = gray as u8;
            rgba(((g as u32) << 24) | ((g as u32) << 16) | ((g as u32) << 8) | 0xff)
        }
    }
}

impl vte::Perform for TerminalScreen {
    fn print(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.newline();
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col] = TerminalCell {
                ch: c,
                style: self.current_style,
            };
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\r' => {
                self.cursor_col = 0;
            }
            b'\n' => {
                self.newline();
            }
            b'\x08' => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            b'\t' => {
                let next_tab = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next_tab.min(self.cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let p: Vec<u16> = params.iter().map(|p| p.get(0).copied().unwrap_or(0)).collect();

        match action {
            'H' | 'f' => {
                let row = p.first().copied().unwrap_or(1).max(1) - 1;
                let col = p.get(1).copied().unwrap_or(1).max(1) - 1;
                self.cursor_row = (row as usize).min(self.rows.saturating_sub(1));
                self.cursor_col = (col as usize).min(self.cols.saturating_sub(1));
            }
            'A' => {
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
            }
            'C' => {
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
            }
            'D' => {
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'J' => {
                let mode = p.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        self.clear_line_from_cursor();
                        for r in (self.cursor_row + 1)..self.rows {
                            for cell in self.cells[r].iter_mut() {
                                *cell = TerminalCell::default();
                            }
                        }
                    }
                    1 => {
                        for r in 0..self.cursor_row {
                            for cell in self.cells[r].iter_mut() {
                                *cell = TerminalCell::default();
                            }
                        }
                        self.clear_line_to_cursor();
                    }
                    2 | 3 => {
                        self.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = p.first().copied().unwrap_or(0);
                match mode {
                    0 => self.clear_line_from_cursor(),
                    1 => self.clear_line_to_cursor(),
                    2 => self.clear_entire_line(),
                    _ => {}
                }
            }
            'h' => {
                if p.contains(&25) {
                    self.cursor_visible = true;
                }
                if p.contains(&1) {
                    self.app_cursor = true;
                }
            }
            'l' => {
                if p.contains(&25) {
                    self.cursor_visible = false;
                }
                if p.contains(&1) {
                    self.app_cursor = false;
                }
            }
            'm' => {
                if p.is_empty() {
                    self.current_style = CellStyle::default();
                    return;
                }
                let mut idx = 0;
                while idx < p.len() {
                    match p[idx] {
                        0 => {
                            self.current_style = CellStyle::default();
                        }
                        1 => self.current_style.bold = true,
                        4 => self.current_style.underline = true,
                        7 => self.current_style.inverse = true,
                        22 => self.current_style.bold = false,
                        24 => self.current_style.underline = false,
                        27 => self.current_style.inverse = false,
                        30..=37 => {
                            self.current_style.fg = Self::standard_color(p[idx] - 30);
                        }
                        39 => self.current_style.fg = self.default_fg,
                        40..=47 => {
                            self.current_style.bg = Self::standard_color(p[idx] - 40);
                        }
                        49 => self.current_style.bg = self.default_bg,
                        90..=97 => {
                            self.current_style.fg = Self::standard_color(p[idx] - 90 + 8);
                        }
                        100..=107 => {
                            self.current_style.bg = Self::standard_color(p[idx] - 100 + 8);
                        }
                        38 => {
                            if idx + 2 < p.len() && p[idx + 1] == 5 {
                                self.current_style.fg = Self::color_256(p[idx + 2]);
                                idx += 2;
                            } else if idx + 4 < p.len() && p[idx + 1] == 2 {
                                let r = p[idx + 2] as u8;
                                let g = p[idx + 3] as u8;
                                let b = p[idx + 4] as u8;
                                self.current_style.fg = rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff);
                                idx += 4;
                            }
                        }
                        48 => {
                            if idx + 2 < p.len() && p[idx + 1] == 5 {
                                self.current_style.bg = Self::color_256(p[idx + 2]);
                                idx += 2;
                            } else if idx + 4 < p.len() && p[idx + 1] == 2 {
                                let r = p[idx + 2] as u8;
                                let g = p[idx + 3] as u8;
                                let b = p[idx + 4] as u8;
                                self.current_style.bg = rgba(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff);
                                idx += 4;
                            }
                        }
                        _ => {}
                    }
                    idx += 1;
                }
            }
            _ => {}
        }
    }
}

pub fn keystroke_to_bytes(keystroke: &Keystroke, app_cursor: bool) -> Option<Vec<u8>> {
    let key = keystroke.key.as_str();

    let has_modifiers = keystroke.modifiers.shift
        || keystroke.modifiers.alt
        || keystroke.modifiers.control;

    if has_modifiers && matches!(key, "up" | "down" | "right" | "left") {
        let mut mod_code = 1;
        if keystroke.modifiers.shift { mod_code += 1; }
        if keystroke.modifiers.alt { mod_code += 2; }
        if keystroke.modifiers.control { mod_code += 4; }
        let dir = match key {
            "up" => 'A',
            "down" => 'B',
            "right" => 'C',
            "left" => 'D',
            _ => unreachable!(),
        };
        return Some(format!("\x1b[1;{}{}", mod_code, dir).into_bytes());
    }

    match key {
        "space" => {
            if keystroke.modifiers.control {
                return Some(vec![0x00]);
            }
            return Some(b" ".to_vec());
        }
        "enter" => return Some(b"\r".to_vec()),
        "escape" => return Some(b"\x1b".to_vec()),
        "backspace" => return Some(b"\x7f".to_vec()),
        "tab" => {
            if keystroke.modifiers.shift {
                return Some(b"\x1b[Z".to_vec());
            }
            return Some(b"\t".to_vec());
        }
        "up" => return Some(if app_cursor { b"\x1bOA".to_vec() } else { b"\x1b[A".to_vec() }),
        "down" => return Some(if app_cursor { b"\x1bOB".to_vec() } else { b"\x1b[B".to_vec() }),
        "right" => return Some(if app_cursor { b"\x1bOC".to_vec() } else { b"\x1b[C".to_vec() }),
        "left" => return Some(if app_cursor { b"\x1bOD".to_vec() } else { b"\x1b[D".to_vec() }),
        "home" => return Some(b"\x1b[H".to_vec()),
        "end" => return Some(b"\x1b[F".to_vec()),
        "pageup" => return Some(b"\x1b[5~".to_vec()),
        "pagedown" => return Some(b"\x1b[6~".to_vec()),
        "delete" => return Some(b"\x1b[3~".to_vec()),
        _ => {}
    }

    if keystroke.modifiers.control && key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if ch.is_ascii_alphabetic() {
            let upper = ch.to_ascii_uppercase();
            return Some(vec![(upper as u8) - b'@']);
        }
        match ch {
            '@' => return Some(vec![0x00]),
            '[' => return Some(vec![0x1b]),
            '\\' => return Some(vec![0x1c]),
            ']' => return Some(vec![0x1d]),
            '^' => return Some(vec![0x1e]),
            '_' => return Some(vec![0x1f]),
            '?' => return Some(vec![0x7f]),
            _ => {}
        }
    }

    if keystroke.modifiers.alt && key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if ch.is_ascii() {
            return Some(vec![0x1b, ch as u8]);
        }
    }

    if !keystroke.modifiers.control && !keystroke.modifiers.alt {
        if let Some(key_char) = &keystroke.key_char {
            return Some(key_char.as_bytes().to_vec());
        }
    }

    if key.len() == 1 && !keystroke.modifiers.control {
        let ch = key.chars().next().unwrap();
        let ch = if keystroke.modifiers.shift { ch.to_ascii_uppercase() } else { ch };
        return Some(ch.to_string().into_bytes());
    }

    None
}

#[derive(Debug)]
pub enum TerminalWsEvent {
    Opened,
    Binary(Vec<u8>),
    Text(String),
    Closed,
    Error(String),
}

pub struct WebTerminalView {
    pub title: String,
    pub session_type: String, // "ssh" | "local"
    pub connection_id: Option<String>,
    pub screen: TerminalScreen,
    pub vte_parser: vte::Parser,
    pub ws: Option<web_sys::WebSocket>,
    pub focus_handle: Option<FocusHandle>,
    pub is_connected: bool,
    pub status_message: String,
    // Keep closures alive
    _on_message: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
    _on_open: Option<Closure<dyn FnMut(web_sys::Event)>>,
    _on_close: Option<Closure<dyn FnMut(web_sys::CloseEvent)>>,
    _on_error: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

impl WebTerminalView {
    pub fn new(title: String) -> Self {
        Self::new_internal(title, "local".to_string(), None)
    }

    pub fn new_ssh(title: String, connection_id: String) -> Self {
        Self::new_internal(title, "ssh".to_string(), Some(connection_id))
    }

    pub fn new_local(title: String) -> Self {
        Self::new_internal(title, "local".to_string(), None)
    }

    fn new_internal(title: String, session_type: String, connection_id: Option<String>) -> Self {
        Self {
            title,
            session_type,
            connection_id,
            screen: TerminalScreen::new(DEFAULT_COLS, DEFAULT_ROWS),
            vte_parser: vte::Parser::new(),
            ws: None,
            focus_handle: None,
            is_connected: false,
            status_message: "Connecting to server...".to_string(),
            _on_message: None,
            _on_open: None,
            _on_close: None,
            _on_error: None,
        }
    }

    pub fn handle_event(&mut self, event: TerminalWsEvent) {
        match event {
            TerminalWsEvent::Opened => {
                let req = if self.session_type == "ssh" {
                    serde_json::json!({
                        "type": "connect",
                        "session_type": "ssh",
                        "connection_id": self.connection_id,
                        "cols": DEFAULT_COLS,
                        "rows": DEFAULT_ROWS,
                        "term": "xterm-256color",
                    })
                } else {
                    serde_json::json!({
                        "type": "connect",
                        "session_type": "local",
                        "connection_id": "local",
                        "cols": DEFAULT_COLS,
                        "rows": DEFAULT_ROWS,
                        "term": "xterm-256color",
                    })
                };
                if let Some(ws) = &self.ws {
                    let _ = ws.send_with_str(&req.to_string());
                }
            }
            TerminalWsEvent::Text(text) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if msg_type == "connected" {
                        if let Some(ws) = &self.ws {
                            let ready = serde_json::json!({ "type": "ready" });
                            let _ = ws.send_with_str(&ready.to_string());
                        }
                        self.is_connected = true;
                        self.status_message.clear();
                    } else if msg_type == "error" {
                        let err = json.get("message").and_then(|v| v.as_str()).unwrap_or("Server error");
                        self.status_message = format!("Error: {err}");
                    }
                }
            }
            TerminalWsEvent::Binary(bytes) => {
                self.vte_parser.advance(&mut self.screen, &bytes);
            }
            TerminalWsEvent::Closed => {
                self.is_connected = false;
                self.status_message = "Disconnected from terminal session".to_string();
            }
            TerminalWsEvent::Error(err) => {
                self.status_message = err;
            }
        }
    }

    pub fn init_connection(&mut self, cx: &mut Context<Self>) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                self.status_message = "No browser window object".to_string();
                return;
            }
        };

        let location = window.location();
        let host = location.host().unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
        let ws_proto = if protocol == "https:" { "wss:" } else { "ws:" };
        let ws_url = format!("{}//{}/ws", ws_proto, host);

        let ws = match web_sys::WebSocket::new(&ws_url) {
            Ok(s) => s,
            Err(e) => {
                self.status_message = format!("WebSocket init failed: {e:?}");
                return;
            }
        };

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let (tx, rx) = flume::unbounded::<TerminalWsEvent>();

        // 1. On Open Handshake
        let on_open = {
            let tx = tx.clone();
            Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let _ = tx.send(TerminalWsEvent::Opened);
            }) as Box<dyn FnMut(web_sys::Event)>)
        };
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        self._on_open = Some(on_open);

        // 2. On Message (Data & Control)
        let on_message = {
            let tx = tx.clone();
            Closure::wrap(Box::new(move |ev: web_sys::MessageEvent| {
                let data = ev.data();
                if let Ok(buf) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
                    let uint8_arr = js_sys::Uint8Array::new(&buf);
                    let mut bytes = vec![0u8; uint8_arr.length() as usize];
                    uint8_arr.copy_to(&mut bytes);
                    let _ = tx.send(TerminalWsEvent::Binary(bytes));
                } else if let Some(text) = data.as_string() {
                    let _ = tx.send(TerminalWsEvent::Text(text));
                }
            }) as Box<dyn FnMut(web_sys::MessageEvent)>)
        };
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        self._on_message = Some(on_message);

        // 3. On Close
        let on_close = {
            let tx = tx.clone();
            Closure::wrap(Box::new(move |_ev: web_sys::CloseEvent| {
                let _ = tx.send(TerminalWsEvent::Closed);
            }) as Box<dyn FnMut(web_sys::CloseEvent)>)
        };
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        self._on_close = Some(on_close);

        // 4. On Error
        let on_error = {
            let tx = tx.clone();
            Closure::wrap(Box::new(move |_ev: web_sys::Event| {
                let _ = tx.send(TerminalWsEvent::Error("WebSocket connection error".to_string()));
            }) as Box<dyn FnMut(web_sys::Event)>)
        };
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        self._on_error = Some(on_error);

        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let cx_handle = cx.clone();
            async move {
                while let Ok(ev) = rx.recv_async().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                this.handle_event(ev);
                                cx.notify();
                            });
                        }
                    });
                }
            }
        }).detach();

        self.ws = Some(ws);
    }

    pub fn send_input(&self, bytes: &[u8]) {
        if let Some(ws) = &self.ws {
            if ws.ready_state() == web_sys::WebSocket::OPEN {
                let _ = ws.send_with_u8_array(bytes);
            }
        }
    }
}

impl Render for WebTerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.ws.is_none() {
            self.init_connection(cx);
        }

        let focus_handle = self.focus_handle.get_or_insert_with(|| cx.focus_handle()).clone();
        let cursor_row = self.screen.cursor_row;
        let cursor_col = self.screen.cursor_col;
        let cursor_visible = self.screen.cursor_visible;
        let app_cursor = self.screen.app_cursor;

        let rows_elements = self.screen.cells.iter().enumerate().map(|(r, row)| {
            let mut spans = Vec::new();
            let mut c = 0;
            while c < row.len() {
                let is_cursor = r == cursor_row && c == cursor_col && cursor_visible;
                let cell = &row[c];

                if is_cursor {
                    let display_ch = if cell.ch == ' ' { ' ' } else { cell.ch };
                    spans.push(
                        div()
                            .bg(rgb(0x58a6ff))
                            .text_color(rgb(0x0d1117))
                            .child(display_ch.to_string())
                    );
                    c += 1;
                } else {
                    let mut chunk = String::new();
                    let style = cell.style;
                    while c < row.len() {
                        let next_is_cursor = r == cursor_row && c == cursor_col && cursor_visible;
                        if next_is_cursor || row[c].style != style {
                            break;
                        }
                        chunk.push(row[c].ch);
                        c += 1;
                    }

                    if !chunk.is_empty() {
                        let (fg, bg) = if style.inverse {
                            (style.bg, style.fg)
                        } else {
                            (style.fg, style.bg)
                        };

                        let mut el = div().text_color(fg);
                        if bg != rgb(0x0d1117) {
                            el = el.bg(bg);
                        }
                        spans.push(el.child(chunk));
                    }
                }
            }

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(18.0))
                .children(spans)
        });

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0d1117))
            .p_4()
            .font_family("JetBrains Mono")
            .text_sm()
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _window, _cx| {
                if let Some(bytes) = keystroke_to_bytes(&event.keystroke, app_cursor) {
                    this.send_input(&bytes);
                }
            }))
            .when(!self.status_message.is_empty(), |this| {
                this.child(
                    div()
                        .px_3()
                        .py_1()
                        .mb_2()
                        .rounded_md()
                        .bg(rgb(0x21262d))
                        .text_color(rgb(0x8b949e))
                        .text_xs()
                        .child(self.status_message.clone())
                )
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .children(rows_elements)
            )
    }
}

