//! Navigation shell view: left sidebar and main content views.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::{AppState, View};

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

    div()
        .flex()
        .size_full()
        .bg(bg_color)
        .text_color(text_color)
        .font_family("JetBrains Mono")
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
                .p_8()
                .child(if active_view == View::Settings {
                    crate::views::settings::render_settings_view(app, cx)
                } else {
                    render_content_pane(active_view, is_dark)
                }),
        )
        .into_any_element()
}

fn render_content_pane(view: View, is_dark: bool) -> AnyElement {
    let (header, phase_note) = match view {
        View::Hosts => ("Hosts", "Host connection catalog — arrives in Phase 23"),
        View::Keys => ("SSH Keys", "Key vault and passphrases — arrives in Phase 23"),
        View::Sftp => ("SFTP", "SFTP Dual-Pane File Manager — arrives in Phase 25"),
        View::Settings => ("Settings", "Settings"),
    };

    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    div()
        .flex()
        .flex_col()
        .size_full()
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child(header),
        )
        .child(
            div()
                .mt_4()
                .p_6()
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
        )
        .into_any_element()
}
