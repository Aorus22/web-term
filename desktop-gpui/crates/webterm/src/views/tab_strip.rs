//! Terminal Tab Strip UI component.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;
use crate::session::SessionStatus;

/// Renders the horizontal tab strip for open terminal sessions.
pub fn render_tab_strip(app: &mut AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let active_index = app.session_manager.active_index();
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let bar_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf1f5f9) };

    let tabs_len = app.session_manager.tab_count();

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(38.0))
        .bg(bar_bg)
        .border_b_1()
        .border_color(border_color)
        .px_2()
        .gap_1()
        // Persistent Hosts Catalog Tab
        .child({
            let is_active = app.show_hosts_catalog;
            let tab_bg = if is_active {
                if is_dark { rgb(0x27272a) } else { rgb(0xffffff) }
            } else {
                if is_dark { rgb(0x1f1f23) } else { rgb(0xe2e8f0) }
            };
            let text_color = if is_active {
                if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) }
            } else {
                if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) }
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(30.0))
                .px_3()
                .gap_2()
                .rounded_t_md()
                .bg(tab_bg)
                .border_t_2()
                .border_color(if is_active {
                    if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                } else {
                    rgba(0x00000000)
                })
                .cursor_pointer()
                .hover(|s| s.bg(if is_active {
                    tab_bg
                } else {
                    if is_dark { rgb(0x2d2d32) } else { rgb(0xd8e0e9) }
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.show_hosts_catalog = true;
                    cx.notify();
                }))
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(text_color)
                        .child("🖥️ Hosts"),
                )
        })
        // Tabs container
        .children((0..tabs_len).map(|idx| {
            let is_active = !app.show_hosts_catalog && idx == active_index;
            let tab = &app.session_manager.tabs()[idx];
            let title = tab.title.clone();
            let status = tab.status.clone();

            let dot_color = match &status {
                SessionStatus::Connected => rgb(0x22c55e), // Green
                SessionStatus::Connecting | SessionStatus::Reconnecting { .. } => rgb(0xeab308), // Yellow
                SessionStatus::Disconnected(_) => rgb(0xef4444), // Red
            };

            let tab_bg = if is_active {
                if is_dark { rgb(0x27272a) } else { rgb(0xffffff) }
            } else {
                if is_dark { rgb(0x1f1f23) } else { rgb(0xe2e8f0) }
            };

            let text_color = if is_active {
                if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) }
            } else {
                if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) }
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(30.0))
                .px_3()
                .gap_2()
                .rounded_t_md()
                .bg(tab_bg)
                .border_t_2()
                .border_color(if is_active {
                    if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                } else {
                    rgba(0x00000000)
                })
                .cursor_pointer()
                .hover(|s| s.bg(if is_active {
                    tab_bg
                } else {
                    if is_dark { rgb(0x2d2d32) } else { rgb(0xd8e0e9) }
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.show_hosts_catalog = false;
                    this.switch_tab(idx, cx);
                }))
                // Status dot
                .child(
                    div()
                        .size(px(7.0))
                        .rounded_full()
                        .bg(dot_color),
                )
                // Tab title
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(text_color)
                        .child(title),
                )
                // Close button ('×')
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(16.0))
                        .rounded_sm()
                        .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                        .text_xs()
                        .text_color(if is_dark { rgb(0x71717a) } else { rgb(0x94a3b8) })
                        .child("×")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                            this.close_tab(idx, cx);
                        })),
                )
        }))
        // '+' New Tab button
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded_md()
                .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                .cursor_pointer()
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) })
                .child("+")
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.toggle_new_tab_modal(cx);
                })),
        )
}
