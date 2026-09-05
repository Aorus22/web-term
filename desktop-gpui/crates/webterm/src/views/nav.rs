//! Navigation shell view: left sidebar and main content views.

use gpui::*;
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

    // Sidebar items matching web client (Lucide SVGs)
    let items = [
        (View::Hosts, "Hosts", crate::icons::SERVER_SVG),
        (View::Keys, "SSH Keys", crate::icons::KEY_SVG),
        (View::Forwards, "Port Forwards", crate::icons::ARROW_LEFT_RIGHT_SVG),
        (View::Sftp, "SFTP", crate::icons::FILES_SVG),
    ];

    let is_dark = app.is_dark();
    let bg_color = app.bg_color();
    let sidebar_bg = app.bg_color();
    let text_color = app.text_color();
    let _muted_text = app.muted_text();
    let border_color = app.border_color();

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

    let sftp_context_menu_overlay = if app.sftp_manager.context_menu.is_some() {
        crate::views::sftp::render_sftp_context_menu(app, is_dark, cx)
    } else {
        None
    };

    let sftp_modal_overlay = if app.sftp_manager.modal.is_some() {
        crate::views::sftp::render_sftp_modal(app, is_dark, cx)
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
            .bg(app.card_bg())
            .border_1()
            .border_color(app.primary_color())
            .shadow_lg()
            .text_sm()
            .text_color(text_color)
            .child(msg)
            .child(
                div()
                    .cursor_pointer()
                    .text_xs()
                    .text_color(app.muted_text())
                    .hover(|s| s.text_color(text_color))
                    .child(svg().data(crate::icons::X_SVG).size(px(14.0)).text_color(app.muted_text()))
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
            this.open_new_tab_page(cx);
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
        // Left Sidebar (collapsible)
        .children(if app.sidebar_open {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .w(px(200.0))
                    .h_full()
                    .bg(sidebar_bg)
                    .border_r_1()
                    .border_color(border_color)
                    .px_3()
                    .py_4()
                    .justify_between()
                    // Top header & items
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            // Brand Header: Clean "WebTerm" matching web client
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .mb_3()
                                    .child(
                                        div()
                                            .text_lg()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(text_color)
                                            .child("WebTerm"),
                                    ),
                            )
                            // Navigation items
                            .children(items.into_iter().map(|(view, label, icon)| {
                                let is_active = active_view == view && (view != View::Hosts || app.show_hosts_catalog);
                                let accent_color = app.accent_color();
                                let accent_fg = app.accent_fg();
                                let item_bg = if is_active {
                                    accent_color
                                } else {
                                    sidebar_bg
                                };
                                let item_hover = if is_active {
                                    accent_color
                                } else {
                                    app.muted_bg()
                                };
                                let item_text_color = if is_active {
                                    accent_fg
                                } else {
                                    text_color
                                };
                                let icon_color = if is_active {
                                    accent_fg
                                } else {
                                    app.muted_text()
                                };

                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(item_bg)
                                    .hover(move |s| s.bg(item_hover))
                                    .cursor_pointer()
                                    .child(svg().data(icon).size(px(16.0)).text_color(icon_color))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(if is_active {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::MEDIUM
                                            })
                                            .text_color(item_text_color)
                                            .child(label),
                                    )
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        this.show_new_tab_popover = false;
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
                    // Bottom footer: Settings pinned to bottom
                    .child(
                        div()
                            .pt_2()
                            .border_t_1()
                            .border_color(border_color)
                            .child({
                                let is_active = active_view == View::Settings;
                                let accent_color = app.accent_color();
                                let accent_fg = app.accent_fg();
                                let item_bg = if is_active {
                                    accent_color
                                } else {
                                    sidebar_bg
                                };
                                let item_hover = if is_active {
                                    accent_color
                                } else {
                                    app.muted_bg()
                                };
                                let item_text_color = if is_active {
                                    accent_fg
                                } else {
                                    text_color
                                };
                                let icon_color = if is_active {
                                    accent_fg
                                } else {
                                    app.muted_text()
                                };

                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_3()
                                    .py_2()
                                    .rounded_lg()
                                    .bg(item_bg)
                                    .hover(move |s| s.bg(item_hover))
                                    .cursor_pointer()
                                    .child(svg().data(crate::icons::SETTINGS_SVG).size(px(16.0)).text_color(icon_color))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(if is_active {
                                                FontWeight::BOLD
                                            } else {
                                                FontWeight::MEDIUM
                                            })
                                            .text_color(item_text_color)
                                            .child("Settings"),
                                    )
                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                        this.show_new_tab_popover = false;
                                        this.active_view = View::Settings;
                                        this.show_hosts_catalog = false;
                                        cx.notify();
                                    }))
                            }),
                    ),
            )
        } else {
            None
        })
        // Main Content Area
        .child(
            div()
                .relative()
                .flex_1()
                .h_full()
                .overflow_hidden()
                // Content Pane (occupies full height with 40px top padding for tab strip)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .size_full()
                        .pt(px(40.0))
                        .overflow_hidden()
                        // Reconnection banner (when active tab is reconnecting or disconnected and viewing terminal)
                        .children(if active_view == View::Hosts && !app.show_hosts_catalog {
                            render_reconnect_banner(app, cx)
                        } else {
                            None
                        })
                        .child(
                            div()
                                .flex_1()
                                .w_full()
                                .overflow_hidden()
                                .child(content_pane),
                        ),
                )
                // Backdrop when popover is open to dismiss on click outside
                .children(if app.show_new_tab_popover {
                    Some(
                        div()
                            .absolute()
                            .inset_0()
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                this.show_new_tab_popover = false;
                                cx.notify();
                            })),
                    )
                } else {
                    None
                })
                // Top Tab Strip across all views (rendered AFTER content_pane so it and its popovers float on top!)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .right_0()
                        .h(px(40.0))
                        .child(render_tab_strip(app, cx)),
                ),
        )
        .children(modal_overlay)
        .children(conn_modal_overlay)
        .children(import_modal_overlay)
        .children(add_key_modal_overlay)
        .children(passphrase_modal_overlay)
        .children(forward_modal_overlay)
        .children(delete_forward_modal_overlay)
        .children(sftp_context_menu_overlay)
        .children(sftp_modal_overlay)
        .children(notification_toast)
        .into_any_element()
}

fn render_content_pane(
    app: &mut AppState,
    view: View,
    _is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let bg = app.bg_color();
    match view {
        View::NewTab => {
            div()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(crate::views::new_tab::render_new_tab_page(app, cx))
                .into_any_element()
        }
        View::Hosts => {
            let active_tab_view = app.session_manager.active_tab().and_then(|t| t.view.clone());
            let host_or_term = if app.show_hosts_catalog || active_tab_view.is_none() {
                render_hosts_view(app, cx)
            } else if let Some(term_view) = active_tab_view {
                div().size_full().bg(bg).child(term_view).into_any_element()
            } else {
                render_hosts_view(app, cx)
            };

            div()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(host_or_term)
                .into_any_element()
        }
        View::Keys => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(crate::views::keys::render_keys_view(app, cx))
                .into_any_element()
        }
        View::Forwards => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(crate::views::forwards::render_forwards_view(app, cx))
                .into_any_element()
        }
        View::Sftp => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(crate::views::sftp::render_sftp_view(app, cx))
                .into_any_element()
        }
        View::Settings => {
            div()
                .flex()
                .flex_col()
                .size_full()
                .overflow_hidden()
                .bg(bg)
                .child(crate::views::settings::render_settings_view(app, cx))
                .into_any_element()
        }
    }
}
