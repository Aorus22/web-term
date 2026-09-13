//! Navigation shell view: left sidebar and main content views.

use crate::actions::{
    CloseTab, EscapeOverlays, JumpTab1, JumpTab2, JumpTab3, JumpTab4, JumpTab5, JumpTab6, JumpTab7, JumpTab8,
    JumpTab9, NewTab, NextTab, PrevTab,
};
use crate::app_state::{AppState, View};
use crate::views::hosts::render_hosts_view;
use crate::views::reconnect_banner::render_reconnect_banner;
use crate::views::tab_strip::render_tab_strip;
use gpui::*;

/// Render the left navigation sidebar and active content pane.
pub fn render_nav_shell(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let active_view = app.active_view;

    // Sidebar items matching web client (Lucide SVGs)
    let items = [
        (View::Hosts, "Hosts", crate::icons::SERVER_SVG),
        (View::Keys, "SSH Keys", crate::icons::KEY_SVG),
        (
            View::Forwards,
            "Port Forwards",
            crate::icons::ARROW_LEFT_RIGHT_SVG,
        ),
        (View::Sftp, "SFTP", crate::icons::FILES_SVG),
    ];

    let is_dark = app.is_dark();
    let bg_color = app.bg_color();
    let sidebar_bg = app.bg_color();
    let text_color = app.text_color();
    let _muted_text = app.muted_text();
    let border_color = app.border_color();

    let content_pane = render_content_pane(app, active_view, is_dark, cx);

    let sheet_slot = render_sheet_slot(app, cx);

    let passphrase_modal_overlay = if app.pending_passphrase_conn.is_some() {
        Some(crate::views::passphrase_modal::render_passphrase_modal(
            app, cx,
        ))
    } else {
        None
    };

    let confirm_modals_overlay = render_confirm_modals(app, cx);

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
                    .child(
                        svg()
                            .data(crate::icons::X_SVG)
                            .size(px(14.0))
                            .text_color(app.muted_text()),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.dismiss_notification(cx);
                        }),
                    ),
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
        .on_action(cx.listener(|this, _: &EscapeOverlays, window, cx| {
            this.handle_escape(window, cx);
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
                                div().px_2().py_1().mb_3().child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_color)
                                        .child("WebTerm"),
                                ),
                            )
                            // Navigation items
                            .children(items.into_iter().map(|(view, label, icon)| {
                                let is_active = active_view == view
                                    && (view != View::Hosts || app.show_hosts_catalog);
                                let accent_color = app.accent_color();
                                let accent_fg = app.accent_fg();
                                let item_bg = if is_active { accent_color } else { sidebar_bg };
                                let item_hover = if is_active {
                                    accent_color
                                } else {
                                    app.muted_bg()
                                };
                                let item_text_color =
                                    if is_active { accent_fg } else { text_color };
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
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
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
                                        }),
                                    )
                            })),
                    )
                    // Bottom footer: Settings pinned to bottom
                    .child(div().pt_2().border_t_1().border_color(border_color).child({
                        let is_active = active_view == View::Settings;
                        let accent_color = app.accent_color();
                        let accent_fg = app.accent_fg();
                        let item_bg = if is_active { accent_color } else { sidebar_bg };
                        let item_hover = if is_active {
                            accent_color
                        } else {
                            app.muted_bg()
                        };
                        let item_text_color = if is_active { accent_fg } else { text_color };
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
                            .child(
                                svg()
                                    .data(crate::icons::SETTINGS_SVG)
                                    .size(px(16.0))
                                    .text_color(icon_color),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.show_new_tab_popover = false;
                                    this.active_view = View::Settings;
                                    this.show_hosts_catalog = false;
                                    cx.notify();
                                }),
                            )
                    })),
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
                            // Row: active view content + right-side sheet that pushes it aside
                            div()
                                .flex()
                                .flex_row()
                                .flex_1()
                                .w_full()
                                .min_h_0()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .child(content_pane),
                                )
                                .children(sheet_slot),
                        ),
                )
                // Backdrop when popover is open to dismiss on click outside
                .children(if app.show_new_tab_popover {
                    Some(div().absolute().inset_0().on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.show_new_tab_popover = false;
                            cx.notify();
                        }),
                    ))
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
        .children(passphrase_modal_overlay)
        .children(confirm_modals_overlay)
        .children(delete_forward_modal_overlay)
        .children(sftp_context_menu_overlay)
        .children(sftp_modal_overlay)
        .children(notification_toast)
        .into_any_element()
}

/// Pick the currently open (or exiting) sheet and wrap it in the shared
/// push-aside animation. Only one sheet is open at a time.
fn render_sheet_slot(app: &mut AppState, cx: &mut Context<AppState>) -> Option<AnyElement> {
    use crate::views::sheet::animated_sheet;

    if app.connection_modal.is_some() || app.connection_sheet_closing {
        let sheet = crate::views::connection_modal::render_connection_sheet(app, cx);
        return Some(animated_sheet(
            "connection-sheet-in",
            "connection-sheet-out",
            app.connection_sheet_closing,
            sheet,
        ));
    }
    if app.edit_key_modal.is_some() || app.edit_key_sheet_closing {
        let sheet = crate::views::keys::render_edit_key_sheet(app, cx);
        return Some(animated_sheet(
            "edit-key-sheet-in",
            "edit-key-sheet-out",
            app.edit_key_sheet_closing,
            sheet,
        ));
    }
    if app.show_add_key_modal || app.add_key_sheet_closing {
        let sheet = crate::views::keys::render_add_key_sheet(app, cx);
        return Some(animated_sheet(
            "add-key-sheet-in",
            "add-key-sheet-out",
            app.add_key_sheet_closing,
            sheet,
        ));
    }
    if app.forward_modal.is_some() || app.forward_sheet_closing {
        let sheet = crate::views::forwards::render_forward_sheet(app, cx);
        return Some(animated_sheet(
            "forward-sheet-in",
            "forward-sheet-out",
            app.forward_sheet_closing,
            sheet,
        ));
    }
    None
}

fn render_content_pane(
    app: &mut AppState,
    view: View,
    _is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let bg = app.bg_color();
    match view {
        View::NewTab => div()
            .size_full()
            .overflow_hidden()
            .bg(bg)
            .child(crate::views::new_tab::render_new_tab_page(app, cx))
            .into_any_element(),
        View::Hosts => {
            let active_tab_view = app
                .session_manager
                .active_tab()
                .and_then(|t| t.view.clone());
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
        View::Keys => div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(bg)
            .child(crate::views::keys::render_keys_view(app, cx))
            .into_any_element(),
        View::Forwards => div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(bg)
            .child(crate::views::forwards::render_forwards_view(app, cx))
            .into_any_element(),
        View::Sftp => div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(bg)
            .child(crate::views::sftp::render_sftp_view(app, cx))
            .into_any_element(),
        View::Settings => div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(bg)
            .child(crate::views::settings::render_settings_view(app, cx))
            .into_any_element(),
    }
}


/// Confirmation dialogs: close connected tab, delete connection, delete key
/// (with the affected-connections warning step from the web client).
pub fn render_confirm_modals(app: &mut AppState, cx: &mut Context<AppState>) -> Option<AnyElement> {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();
    let destructive = app.destructive_color();
    let _ = is_dark;

    if let Some(idx) = app.pending_close_tab {
        let title = app
            .session_manager
            .tabs()
            .get(idx)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "this host".to_string());
        return Some(
            confirm_dialog_shell(app, cx, |this, _, cx| {
                this.cancel_close_tab(cx);
            })
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_color)
                    .child("Disconnect from host"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(muted_text)
                    .child(format!("Disconnect from {}? The tab will be closed.", title)),
            )
            .child(
                confirm_dialog_buttons(
                    app,
                    cx,
                    "Cancel",
                    "Disconnect",
                    |this, cx| this.cancel_close_tab(cx),
                    move |this, _, cx| {
                        // Backdrop dismiss must not close the tab; use idx.
                        this.confirm_close_tab(idx, cx);
                    },
                ),
            )
            .into_any_element(),
        );
    }

    if let Some(conn) = app.delete_connection_target.clone() {
        return Some(
            confirm_dialog_shell(app, cx, |this, _, cx| {
                this.cancel_delete_connection_modal(cx);
            })
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_color)
                    .child("Delete Connection"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(muted_text)
                    .child(format!(
                        "Delete \"{}\"? This action cannot be undone.",
                        conn.label
                    )),
            )
            .child(confirm_dialog_buttons(
                app,
                cx,
                "Cancel",
                "Delete",
                |this, cx| this.cancel_delete_connection_modal(cx),
                move |this, _, cx| {
                    this.delete_connection(&conn.id, cx);
                    this.delete_connection_target = None;
                },
            ))
            .into_any_element(),
        );
    }

    if let Some(key) = app.delete_key_target.clone() {
        let (title, body) = match &app.key_delete_warning {
            Some((warning, count)) => (
                "Key still in use",
                format!(
                    "{}\n\nDeleting \"{}\" will remove key-based auth from {} connection(s). Are you sure you want to proceed?",
                    warning, key.name, count
                ),
            ),
            None => (
                "Delete SSH Key",
                format!("Are you sure you want to delete \"{}\"? This action cannot be undone.", key.name),
            ),
        };
        return Some(
            confirm_dialog_shell(app, cx, move |this, _, cx| {
                this.confirm_delete_key(cx);
            })
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(text_color)
                    .child(title),
            )
            .child(div().text_sm().text_color(muted_text).child(body))
            .child(confirm_dialog_buttons(
                app,
                cx,
                "Cancel",
                "Delete",
                |this, cx| this.cancel_delete_key_modal(cx),
                |this, _, cx| this.confirm_delete_key(cx),
            ))
            .into_any_element(),
        );
    }

    None
}

/// Shared centered dialog shell: dark backdrop, dismiss on backdrop click.
fn confirm_dialog_shell(
    app: &AppState,
    cx: &mut Context<AppState>,
    on_dismiss: impl Fn(&mut AppState, &mut Window, &mut Context<AppState>) + 'static,
) -> Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| on_dismiss(this, window, cx)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(420.0))
                .rounded_xl()
                .bg(app.card_bg())
                .border_1()
                .border_color(app.border_color())
                .shadow_lg()
                .p_6()
                .gap_3()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}),
        )
}

/// Right-aligned Cancel (neutral) + destructive action buttons.
fn confirm_dialog_buttons(
    app: &AppState,
    cx: &mut Context<AppState>,
    cancel_label: &'static str,
    action_label: &'static str,
    on_cancel: impl Fn(&mut AppState, &mut Context<AppState>) + 'static,
    on_action: impl Fn(&mut AppState, &mut Window, &mut Context<AppState>) + 'static,
) -> Div {
    let tag_bg = app.muted_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    div()
        .mt_2()
        .flex()
        .flex_row()
        .justify_end()
        .gap_2()
        .child(
            div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(tag_bg)
                .hover(move |s| s.bg(border_color))
                .cursor_pointer()
                .text_sm()
                .text_color(text_color)
                .child(cancel_label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| on_cancel(this, cx)),
                ),
        )
        .child(
            div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(app.destructive_color())
                .hover(|s| s.opacity(0.9))
                .cursor_pointer()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(0xffffff))
                .child(action_label)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, window, cx| on_action(this, window, cx)),
                ),
        )
}
