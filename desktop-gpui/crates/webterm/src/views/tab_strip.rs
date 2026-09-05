//! Terminal Tab Strip UI component: frameless titlebar with sidebar toggle, active session tabs, new tab button, draggable region, and custom window controls.

use gpui::*;
use crate::app_state::{AppState, View};
use crate::icons::{MINIMIZE_SVG, MAXIMIZE_SVG, PANEL_LEFT_SVG, PLUS_SVG, X_SVG};
use crate::session::SessionStatus;

/// Renders the horizontal top bar containing sidebar toggle, open terminal tabs, draggable titlebar area, and custom window controls.
pub fn render_tab_strip(app: &mut AppState, cx: &mut Context<AppState>) -> impl IntoElement {
    let is_dark = app.is_dark();
    let active_index = app.session_manager.active_index();
    let border_color = app.border_color();
    let bar_bg = app.bg_color();
    let muted_text = app.muted_text();

    let tabs_len = app.session_manager.tab_count();

    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(40.0))
        .bg(bar_bg)
        .border_b_1()
        .border_color(border_color)
        .pl_2()
        .pr_0()
        .gap_2()
        // Left: Sidebar Toggle Button (PanelLeft)
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                .child(
                    svg()
                        .data(PANEL_LEFT_SVG)
                        .size(px(16.0))
                        .text_color(muted_text),
                )
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.toggle_sidebar(cx);
                })),
        )
        // Tabs container
        .children((0..tabs_len).map(|idx| {
            let is_active = !app.show_hosts_catalog && app.active_view == View::Hosts && idx == active_index;
            let tab = &app.session_manager.tabs()[idx];
            let title = tab.title.clone();
            let status = tab.status.clone();

            let dot_color = match &status {
                SessionStatus::Connected => rgb(0x22c55e), // Green
                SessionStatus::Connecting | SessionStatus::Reconnecting { .. } => rgb(0xeab308), // Yellow
                SessionStatus::Disconnected(_) => rgb(0xef4444), // Red
            };

            let tab_bg = if is_active {
                app.card_bg()
            } else {
                rgba(0x00000000)
            };

            let tab_border = if is_active {
                app.border_color()
            } else {
                rgba(0x00000000)
            };

            let text_color = if is_active {
                app.text_color()
            } else {
                muted_text
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .h(px(28.0))
                .px_2p5()
                .gap_2()
                .rounded_md()
                .bg(tab_bg)
                .border_1()
                .border_color(tab_border)
                .cursor_pointer()
                .hover(|s| s.bg(if is_active {
                    tab_bg
                } else {
                    if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }
                }))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.show_hosts_catalog = false;
                    this.active_view = View::Hosts;
                    this.switch_tab(idx, cx);
                }))
                // Status dot
                .child(
                    div()
                        .size(px(6.0))
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
                        .child(
                            svg()
                                .data(X_SVG)
                                .size(px(10.0))
                                .text_color(muted_text),
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
                .size(px(28.0))
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                .child(
                    svg()
                        .data(PLUS_SVG)
                        .size(px(14.0))
                        .text_color(muted_text),
                )
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                    this.toggle_new_tab_modal(cx);
                })),
        )
        // Center: Draggable Title Bar Area
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag),
        )
        // Right: Custom Window Controls (Minimize, Maximize/Restore, Close)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .h_full()
                // Minimize Button
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(44.0))
                        .h_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                        .child(
                            svg()
                                .data(MINIMIZE_SVG)
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                            window.minimize_window();
                        }),
                )
                // Maximize / Restore Button
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(44.0))
                        .h_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                        .child(
                            svg()
                                .data(MAXIMIZE_SVG)
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                            window.zoom_window();
                        }),
                )
                // Close Button (Red on hover)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                        .child(
                            svg()
                                .data(X_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(MouseButton::Left, |_, window, _| {
                            window.remove_window();
                        }),
                ),
        )
}
