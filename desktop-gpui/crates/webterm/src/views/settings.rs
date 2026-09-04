//! Minimal Settings view surfacing theme preferences and local backend status.
//!
//! Note: Full desktop settings and engine selection arrive in Phase 26.
//! This view displays existing configuration without exposing port numbers.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;

/// Render the minimal Settings page.
pub fn render_settings_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let current_theme = app.theme;
    let is_dark = current_theme != SettingsTheme::Light;

    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    let backend_path_str = app
        .spawn_opts
        .as_ref()
        .map(|o| o.backend_path.to_string_lossy().to_string())
        .unwrap_or_else(|| "bundled executable".to_string());

    let backend_status_label = match app.backend_status {
        webterm_supervisor::BackendStatus::Ready => "Running (local)",
        webterm_supervisor::BackendStatus::Starting => "Starting",
        webterm_supervisor::BackendStatus::Failed { .. } => "Failed",
        webterm_supervisor::BackendStatus::Crashed { .. } => "Crashed",
    };

    div()
        .flex()
        .flex_col()
        .size_full()
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .child("Settings"),
        )
        .child(
            div()
                .mt_4()
                .flex()
                .flex_col()
                .gap_4()
                .w(px(700.0))
                // Theme section
                .child(
                    div()
                        .p_6()
                        .rounded_lg()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Appearance Theme"),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Select application color scheme. Syncs across navigation sidebar and views."),
                        )
                        .child(
                            div()
                                .mt_4()
                                .flex()
                                .gap_3()
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if is_dark { rgb(0x3b82f6) } else { rgb(0xe2e8f0) })
                                        .text_color(if is_dark { rgb(0xffffff) } else { rgb(0x0f172a) })
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Dark Theme")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            if this.theme != SettingsTheme::Dark {
                                                this.toggle_theme(cx);
                                            }
                                        })),
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(if !is_dark { rgb(0x3b82f6) } else { rgb(0x3f3f46) })
                                        .text_color(if !is_dark { rgb(0xffffff) } else { rgb(0xf4f4f5) })
                                        .cursor_pointer()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Light Theme")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            if this.theme == SettingsTheme::Dark {
                                                this.toggle_theme(cx);
                                            }
                                        })),
                                ),
                        ),
                )
                // Backend Status Section (Host-only, port never displayed per Prohibition 3)
                .child(
                    div()
                        .p_6()
                        .rounded_lg()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Backend Service"),
                        )
                        .child(
                            div()
                                .mt_3()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .text_sm()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child(div().text_color(muted_text).child("Process Status:"))
                                        .child(
                                            div()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if app.backend_status == webterm_supervisor::BackendStatus::Ready {
                                                    rgb(0x22c55e)
                                                } else {
                                                    rgb(0xef4444)
                                                })
                                                .child(backend_status_label),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child(div().text_color(muted_text).child("Executable Path:"))
                                        .child(
                                            div()
                                                .font_family("JetBrains Mono")
                                                .text_xs()
                                                .child(backend_path_str),
                                        ),
                                ),
                        ),
                )
                // Phase 26 Note
                .child(
                    div()
                        .p_4()
                        .rounded_lg()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Comprehensive application configuration, terminal engine selector (wterm/xterm.js), and font customizations will arrive in Phase 26."),
                        ),
                ),
        )
        .into_any_element()
}
