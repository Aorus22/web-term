//! Navigation shell view: left sidebar and main content views.

use gpui::*;
use gpui_component::{Icon, IconName};
use webterm_settings::Theme as SettingsTheme;
use crate::actions::{
    CloseTab, JumpTab1, JumpTab2, JumpTab3, JumpTab4, JumpTab5, JumpTab6, JumpTab7, JumpTab8,
    JumpTab9, NewTab, NextTab, PrevTab,
};
use crate::app_state::{AppState, View};
use crate::views::hosts::render_hosts_view;
use crate::views::new_tab_modal::render_new_tab_modal;
use crate::views::reconnect_banner::render_reconnect_banner;
use crate::views::tab_strip::render_tab_strip;

/// Render the left navigation sidebar and active content pane.
pub fn render_nav_shell(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let active_view = app.active_view;
    let current_theme = app.theme;

    // Sidebar items with modern icons
    let items = [
        (View::Hosts, "Hosts", IconName::HardDrive),
        (View::Keys, "SSH Keys", IconName::CircleUser),
        (View::Forwards, "Port Forwards", IconName::Network),
        (View::Sftp, "Files", IconName::Folder),
        (View::Settings, "Settings", IconName::Settings),
    ];

    let is_dark = current_theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x121214) } else { rgb(0xf8fafc) };
    let sidebar_bg = if is_dark { rgb(0x18181b) } else { rgb(0xffffff) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let border_color = if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) };

    let show_modal = app.show_new_tab_modal;
    let content_pane = render_content_pane(app, active_view, is_dark, cx);

    let modal_overlay = if show_modal {
        Some(render_new_tab_modal(app, cx))
    } else {
        None
    };

    let conn_modal_overlay = if app.connection_modal.is_some() {
        Some(crate::views::connection_modal::render_connection_modal(app, cx))
    } else {
        None
    };

    let import_modal_overlay = if app.show_import_modal {
        Some(crate::views::connection_modal::render_import_modal(app, cx))
    } else {
        None
    };

    let add_key_modal_overlay = if app.show_add_key_modal {
        Some(crate::views::keys::render_add_key_modal(app, cx))
    } else {
        None
    };

    let passphrase_modal_overlay = if app.pending_passphrase_conn.is_some() {
        Some(crate::views::passphrase_modal::render_passphrase_modal(app, cx))
    } else {
        None
    };

    let forward_modal_overlay = if app.forward_modal.is_some() {
        Some(crate::views::forwards::render_forward_modal(app, cx))
    } else {
        None
    };

    let delete_forward_modal_overlay = if app.delete_forward_target.is_some() {
        Some(crate::views::forwards::render_delete_forward_modal(app, cx))
    } else {
        None
    };

    let notification_toast = app.notification.clone().map(|msg| {
        div()
            .absolute()
            .top(px(16.0))
            .right(px(16.0))
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .px_4()
            .py_2()
            .rounded_lg()
            .bg(if is_dark { rgb(0x18181b) } else { rgb(0xffffff) })
            .border_1()
            .border_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) })
            .shadow_lg()
            .text_sm()
            .text_color(text_color)
            .child(msg)
            .child(
                div()
                    .cursor_pointer()
                    .text_xs()
                    .text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) })
                    .hover(|s| s.text_color(text_color))
                    .child("×")
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                        this.dismiss_notification(cx);
                    })),
            )
            .into_any_element()
    });

    div()
        .relative()
        .flex()
        .size_full()
        .bg(bg_color)
        .text_color(text_color)
        // Tab shortcuts
        .on_action(cx.listener(|this, _: &NewTab, _window, cx| {
            this.toggle_new_tab_modal(cx);
        }))
        .on_action(cx.listener(|this, _: &CloseTab, _window, cx| {
            let idx = this.session_manager.active_index();
            this.close_tab(idx, cx);
        }))
        .on_action(cx.listener(|this, _: &NextTab, _window, cx| {
            this.cycle_next_tab(cx);
        }))
        .on_action(cx.listener(|this, _: &PrevTab, _window, cx| {
            this.cycle_prev_tab(cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab1, _window, cx| {
            this.jump_to_tab(0, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab2, _window, cx| {
            this.jump_to_tab(1, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab3, _window, cx| {
            this.jump_to_tab(2, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab4, _window, cx| {
            this.jump_to_tab(3, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab5, _window, cx| {
            this.jump_to_tab(4, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab6, _window, cx| {
            this.jump_to_tab(5, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab7, _window, cx| {
            this.jump_to_tab(6, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab8, _window, cx| {
            this.jump_to_tab(7, cx);
        }))
        .on_action(cx.listener(|this, _: &JumpTab9, _window, cx| {
            this.jump_to_tab(8, cx);
        }))
        // Left Sidebar
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(220.0))
                .h_full()
                .bg(sidebar_bg)
                .border_r_1()
                .border_color(border_color)
                .p_3()
                .justify_between()
                // Top header & items
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        // Brand Header
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2p5()
                                .px_2()
                                .py_2()
                                .mb_2()
                                .child(
                                    div()
                                        .size(px(28.0))
                                        .rounded_lg()
                                        .bg(rgb(0x0284c7))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(rgb(0xffffff))
                                        .child(Icon::new(IconName::SquareTerminal).size(px(16.0))),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(text_color)
                                                .child("WebTerm"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted_text)
                                                .child("Desktop Client"),
                                        ),
                                ),
                        )
                        // Navigation items
                        .children(items.into_iter().map(|(view, label, icon)| {
                            let is_active = active_view == view;
                            let item_bg = if is_active {
                                if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }
                            } else {
                                sidebar_bg
                            };
                            let item_hover = if is_dark { rgb(0x202024) } else { rgb(0xf1f5f9) };
                            let icon_color = if is_active {
                                if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                            } else {
                                muted_text
                            };

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2p5()
                                .px_3()
                                .py_2()
                                .rounded_lg()
                                .bg(item_bg)
                                .hover(move |s| s.bg(item_hover))
                                .cursor_pointer()
                                .child(Icon::new(icon).size(px(16.0)).text_color(icon_color))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(if is_active {
                                            FontWeight::SEMIBOLD
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(if is_active { text_color } else { muted_text })
                                        .child(label),
                                )
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                    if view == View::Sftp {
                                        this.navigate_to_sftp(cx);
                                    } else {
                                        this.active_view = view;
                                        if view == View::Hosts {
                                            this.show_hosts_catalog = true;
                                        }
                                        cx.notify();
                                    }
                                }))
                        })),
                )
                // Bottom footer: Theme Toggle
                .child(
                    div()
                        .pt_3()
                        .border_t_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .px_3()
                                .py_2()
                                .rounded_lg()
                                .bg(if is_dark { rgb(0x202024) } else { rgb(0xf1f5f9) })
                                .hover(|s| s.bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) }))
                                .cursor_pointer()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            Icon::new(if is_dark { IconName::Moon } else { IconName::Sun })
                                                .size(px(14.0))
                                                .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0xf59e0b) }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(text_color)
                                                .child(if is_dark { "Dark Theme" } else { "Light Theme" }),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted_text)
                                        .child("Switch"),
                                )
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.toggle_theme(cx);
                                })),
                        ),
                ),
        )
        // Main Content Area
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .h_full()
                .child(content_pane),
        )
        .children(modal_overlay)
        .children(conn_modal_overlay)
        .children(import_modal_overlay)
        .children(add_key_modal_overlay)
        .children(passphrase_modal_overlay)
        .children(forward_modal_overlay)
        .children(delete_forward_modal_overlay)
        .children(notification_toast)
        .into_any_element()
}

fn render_content_pane(
    app: &mut AppState,
    view: View,
    _is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    match view {
        View::Hosts => {
            let active_tab_view = app.session_manager.active_tab().and_then(|t| t.view.clone());
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                // Top Tab Strip
                .child(render_tab_strip(app, cx))
                // Reconnection banner (when active tab is reconnecting or disconnected)
                .children(render_reconnect_banner(app, cx))
                // Active Terminal or Hosts Catalog
                .child({
                    let host_or_term = if app.show_hosts_catalog || active_tab_view.is_none() {
                        render_hosts_view(app, cx)
                    } else if let Some(term_view) = active_tab_view {
                        div().size_full().child(term_view).into_any_element()
                    } else {
                        render_hosts_view(app, cx)
                    };

                    div()
                        .flex_1()
                        .w_full()
                        .overflow_hidden()
                        .child(host_or_term)
                })
                .into_any_element()
        }
        View::Keys => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .child(crate::views::keys::render_keys_view(app, cx))
                .into_any_element()
        }
        View::Forwards => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .child(crate::views::forwards::render_forwards_view(app, cx))
                .into_any_element()
        }
        View::Sftp => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .child(crate::views::sftp::render_sftp_view(app, cx))
                .into_any_element()
        }
        View::Settings => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .child(crate::views::settings::render_settings_view(app, cx))
                .into_any_element()
        }
    }
}
