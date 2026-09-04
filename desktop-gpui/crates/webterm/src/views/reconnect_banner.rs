//! Reconnection banner UI component for disconnected or reconnecting sessions.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;
use crate::session::SessionStatus;

/// Renders the reconnection alert banner if the active tab is not in Connected state.
pub fn render_reconnect_banner(app: &mut AppState, cx: &mut Context<AppState>) -> Option<AnyElement> {
    let active_tab = app.session_manager.active_tab()?;
    let tab_id = active_tab.id;
    let status = active_tab.status.clone();
    let is_dark = app.theme != SettingsTheme::Light;

    match status {
        SessionStatus::Connected | SessionStatus::Connecting => None,
        SessionStatus::Reconnecting {
            attempt,
            next_retry_secs,
        } => {
            let bg_color = if is_dark { rgb(0x451a03) } else { rgb(0xfef3c7) };
            let border_color = if is_dark { rgb(0xb45309) } else { rgb(0xf59e0b) };
            let text_color = if is_dark { rgb(0xfde68a) } else { rgb(0x92400e) };
            let btn_bg = if is_dark { rgb(0x78350f) } else { rgb(0xfde68a) };
            let btn_hover = if is_dark { rgb(0x92400e) } else { rgb(0xfcd34d) };

            Some(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_4()
                    .py_2()
                    .bg(bg_color)
                    .border_b_1()
                    .border_color(border_color)
                    .text_sm()
                    .text_color(text_color)
                    // Message
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_base().child("⚠️"))
                            .child(format!(
                                "Connection lost. Reconnecting in {}s (attempt {}/5)...",
                                next_retry_secs, attempt
                            )),
                    )
                    // Actions
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(btn_bg)
                                    .hover(|s| s.bg(btn_hover))
                                    .cursor_pointer()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Reconnect Now")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        this.reconnect_tab(tab_id, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                                    .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                    .cursor_pointer()
                                    .text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) })
                                    .child("Close Tab")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        let idx = this.session_manager.active_index();
                                        this.close_tab(idx, cx);
                                    })),
                            ),
                    )
                    .into_any_element(),
            )
        }
        SessionStatus::Disconnected(reason) => {
            let bg_color = if is_dark { rgb(0x450a0a) } else { rgb(0xfee2e2) };
            let border_color = if is_dark { rgb(0xb91c1c) } else { rgb(0xef4444) };
            let text_color = if is_dark { rgb(0xfecaca) } else { rgb(0x991b1b) };
            let btn_bg = if is_dark { rgb(0x7f1d1d) } else { rgb(0xfecaca) };
            let btn_hover = if is_dark { rgb(0x991b1b) } else { rgb(0xfca5a5) };

            let reason_text = reason.unwrap_or_else(|| "Connection closed by remote host".to_string());

            Some(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_4()
                    .py_2()
                    .bg(bg_color)
                    .border_b_1()
                    .border_color(border_color)
                    .text_sm()
                    .text_color(text_color)
                    // Message
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().text_base().child("❌"))
                            .child(format!("Session disconnected: {}", reason_text)),
                    )
                    // Actions
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(btn_bg)
                                    .hover(|s| s.bg(btn_hover))
                                    .cursor_pointer()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Reconnect")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        this.reconnect_tab(tab_id, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                                    .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                    .cursor_pointer()
                                    .text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) })
                                    .child("Close Tab")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        let idx = this.session_manager.active_index();
                                        this.close_tab(idx, cx);
                                    })),
                            ),
                    )
                    .into_any_element(),
            )
        }
    }
}
