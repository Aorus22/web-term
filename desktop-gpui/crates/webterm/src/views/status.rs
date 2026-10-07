//! Status view rendering Starting, Ready, and Failed application states.
//!
//! Manual failure test procedure:
//! 1. In `%APPDATA%\webterm-desktop\settings.json` (or `~/.config/webterm-desktop/settings.json`),
//!    set `"backend_path": "nonexistent_binary"`.
//! 2. Launch the application: `cargo run -p webterm`.
//! 3. Observe the honest Failed state display:
//!    - Red header and failure reason.
//!    - Redacted stderr tail (no encryption key material exposed).
//!    - Functional "Retry" and "Quit" buttons.
//! 4. Restore settings.json or remove the invalid backend_path override.

use gpui::*;
use webterm_supervisor::BackendStatus;

/// Theme colours for the full-window status pages.
///
/// These used to be hardcoded zinc literals, which made `Desktop (GTK)` look
/// half-applied the moment the backend was still starting (or had failed): the
/// status page kept painting zinc while the shell followed the desktop. The
/// caller resolves them through the app's own accessors.
#[derive(Clone, Copy)]
pub struct StatusColors {
    pub background: Rgba,
    pub foreground: Rgba,
    pub muted_foreground: Rgba,
    pub card: Rgba,
    pub muted: Rgba,
    pub border: Rgba,
    pub destructive: Rgba,
    pub primary: Rgba,
    pub primary_foreground: Rgba,
}

/// Redact encryption key material and secret tokens from stderr lines.
///
/// Keeps at most 10 newest lines and replaces key material with `[REDACTED]`.
pub fn redact_key_material(text: &str, key_secret: Option<&str>) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let tail_slice = if lines.len() > 10 {
        &lines[lines.len() - 10..]
    } else {
        &lines[..]
    };

    tail_slice
        .iter()
        .map(|line| {
            let mut sanitized = line.to_string();
            if let Some(key) = key_secret {
                if !key.is_empty() {
                    if sanitized.contains(key) {
                        sanitized = sanitized.replace(key, "[REDACTED KEY]");
                    }
                    if key.len() >= 8 {
                        let prefix = &key[..8];
                        if sanitized.contains(prefix) {
                            sanitized = sanitized.replace(prefix, "[REDACTED]");
                        }
                    }
                }
            }
            if sanitized.contains("WEBTERM_ENCRYPTION_KEY") {
                sanitized = "[REDACTED KEY CONFIGURATION]".to_string();
            }
            sanitized
        })
        .collect()
}

/// Render the backend status page (Starting or Failed state).
pub fn render_status_page<V: 'static>(
    status: &BackendStatus,
    key_secret: Option<&str>,
    colors: StatusColors,
    on_retry: impl Fn(&mut V, &MouseDownEvent, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> AnyElement {
    match status {
        BackendStatus::Starting => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .items_center()
                .justify_center()
                .bg(colors.background)
                .text_color(colors.foreground)
                .child(
                    div()
                        .text_xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Starting backend…"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors.muted_foreground)
                        .mt_2()
                        .child(
                            "Initializing Go server, capturing handshake port and verifying readiness",
                        ),
                )
                .into_any_element()
        }
        BackendStatus::Failed {
            reason,
            stderr_tail,
        } => {
            let redacted_lines = redact_key_material(stderr_tail, key_secret);
            let retry_handler = on_retry.clone();

            div()
                .flex()
                .flex_col()
                .size_full()
                .items_center()
                .justify_center()
                .bg(colors.background)
                .text_color(colors.foreground)
                .p_8()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w(px(700.0))
                        .bg(colors.card)
                        .border_1()
                        .border_color(colors.destructive)
                        .rounded_lg()
                        .p_6()
                        .shadow_lg()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(colors.destructive)
                                .child("Backend Startup Failed"),
                        )
                        .child(
                            div()
                                .mt_2()
                                .text_sm()
                                .text_color(colors.foreground)
                                .child(format!("Reason: {}", reason)),
                        )
                        .child(
                            div()
                                .mt_4()
                                .p_3()
                                .bg(colors.muted)
                                .border_1()
                                .border_color(colors.border)
                                .rounded_md()
                                .text_xs()
                                .font_family("JetBrains Mono")
                                .text_color(colors.foreground)
                                .children(redacted_lines.into_iter().map(|line| div().child(line))),
                        )
                        .child(
                            div()
                                .mt_6()
                                .flex()
                                .gap_3()
                                .justify_end()
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(colors.muted)
                                        .id("status-01")
                                        .hover(|s| s.bg(colors.border))
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .child("Quit")
                                        .on_mouse_down(MouseButton::Left, |_ev, _window, cx| {
                                            cx.quit();
                                        }),
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(colors.primary)
                                        .id("status-02")
                                        .hover(|s| s.bg(colors.primary))
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors.primary_foreground)
                                        .child("Retry")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, ev, window, cx| {
                                                retry_handler(this, ev, window, cx);
                                            }),
                                        ),
                                ),
                        ),
                )
                .into_any_element()
        }
        BackendStatus::Crashed { exit_code } => {
            let code_str = exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".into());
            div()
                .flex()
                .flex_col()
                .size_full()
                .items_center()
                .justify_center()
                .bg(colors.background)
                .text_color(colors.foreground)
                .child(
                    div()
                        .text_lg()
                        .text_color(colors.destructive)
                        .child(format!("Backend process crashed (code: {})", code_str)),
                )
                .into_any_element()
        }
        BackendStatus::Ready => div().into_any_element(),
    }
}
