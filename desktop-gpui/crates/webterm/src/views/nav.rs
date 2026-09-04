//! Navigation shell view: left sidebar and main content views.

use gpui::*;
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

    // Sidebar items
    let items = [
        (View::Hosts, "Hosts"),
        (View::Keys, "SSH Keys"),
        (View::Sftp, "SFTP"),
        (View::Settings, "Settings"),
    ];

    let is_dark = current_theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let sidebar_bg = if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    let show_modal = app.show_new_tab_modal;
    let content_pane = if active_view == View::Settings {
        crate::views::settings::render_settings_view(app, cx)
    } else {
        render_content_pane(app, active_view, is_dark, cx)
    };

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
        .font_family("JetBrains Mono")
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
                .w(px(240.0))
                .h_full()
                .bg(sidebar_bg)
                .border_r_1()
                .border_color(border_color)
                .p_4()
                .justify_between()
                // Top header & items
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .pb_4()
                                .child("WebTerm Desktop"),
                        )
                        .children(items.into_iter().map(|(view, label)| {
                            let is_active = active_view == view;
                            let item_bg = if is_active {
                                if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) }
                            } else {
                                sidebar_bg
                            };

                            div()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(item_bg)
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) }))
                                .cursor_pointer()
                                .text_sm()
                                .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .child(label)
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                    this.active_view = view;
                                    if view == View::Hosts {
                                        this.show_hosts_catalog = true;
                                    }
                                    cx.notify();
                                }))
                        })),
                )
                // Bottom footer: Theme Toggle
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .pt_4()
                        .border_t_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) })
                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_sm()
                                .child(if is_dark { "Theme: Dark 🌙" } else { "Theme: Light ☀️" })
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
                .p_6()
                .child(content_pane),
        )
        .children(modal_overlay)
        .children(conn_modal_overlay)
        .children(import_modal_overlay)
        .children(add_key_modal_overlay)
        .children(passphrase_modal_overlay)
        .children(notification_toast)
        .into_any_element()
}

fn render_content_pane(
    app: &mut AppState,
    view: View,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let (header, phase_note) = match view {
        View::Hosts => ("Hosts & Terminal", "Host connection catalog — arrives in Phase 23"),
        View::Keys => ("SSH Keys", "Key vault and passphrases — arrives in Phase 23"),
        View::Sftp => ("SFTP", "SFTP Dual-Pane File Manager — arrives in Phase 25"),
        View::Settings => ("Settings", "Settings"),
    };

    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    let mut content = div()
        .flex()
        .flex_col()
        .size_full()
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child(header),
        );

    if view == View::Hosts {
        let active_tab_view = app.session_manager.active_tab().and_then(|t| t.view.clone());

        content = content.child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .mt_4()
                .rounded_lg()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
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
                }),
        );
    } else if view == View::Keys {
        content = content.child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .mt_4()
                .rounded_lg()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .overflow_hidden()
                .child(crate::views::keys::render_keys_view(app, cx)),
        );
    } else {
        content = content.child(
            div()
                .mt_4()
                .p_4()
                .rounded_lg()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .child(
                    div()
                        .text_sm()
                        .text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) })
                        .child(phase_note),
                ),
        );
    }

    content.into_any_element()
}
