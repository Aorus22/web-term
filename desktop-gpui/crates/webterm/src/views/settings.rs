//! Desktop Settings view surfacing theme selection, backend path override, font sizing, and fixed Alacritty engine status.
//!
//! Conforms to requirement SET-01:
//! - Configurable appearance themes (Dark, Light, System) syncing live mid-session across app & terminal palette.
//! - Desktop preferences including backend binary path override.
//! - Fixed Alacritty terminal engine notice with NO engine selector.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;

/// Render the comprehensive desktop Settings page.
pub fn render_settings_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.accent_color();
    let primary_color = app.primary_color();
    let current_theme = app.theme;
    let active_preset_id = app.settings.theme_preset.clone();

    let backend_path_str = app
        .spawn_opts
        .as_ref()
        .map(|o| o.backend_path.to_string_lossy().to_string())
        .unwrap_or_else(|| "bundled executable".to_string());

    let backend_status_label = match app.backend_status {
        webterm_supervisor::BackendStatus::Ready => "Running (local supervisor)",
        webterm_supervisor::BackendStatus::Starting => "Starting",
        webterm_supervisor::BackendStatus::Failed { .. } => "Failed",
        webterm_supervisor::BackendStatus::Crashed { .. } => "Crashed",
    };

    let has_override = app.settings.backend_path.is_some();
    let override_display = app
        .settings
        .backend_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "None (using bundled backend)".to_string());

    let current_font_size = app.terminal_font_size;
    let font_sizes = [12.0f32, 14.0f32, 16.0f32, 18.0f32, 20.0f32];

    div()
        .id("settings-scroll-area")
        .flex()
        .flex_col()
        .size_full()
        .overflow_y_scroll()
        .p_6()
        .child(
            div()
                .text_2xl()
                .font_weight(FontWeight::BOLD)
                .text_color(text_color)
                .child("Desktop Settings"),
        )
        .child(
            div()
                .mt_4()
                .flex()
                .flex_col()
                .gap_4()
                .w(px(760.0))
                // 1. Appearance Theme Section
                .child(
                    div()
                        .p_6()
                        .rounded_xl()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(text_color)
                                        .child("Appearance Theme"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(tag_bg)
                                        .text_xs()
                                        .text_color(muted_text)
                                        .child("Live mid-session sync"),
                                ),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Theme changes apply immediately to the GPUI interface and synchronize the terminal color palette in real-time across all open sessions."),
                        )
                        .child(
                            div()
                                .mt_4()
                                .flex()
                                .flex_row()
                                .gap_3()
                                // Dark Theme Option
                                .child(
                                    div()
                                        .flex_1()
                                        .px_4()
                                        .py_3()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(if current_theme == SettingsTheme::Dark {
                                            primary_color
                                        } else {
                                            border_color
                                        })
                                        .bg(if current_theme == SettingsTheme::Dark {
                                            tag_bg
                                        } else {
                                            card_bg
                                        })
                                        .cursor_pointer()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::MOON_SVG)
                                                                .size(px(16.0))
                                                                .text_color(if current_theme == SettingsTheme::Dark {
                                                                    primary_color
                                                                } else {
                                                                    muted_text
                                                                }),
                                                        )
                                                        .child("Dark Theme"),
                                                )
                                                .child(if current_theme == SettingsTheme::Dark {
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_1()
                                                        .text_xs()
                                                        .text_color(primary_color)
                                                        .child(svg().data(crate::icons::CHECK_SVG).size(px(12.0)).text_color(primary_color))
                                                        .child("Active")
                                                } else {
                                                    div()
                                                }),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.set_theme(SettingsTheme::Dark, cx);
                                        })),
                                // Light Theme Option
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .px_4()
                                        .py_3()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(if current_theme == SettingsTheme::Light {
                                            primary_color
                                        } else {
                                            border_color
                                        })
                                        .bg(if current_theme == SettingsTheme::Light {
                                            tag_bg
                                        } else {
                                            card_bg
                                        })
                                        .cursor_pointer()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::SUN_SVG)
                                                                .size(px(16.0))
                                                                .text_color(if current_theme == SettingsTheme::Light {
                                                                    primary_color
                                                                } else {
                                                                    muted_text
                                                                }),
                                                        )
                                                        .child("Light Theme"),
                                                )
                                                .child(if current_theme == SettingsTheme::Light {
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_1()
                                                        .text_xs()
                                                        .text_color(primary_color)
                                                        .child(svg().data(crate::icons::CHECK_SVG).size(px(12.0)).text_color(primary_color))
                                                        .child("Active")
                                                } else {
                                                    div()
                                                }),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.set_theme(SettingsTheme::Light, cx);
                                        })),
                                )
                                // System Theme Option
                                .child(
                                    div()
                                        .flex_1()
                                        .px_4()
                                        .py_3()
                                        .rounded_lg()
                                        .border_1()
                                        .border_color(if current_theme == SettingsTheme::System {
                                            primary_color
                                        } else {
                                            border_color
                                        })
                                        .bg(if current_theme == SettingsTheme::System {
                                            tag_bg
                                        } else {
                                            card_bg
                                        })
                                        .cursor_pointer()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child(
                                                            svg()
                                                                .data(crate::icons::MONITOR_SVG)
                                                                .size(px(16.0))
                                                                .text_color(if current_theme == SettingsTheme::System {
                                                                    primary_color
                                                                } else {
                                                                    muted_text
                                                                }),
                                                        )
                                                        .child("System"),
                                                )
                                                .child(if current_theme == SettingsTheme::System {
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_1()
                                                        .text_xs()
                                                        .text_color(primary_color)
                                                        .child(svg().data(crate::icons::CHECK_SVG).size(px(12.0)).text_color(primary_color))
                                                        .child("Active")
                                                } else {
                                                    div()
                                                }),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.set_theme(SettingsTheme::System, cx);
                                        })),
                                ),
                        )
                        // Theme Presets Grid (Parity with Web Client)
                        .child(
                            div()
                                .mt_5()
                                .pt_4()
                                .border_t_1()
                                .border_color(border_color)
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(text_color)
                                                .child("Color Presets"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(muted_text)
                                                .child("38 Presets from Web Client"),
                                        ),
                                )
                                .child(
                                    div()
                                        .mt_3()
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .gap_2p5()
                                        .children(crate::theme::THEME_PRESETS.iter().map(|preset| {
                                            let is_active = preset.id == active_preset_id;
                                            let preset_id = preset.id;
                                            let p_bg = rgb(preset.background);
                                            let p_primary = rgb(preset.primary);
                                            let p_accent = rgb(preset.accent);
                                            let p_destructive = rgb(preset.destructive);
                                            let p_fg = rgb(preset.foreground);
                                            let p_muted_fg = rgb(preset.muted_foreground);

                                            div()
                                                .w(px(226.0))
                                                .p_2()
                                                .rounded_lg()
                                                .border_2()
                                                .border_color(if is_active {
                                                    p_primary
                                                } else {
                                                    border_color
                                                })
                                                .bg(if is_active {
                                                    app.bg_color()
                                                } else {
                                                    card_bg
                                                })
                                                .cursor_pointer()
                                                .hover(|s| s.border_color(rgb(0x71717a)))
                                                // Mini preview swatch box
                                                .child(
                                                    div()
                                                        .h(px(48.0))
                                                        .w_full()
                                                        .rounded_md()
                                                        .p_2()
                                                        .flex()
                                                        .flex_col()
                                                        .justify_between()
                                                        .bg(p_bg)
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .gap_1p5()
                                                                .child(div().size(px(7.0)).rounded_full().bg(p_primary))
                                                                .child(div().size(px(7.0)).rounded_full().bg(p_accent))
                                                                .child(div().size(px(7.0)).rounded_full().bg(p_destructive)),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .gap_1()
                                                                .child(div().w(px(60.0)).h(px(3.0)).rounded_sm().bg(p_fg))
                                                                .child(div().w(px(30.0)).h(px(3.0)).rounded_sm().bg(p_muted_fg)),
                                                        ),
                                                )
                                                // Theme Label & Active Checkmark
                                                .child(
                                                    div()
                                                        .mt_1p5()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .justify_between()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(if is_active { text_color } else { muted_text })
                                                                .child(preset.label),
                                                        )
                                                        .children(if is_active {
                                                            Some(
                                                                svg()
                                                                    .data(crate::icons::CHECK_SVG)
                                                                    .size(px(13.0))
                                                                    .text_color(p_primary),
                                                            )
                                                        } else {
                                                            None
                                                        }),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.set_theme_preset(preset_id, cx);
                                                }))
                                        })),
                                ),
                        ),
                )
                // 2. Local Backend Service & Binary Override Section
                .child(
                    div()
                        .p_6()
                        .rounded_xl()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(text_color)
                                        .child("Backend Service & Path Override"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(if app.backend_status == webterm_supervisor::BackendStatus::Ready {
                                            if is_dark { rgb(0x14532d) } else { rgb(0xdcfce7) }
                                        } else {
                                            if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) }
                                        })
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(if app.backend_status == webterm_supervisor::BackendStatus::Ready {
                                            if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) }
                                        } else {
                                            rgb(0xef4444)
                                        })
                                        .child(backend_status_label),
                                ),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_xs()
                                .text_color(muted_text)
                                .child("The desktop client runs an internal Go backend on loopback for SSH/SFTP and local PTY management. You may specify a custom executable path override for development."),
                        )
                        .child(
                            div()
                                .mt_3()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .text_xs()
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child(div().text_color(muted_text).child("Active Binary Path:"))
                                        .child(
                                            div()
                                                .font_family("JetBrains Mono")
                                                .text_color(text_color)
                                                .child(backend_path_str),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .justify_between()
                                        .child(div().text_color(muted_text).child("Configured Override:"))
                                        .child(
                                            div()
                                                .font_family("JetBrains Mono")
                                                .text_color(if has_override {
                                                    primary_color
                                                } else {
                                                    muted_text
                                                })
                                                .child(override_display),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .mt_4()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(if has_override {
                                            if is_dark { rgb(0x7f1d1d) } else { rgb(0xfecaca) }
                                        } else {
                                            tag_bg
                                        })
                                        .text_color(if has_override { rgb(0xef4444) } else { muted_text })
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("Reset to Bundled Backend")
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                            this.reset_backend_path_override(cx);
                                        })),
                                ),
                        ),
                )
                // 3. Terminal Engine Information Card (Strictly No Engine Selector per SET-01)
                .child(
                    div()
                        .p_6()
                        .rounded_xl()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(text_color)
                                        .child("Terminal Engine: Alacritty (Fixed)"),
                                )
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(if is_dark { rgb(0x14532d) } else { rgb(0xdcfce7) })
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(if is_dark { rgb(0x4ade80) } else { rgb(0x16a34a) })
                                        .child("Native In-Tree"),
                                ),
                        )
                        .child(
                            div()
                                .mt_2()
                                .text_xs()
                                .text_color(muted_text)
                                .child("The desktop client is permanently powered by a native Rust Alacritty terminal engine (alacritty_terminal). It provides Zed-proven GPU/CPU quad rendering, full VT100/xterm escape parsing, and seamless host clipboard copy/paste with zero browser overhead."),
                        )
                        .child(
                            div()
                                .mt_2()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .text_xs()
                                .text_color(primary_color)
                                .child(svg().data(crate::icons::INFO_SVG).size(px(14.0)).text_color(primary_color))
                                .child("Requirement SET-01 enforced: Browser-based engines (wterm / xterm.js) are excluded from the desktop application."),
                        ),
                )
                // 4. Terminal Typography & Sizing Section
                .child(
                    div()
                        .p_6()
                        .rounded_xl()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_color)
                                .child("Terminal Font & Typography"),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Monospace rendering uses bundled JetBrains Mono with automatic glyph metrics and cell layout."),
                        )
                        .child(
                            div()
                                .mt_3()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(div().text_xs().text_color(muted_text).child("Font Size:"))
                                .children(font_sizes.iter().map(|&sz| {
                                    let is_active = (current_font_size - sz).abs() < 0.1;
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .border_1()
                                        .border_color(if is_active {
                                            primary_color
                                        } else {
                                            border_color
                                        })
                                        .bg(if is_active {
                                            tag_bg
                                        } else {
                                            app.bg_color()
                                        })
                                        .text_xs()
                                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_active { primary_color } else { text_color })
                                        .child(format!("{sz:.0}px"))
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.set_terminal_font_size(sz, cx);
                                        }))
                                })),
                        ),
                ),
        )
        .into_any_element()
}
