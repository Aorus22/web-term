//! Navigation shell view: left sidebar and main content views.

use crate::actions::{
    CloseTab, EscapeOverlays, JumpTab1, JumpTab2, JumpTab3, JumpTab4, JumpTab5, JumpTab6, JumpTab7,
    JumpTab8, JumpTab9, NewTab, NextTab, PrevTab,
};
use crate::app_state::{AppState, View, FRAME_ROUNDING};
use crate::views::hosts::render_hosts_view;
use crate::views::reconnect_banner::render_reconnect_banner;
use crate::views::tab_strip::render_tab_strip;
use gpui::prelude::FluentBuilder as _;
use gpui::*;

/// Fixed sidebar width, matching the 200px used in `toggle_sidebar` layout math.
pub const SIDEBAR_WIDTH: f32 = 200.0;

/// Render the left navigation sidebar and active content pane.
///
/// `framed` enables CSD leaf rounding (Zed-style): the sidebar owns the left
/// corners, the tab strip the top-right, content the bottom-right. Skipped
/// when maximized/tiled so edges stay flush.
pub fn render_nav_shell(
    app: &mut AppState,
    framed: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
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
    let sidebar_bg = app.bg_color();
    let text_color = app.text_color();
    let _muted_text = app.muted_text();
    let border_color = app.border_color();

    let content_pane = render_content_pane(app, active_view, is_dark, cx);

    let sheet_slot = render_sheet_slot(app, framed, cx);

    let passphrase_modal_overlay = if app.pending_passphrase_conn.is_some() {
        Some(crate::views::passphrase_modal::render_passphrase_modal(
            app, cx,
        ))
    } else {
        None
    };

    let confirm_modals_overlay = render_confirm_modals(app, cx);
    let terminal_menu_overlay = if app.terminal_context_menu.is_some() {
        render_terminal_context_menu(app, cx)
    } else {
        None
    };
    let save_banner_overlay = render_save_connection_banner(app, cx);

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
                    .id("nav-01").hover(|s| s.text_color(text_color))
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
        // NOTE: no .bg here — the root must stay transparent so CSD rounded
        // corners (tab strip / sidebar / content leaves) show real
        // transparency instead of a square opaque backdrop.
        .text_color(text_color)
        // Tab shortcuts
        .on_action(cx.listener(|this, _: &NewTab, window, cx| {
            this.open_new_tab_page(window, cx);
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
        // Left Sidebar (collapsible): transparent shell + 40px spacer so the
        // floating tab strip's top-left corner stays transparent; bg lives on
        // the body below which owns only the bottom-left corner.
        // The slot's width animates (same pattern as right-side sheets) while
        // the inner panel keeps its full 200px and is clipped, so open/close
        // slides smoothly instead of snapping.
        .children(if app.sidebar_open || app.sidebar_closing {
            let closing = app.sidebar_closing;
            let animation = crate::views::sheet::sheet_animation();
            let (slot_id, width_fn): (ElementId, fn(f32) -> f32) = if closing {
                (
                    ElementId::Name("sidebar-out".into()),
                    |delta| SIDEBAR_WIDTH * (1.0 - delta).clamp(0.0, 1.0),
                )
            } else {
                (
                    ElementId::Name("sidebar-in".into()),
                    |delta| SIDEBAR_WIDTH * delta.clamp(0.0, 1.0),
                )
            };
            Some(
                div()
                    .h_full()
                    .flex_shrink_0()
                    .w(px(SIDEBAR_WIDTH))
                    .overflow_hidden()
                    .with_animation(slot_id, animation, move |el, delta| {
                        el.w(px(width_fn(delta)))
                    })
                    .child(
                        div()
                    .flex()
                    .flex_col()
                    .w(px(SIDEBAR_WIDTH))
                    .min_w(px(SIDEBAR_WIDTH))
                    .h_full()
                    .flex_shrink_0()
                    .child(div().h(px(40.0)).flex_none())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .bg(sidebar_bg)
                            .border_r_1()
                            .border_color(border_color)
                            .px_3()
                            .py_4()
                            .justify_between()
                            .when(framed, |d| d.rounded_bl(FRAME_ROUNDING))
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
                                    .id("nav-02").hover(move |s| s.bg(item_hover))
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
                                            // Sheets belong to their own page; navigating
                                            // away unmounts them (web client parity).
                                            this.close_all_sheets_instant();
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
                            .id("nav-03").hover(move |s| s.bg(item_hover))
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
                                    this.close_all_sheets_instant();
                                    this.active_view = View::Settings;
                                    this.show_hosts_catalog = false;
                                    cx.notify();
                                }),
                            )
                    })),
                    ),
                ),
            )
        } else {
            None
        })
        // Main Content Area: transparent shell + 40px spacer (tab strip floats
        // above it) so rounded corners stay transparent; bg lives on the body.
        // Tree: shell > spacer, body > pane > row; tab strip + popover
        // backdrop attach to the shell AFTER the body (paint on top).
        .child({
            let mut shell = div()
                .relative()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .flex()
                .flex_col()
                .h_full()
                .child(div().h(px(40.0)).flex_none());
            let mut body = div()
                .relative()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .w_full()
                .overflow_hidden()
                // Own bg + gutters on all sides: views paint full-bleed and
                // would cover the rounded corners, so the body provides the
                // rounded backdrop and insets content from the corners. The
                // left/top insets also keep the terminal grid out of the
                // window-edge resize zones and the floating tab strip's
                // hitbox, so selecting text near the top-left corner works.
                .bg(sidebar_bg)
                .p(FRAME_ROUNDING);
            if framed {
                body = body.rounded_br(FRAME_ROUNDING);
                if !app.sidebar_open && !app.sidebar_closing {
                    body = body.rounded_bl(FRAME_ROUNDING);
                }
            }
            let pane = div()
                .flex()
                .flex_col()
                .size_full()
                .min_w_0()
                .overflow_hidden()
                .children(if active_view == View::Hosts && !app.show_hosts_catalog {
                    render_reconnect_banner(app, cx)
                } else {
                    None
                })
                .child(
                    // Row: active view content + right-side sheet that pushes it aside.
                    // min_w_0 lets the fixed-width sheet squeeze the content
                    // instead of pushing the whole window wider than the viewport.
                    div()
                        .flex()
                        .flex_row()
                        .flex_1()
                        .w_full()
                        .min_w_0()
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
                );
            body = body.child(pane);
            shell = shell.child(body);
            // Backdrop when popover is open to dismiss on click outside
            if app.show_new_tab_popover {
                shell = shell.child(div().absolute().inset_0().on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        this.show_new_tab_popover = false;
                        cx.notify();
                    }),
                ));
            }
            shell
        })
        // Top Tab Strip: full window width (including above the sidebar),
        // floating above the transparent spacers. Owns both top corners.
        // Painted after content so it and its popovers stay on top!
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(40.0))
                .child(render_tab_strip(app, framed, cx)),
        )
        .children(passphrase_modal_overlay)
        .children(confirm_modals_overlay)
        .children(save_banner_overlay)
        .children(terminal_menu_overlay)
        .children(delete_forward_modal_overlay)
        .children(sftp_context_menu_overlay)
        .children(sftp_modal_overlay)
        .children(notification_toast)
        .into_any_element()
}

/// Pick the currently open (or exiting) sheet and wrap it in the shared
/// push-aside animation. Only one sheet is open at a time.
fn render_sheet_slot(app: &mut AppState, framed: bool, cx: &mut Context<AppState>) -> Option<AnyElement> {
    use crate::views::sheet::animated_sheet;

    if app.connection_modal.is_some() || app.connection_sheet_closing {
        let sheet = crate::views::connection_modal::render_connection_sheet(app, cx);
        return Some(animated_sheet(
            "connection-sheet-in",
            "connection-sheet-out",
            app.connection_sheet_closing,
            framed,
            sheet,
        ));
    }
    if app.edit_key_modal.is_some() || app.edit_key_sheet_closing {
        let sheet = crate::views::keys::render_edit_key_sheet(app, cx);
        return Some(animated_sheet(
            "edit-key-sheet-in",
            "edit-key-sheet-out",
            app.edit_key_sheet_closing,
            framed,
            sheet,
        ));
    }
    if app.show_add_key_modal || app.add_key_sheet_closing {
        let sheet = crate::views::keys::render_add_key_sheet(app, cx);
        return Some(animated_sheet(
            "add-key-sheet-in",
            "add-key-sheet-out",
            app.add_key_sheet_closing,
            framed,
            sheet,
        ));
    }
    if app.forward_modal.is_some() || app.forward_sheet_closing {
        let sheet = crate::views::forwards::render_forward_sheet(app, cx);
        return Some(animated_sheet(
            "forward-sheet-in",
            "forward-sheet-out",
            app.forward_sheet_closing,
            framed,
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
                div()
                    .size_full()
                    .min_w_0()
                    .overflow_hidden()
                    .bg(bg)
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, ev: &MouseDownEvent, _window, cx| {
                            this.terminal_context_menu =
                                Some((f32::from(ev.position.x), f32::from(ev.position.y)));
                            cx.notify();
                        }),
                    )
                    .child(term_view)
                    .into_any_element()
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
    let text_color = app.text_color();
    let muted_text = app.muted_text();

    if let Some(idx) = app.pending_close_tab {
        let title = app
            .session_manager
            .tabs()
            .get(idx)
            .map(|t| t.title.clone())
            .unwrap_or_else(|| "this host".to_string());
        let disconnect_buttons = confirm_dialog_buttons(
            app,
            cx,
            "Cancel",
            "Disconnect",
            |this, cx| this.cancel_close_tab(cx),
            move |this, _, cx| {
                // Backdrop dismiss must not close the tab; use idx.
                this.confirm_close_tab(idx, cx);
            },
        )
        .into_any_element();
        return Some(
            confirm_dialog_shell(
                app,
                cx,
                |this, _, cx| {
                    this.cancel_close_tab(cx);
                },
                vec![
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_color)
                        .child("Disconnect from host")
                        .into_any_element(),
                    div().text_sm().text_color(muted_text).child(format!(
                        "Disconnect from {}? The tab will be closed.",
                        title
                    )).into_any_element(),
                    disconnect_buttons,
                ],
            )
            .into_any_element(),
        );
    }

    if let Some(conn) = app.delete_connection_target.clone() {
        let delete_conn_buttons = confirm_dialog_buttons(
            app,
            cx,
            "Cancel",
            "Delete",
            |this, cx| this.cancel_delete_connection_modal(cx),
            move |this, _, cx| {
                this.delete_connection(&conn.id, cx);
                this.delete_connection_target = None;
            },
        )
        .into_any_element();
        return Some(
            confirm_dialog_shell(
                app,
                cx,
                |this, _, cx| {
                    this.cancel_delete_connection_modal(cx);
                },
                vec![
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_color)
                        .child("Delete Connection")
                        .into_any_element(),
                    div().text_sm().text_color(muted_text).child(format!(
                        "Delete \"{}\"? This action cannot be undone.",
                        conn.label
                    )).into_any_element(),
                    delete_conn_buttons,
                ],
            )
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
        let delete_key_buttons = confirm_dialog_buttons(
            app,
            cx,
            "Cancel",
            "Delete",
            |this, cx| this.cancel_delete_key_modal(cx),
            |this, _, cx| this.confirm_delete_key(cx),
        )
        .into_any_element();
        return Some(
            confirm_dialog_shell(
                app,
                cx,
                |this, _, cx| {
                    this.cancel_delete_key_modal(cx);
                },
                vec![
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_color)
                        .child(title)
                        .into_any_element(),
                    div().text_sm().text_color(muted_text).child(body).into_any_element(),
                    delete_key_buttons,
                ],
            )
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
    content: Vec<AnyElement>,
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
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                .children(content),
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
                .id("nav-04").hover(move |s| s.bg(border_color))
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
                .id("nav-05").hover(|s| s.opacity(0.9))
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

/// Right-click context menu for the terminal pane: clipboard, clear, and
/// control-key sender, matching the web TerminalContextMenu.
pub fn render_terminal_context_menu(
    app: &mut AppState,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    let (x, y) = app.terminal_context_menu?;
    let is_dark = app.is_dark();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let hover_bg = app.muted_bg();

    if app
        .session_manager
        .active_tab()
        .and_then(|t| t.view.clone())
        .is_none()
    {
        app.terminal_context_menu = None;
        return None;
    }

    let border_color = app.border_color();
    let card_bg = app.card_bg();

    let menu = div()
        .w(px(190.0))
        .rounded_md()
        .border_1()
        .border_color(border_color)
        .bg(card_bg)
        .shadow_lg()
        .py_1()
        .on_mouse_down(MouseButton::Left, |_, _, _| {})
        .child(context_menu_row(
            "Copy",
            crate::icons::COPY_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, cx| {
                        v.copy_selection(cx);
                    });
                }
            }),
        ))
        .child(context_menu_row(
            "Paste",
            crate::icons::FILES_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, cx| {
                        v.paste_clipboard(cx);
                    });
                }
            }),
        ))
        .child(div().h_px().w_full().bg(border_color).my_0p5())
        .child(context_menu_row(
            "Clear",
            crate::icons::TRASH_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, _cx| {
                        // Clear screen + scrollback (CSI 3J / CSI H / CSI 2J).
                        v.write_to_pty(b"\x1b[3J\x1b[H\x1b[2J");
                    });
                }
            }),
        ))
        .child(div().h_px().w_full().bg(border_color).my_0p5())
        .child(context_menu_row(
            "Send Ctrl+C (Interrupt)",
            crate::icons::TERMINAL_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, _cx| {
                        v.write_to_pty(b"\x03");
                    });
                }
            }),
        ))
        .child(context_menu_row(
            "Send Ctrl+Z (Suspend)",
            crate::icons::TERMINAL_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, _cx| {
                        v.write_to_pty(b"\x1a");
                    });
                }
            }),
        ))
        .child(context_menu_row(
            "Send Ctrl+D (EOF)",
            crate::icons::TERMINAL_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, _cx| {
                        v.write_to_pty(b"\x04");
                    });
                }
            }),
        ))
        .child(context_menu_row(
            "Send Ctrl+L (Clear line)",
            crate::icons::TERMINAL_SVG,
            is_dark,
            text_color,
            muted_text,
            hover_bg,
            cx.listener(|this, _, _window, cx| {
                this.terminal_context_menu = None;
                if let Some(view) = this
                    .session_manager
                    .active_tab()
                    .and_then(|t| t.view.clone())
                {
                    view.update(cx, |v, _cx| {
                        v.write_to_pty(b"\x0c");
                    });
                }
            }),
        ));

    let dismiss = cx.listener(|this, _, _window, cx| {
        this.terminal_context_menu = None;
        cx.notify();
    });
    Some(
        div()
            .absolute()
            .inset_0()
            .on_mouse_down(MouseButton::Left, dismiss)
            .child(div().absolute().left(px(x)).top(px(y)).child(menu))
            .into_any_element(),
    )
}

fn context_menu_row(
    label: &'static str,
    icon: &'static [u8],
    is_dark: bool,
    text_color: Rgba,
    muted_text: Rgba,
    hover_bg: Rgba,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let _ = is_dark;
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_3()
        .py_1p5()
        .rounded_md()
        .cursor_pointer()
        .id("nav-06").hover(move |s| s.bg(hover_bg))
        .text_xs()
        .text_color(text_color)
        .child(svg().data(icon).size(px(12.0)).text_color(muted_text))
        .child(label)
        .on_mouse_down(MouseButton::Left, on_click)
}

/// "Save this connection?" banner shown for disconnected quick-connect tabs.
pub fn render_save_connection_banner(
    app: &mut AppState,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    if app.save_connection_prompt.is_none() {
        return None;
    }
    let disconnected = app
        .session_manager
        .active_tab()
        .map(|t| matches!(t.status, crate::session::SessionStatus::Disconnected(_)))
        .unwrap_or(false);
    if !disconnected {
        return None;
    }
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    Some(
        div()
            .absolute()
            .top(px(48.0))
            .left(px(16.0))
            .right(px(16.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .rounded_lg()
            .bg(app.card_bg())
            .border_1()
            .border_color(app.primary_color())
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(text_color)
                            .child("Save this connection?"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted_text)
                            .child("Save the host so you can reconnect quickly later."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(primary_color)
                            .id("nav-07").hover(|s| s.opacity(0.9))
                            .cursor_pointer()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(primary_fg)
                            .child("Save")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.save_quick_connection(cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .text_xs()
                            .text_color(muted_text)
                            .cursor_pointer()
                            .id("nav-08").hover(move |s| s.text_color(text_color))
                            .child("Dismiss")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.dismiss_save_connection_prompt(cx);
                                }),
                            ),
                    ),
            )
            .into_any_element(),
    )
}
