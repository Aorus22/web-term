//! Terminal Tab Strip UI component.

use gpui::*;
use gpui_component::{Icon, IconName};
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;
use crate::session::SessionStatus;

/// Renders the horizontal tab strip for open terminal sessions.
pub fn render_tab_strip(app: &mut AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let active_index = app.session_manager.active_index();
    let border_color = if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) };
    let bar_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let active_color = if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) };

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
                if is_dark { rgb(0x1e1e24) } else { rgb(0xf1f5f9) }
            };
            let text_color = if is_active {
                if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) }
            } else {
                muted_text
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
                    active_color
                } else {
                    rgba(0x00000000)
                })
                .cursor_pointer()
                .hover(|s| s.bg(if is_active {
                    tab_bg
                } else {
                    if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.show_hosts_catalog = true;
                    cx.notify();
                }))
                .child(
                    Icon::new(IconName::HardDrive)
                        .size(px(13.0))
                        .text_color(if is_active { active_color } else { muted_text })
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(text_color)
                        .child("Hosts"),
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
                if is_dark { rgb(0x1e1e24) } else { rgb(0xf1f5f9) }
            };

            let text_color = if is_active {
                if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) }
            } else {
                muted_text
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
                    active_color
                } else {
                    rgba(0x00000000)
                })
                .cursor_pointer()
                .hover(|s| s.bg(if is_active {
                    tab_bg
                } else {
                    if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.show_hosts_catalog = false;
                    this.switch_tab(idx, cx);
                }))
                // Status dot
                .child(
                    div()
                        .size(px(6.0))
                        .rounded_full()
                        .bg(dot_color),
                )
                // Terminal icon
                .child(
                    Icon::new(IconName::SquareTerminal)
                        .size(px(13.0))
                        .text_color(if is_active { active_color } else { muted_text })
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
                        .child(
                            Icon::new(IconName::Close)
                                .size(px(11.0))
                                .text_color(if is_dark { rgb(0x71717a) } else { rgb(0x94a3b8) })
                        )
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
                .child(
                    Icon::new(IconName::Plus)
                        .size(px(13.0))
                        .text_color(muted_text)
                )
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.toggle_new_tab_modal(cx);
                })),
        )
}
