//! Settings view matching Electron SettingsPage 1:1.
//!
//! Features:
//! - Centered max-w-2xl layout with "Settings" title.
//! - APPEARANCE section:
//!   - Theme Mode filter dropdown ("All themes", "Dark", "Light") with auto-matching.
//!   - Color Theme cards grid (3 columns) with live preview box (3 dots, 2 bars), active checkmark.
//! - TERMINAL section (omitting web-only terminal type & engine):
//!   - Font pill button opening Terminal Font dialog.
//!   - Cursor Style dropdown ("block", "underline", "bar").
//!   - Cursor Blink pulsing dot indicator and switch toggle.
//!   - Scrollback Buffer dropdown ("1,000 lines", "5,000 lines", etc.).
//! - Terminal Font modal with family selector, size stepper/presets, and terminal preview.

use crate::app_state::AppState;
use gpui::*;
use gpui_component::input::Input;
use gpui_component::Sizable;

const MONO_FONTS: &[&str] = &[
    "Geist Mono",
    "JetBrains Mono",
    "Fira Code",
    "Source Code Pro",
    "IBM Plex Mono",
    "Cascadia Code",
    "Inconsolata",
    "Ubuntu Mono",
    "Menlo",
    "Consolas",
    "Monaco",
    "monospace",
];

/// Render the settings page matching Electron 1:1.
pub fn render_settings_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let accent_color = app.accent_color();
    let accent_fg = app.accent_fg();
    let secondary_bg = app.secondary_bg();
    let active_preset_id = app.settings.theme_preset.clone();
    let theme_mode = app.theme_mode_filter.clone();

    // Filter presets according to theme mode
    let filtered_presets: Vec<&'static crate::theme::ThemePreset> = crate::theme::THEME_PRESETS
        .iter()
        .filter(|p| match theme_mode.as_str() {
            "dark" => p.is_dark,
            "light" => !p.is_dark,
            _ => true,
        })
        .collect();

    let show_theme_picker = app.show_theme_mode_picker;
    let show_cursor_picker = app.show_cursor_style_picker;
    let show_scrollback_picker = app.show_scrollback_picker;
    let show_font_dialog = app.show_font_dialog;

    let theme_mode_label = match theme_mode.as_str() {
        "dark" => "Dark",
        "light" => "Light",
        _ => "All themes",
    };

    let cursor_style_val = app.cursor_style.clone();
    let cursor_style_label = match cursor_style_val.as_str() {
        "underline" => "Underline (_)",
        "bar" => "Bar (|)",
        _ => "Block (▮)",
    };

    let scrollback_val = app.scrollback;
    let scrollback_label = match scrollback_val {
        5000 => "5,000 lines",
        10000 => "10,000 lines",
        50000 => "50,000 lines",
        0 => "Unlimited",
        _ => "1,000 lines",
    };

    let cursor_blink_active = app.cursor_blink;
    let font_display = format!(
        "{} {:.0}px",
        app.terminal_font_family, app.terminal_font_size
    );

    div()
        .id("settings-scroll-area")
        .flex()
        .flex_col()
        .items_center()
        .justify_start()
        .size_full()
        .overflow_y_scroll()
        .pt(px(48.0))
        .pb(px(48.0))
        .px_4()
        .child(
            div()
                .w_full()
                .max_w(px(672.0))
                .flex()
                .flex_col()
                .gap(px(32.0))
                .pb_12()
                // Page Heading: Settings
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(text_color)
                        .child("Settings"),
                )
                // ====================================================
                // 1. APPEARANCE SECTION
                // ====================================================
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(muted_text)
                                .child("APPEARANCE"),
                        )
                        .child(
                            div()
                                .rounded_lg()
                                .border_1()
                                .border_color(border_color)
                                .bg(card_bg)
                                .overflow_hidden()
                                // Row 1: Theme Mode Filter
                                .child(
                                    div()
                                        .relative()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_3()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::PAINTBRUSH_SVG)
                                                        .size(px(16.0))
                                                        .text_color(muted_text),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(text_color)
                                                                .child("Theme Mode"),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(muted_text)
                                                                .child("Filter by dark or light appearance"),
                                                        ),
                                                ),
                                        )
                                        // Dropdown Selector Button
                                        .child(
                                            div()
                                                .w(px(110.0))
                                                .h(px(32.0))
                                                .px_3()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(border_color)
                                                .bg(tag_bg)
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .cursor_pointer()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(text_color)
                                                        .child(theme_mode_label),
                                                )
                                                .child(
                                                    svg()
                                                        .data(crate::icons::CHEVRON_DOWN_SVG)
                                                        .size(px(12.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.show_theme_mode_picker = !this.show_theme_mode_picker;
                                                    this.show_cursor_style_picker = false;
                                                    this.show_scrollback_picker = false;
                                                    cx.notify();
                                                })),
                                        )
                                        // Dropdown Popover Menu
                                        .children(if show_theme_picker {
                                            Some(
                                                div()
                                                    .absolute()
                                                    .top(px(46.0))
                                                    .right(px(16.0))
                                                    .w(px(110.0))
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(border_color)
                                                    .bg(card_bg)
                                                    .shadow_lg()
                                                    .py_1()
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if theme_mode == "all" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-01").hover(|s| s.bg(tag_bg))
                                                            .child("All themes")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_theme_mode_filter("all", cx);
                                                            })),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if theme_mode == "dark" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-02").hover(|s| s.bg(tag_bg))
                                                            .child("Dark")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_theme_mode_filter("dark", cx);
                                                            })),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if theme_mode == "light" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-03").hover(|s| s.bg(tag_bg))
                                                            .child("Light")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_theme_mode_filter("light", cx);
                                                            })),
                                                    ),
                                            )
                                        } else {
                                            None
                                        }),
                                )
                                // Divider Line
                                .child(div().h(px(1.0)).bg(border_color).w_full())
                                // Row 2: Color Theme Cards Grid
                                .child(
                                    div()
                                        .px_4()
                                        .py_4()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child("Color Theme"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .child(format!("{} themes available", filtered_presets.len())),
                                                ),
                                        )
                                        // 3 Columns Grid
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .flex_wrap()
                                                .gap(px(12.0))
                                                .children(filtered_presets.into_iter().map(|preset| {
                                                    let is_active = preset.id == active_preset_id;
                                                    let preset_id = preset.id;
                                                    let p_bg = rgb(preset.background);
                                                    let p_primary = rgb(preset.primary);
                                                    let p_accent = rgb(preset.accent);
                                                    let p_destructive = rgb(preset.destructive);
                                                    let p_fg = rgb(preset.foreground);
                                                    let p_muted_fg = rgb(preset.muted_foreground);
                                                    let clean_label = preset
                                                        .label
                                                        .trim_end_matches(" Dark")
                                                        .trim_end_matches(" Light");

                                                    div()
                                                        .relative()
                                                        .w(px(204.0))
                                                        .p_2()
                                                        .rounded_lg()
                                                        .border_2()
                                                        .border_color(if is_active {
                                                            primary_color
                                                        } else {
                                                            border_color
                                                        })
                                                        .bg(if is_active {
                                                            tag_bg
                                                        } else {
                                                            secondary_bg
                                                        })
                                                        .cursor_pointer()
                                                        .id(ElementId::Name(format!("settings-preset-{}", preset.id).into())).hover(|s| s.border_color(rgb(0x71717a)))
                                                        // Mini preview swatch box (h-14 / 56px)
                                                        .child(
                                                            div()
                                                                .h(px(56.0))
                                                                .w_full()
                                                                .rounded_md()
                                                                .p_2()
                                                                .flex()
                                                                .flex_col()
                                                                .justify_between()
                                                                .bg(p_bg)
                                                                // Top-left 3 colored dots
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .flex_row()
                                                                        .gap_1()
                                                                        .child(div().size(px(8.0)).rounded_full().bg(p_primary))
                                                                        .child(div().size(px(8.0)).rounded_full().bg(p_accent))
                                                                        .child(div().size(px(8.0)).rounded_full().bg(p_destructive)),
                                                                )
                                                                // Bottom-left 2 bar lines
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .flex_row()
                                                                        .items_end()
                                                                        .gap_1()
                                                                        .child(div().w(px(110.0)).h(px(4.0)).rounded_sm().bg(p_fg))
                                                                        .child(div().w(px(40.0)).h(px(4.0)).rounded_sm().bg(p_muted_fg)),
                                                                ),
                                                        )
                                                        // Bottom Theme Label
                                                        .child(
                                                            div()
                                                                .mt_1p5()
                                                                .px_1()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .justify_between()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::MEDIUM)
                                                                        .text_color(if is_active { text_color } else { muted_text })
                                                                        .child(clean_label),
                                                                ),
                                                        )
                                                        // Active Checkmark (top-right absolute)
                                                        .children(if is_active {
                                                            Some(
                                                                div()
                                                                    .absolute()
                                                                    .top(px(8.0))
                                                                    .right(px(8.0))
                                                                    .child(
                                                                        svg()
                                                                            .data(crate::icons::CHECK_SVG)
                                                                            .size(px(14.0))
                                                                            .text_color(primary_color),
                                                                    ),
                                                            )
                                                        } else {
                                                            None
                                                        })
                                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                            this.set_theme_preset(preset_id, cx);
                                                        }))
                                                })),
                                        ),
                                ),
                        ),
                )
                // ====================================================
                // 2. TERMINAL SECTION (Excluding terminal type & engine)
                // ====================================================
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(muted_text)
                                .child("TERMINAL"),
                        )
                        .child(
                            div()
                                .rounded_lg()
                                .border_1()
                                .border_color(border_color)
                                .bg(card_bg)
                                .overflow_hidden()
                                // Row 1: Font
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_3()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::TYPE_SVG)
                                                        .size(px(16.0))
                                                        .text_color(muted_text),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child("Font"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .px_2p5()
                                                .py_1()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(border_color)
                                                .bg(tag_bg)
                                                .text_xs()
                                                .text_color(text_color)
                                                .cursor_pointer()
                                                .id("settings-05").hover(|s| s.bg(border_color))
                                                .child(font_display)
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.font_dialog_family = this.terminal_font_family.clone();
                                                    this.font_dialog_size = this.terminal_font_size;
                                                    this.show_font_dialog = true;
                                                    this.show_font_dialog_picker = false;
                                                    cx.notify();
                                                })),
                                        ),
                                )
                                // Divider
                                .child(div().h(px(1.0)).bg(border_color).w_full())
                                // Row 2: Cursor Style
                                .child(
                                    div()
                                        .relative()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_3()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::MOUSE_POINTER_2_SVG)
                                                        .size(px(16.0))
                                                        .text_color(muted_text),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child("Cursor Style"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .w(px(120.0))
                                                .h(px(32.0))
                                                .px_3()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(border_color)
                                                .bg(tag_bg)
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .cursor_pointer()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(text_color)
                                                        .child(cursor_style_label),
                                                )
                                                .child(
                                                    svg()
                                                        .data(crate::icons::CHEVRON_DOWN_SVG)
                                                        .size(px(12.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.show_cursor_style_picker = !this.show_cursor_style_picker;
                                                    this.show_theme_mode_picker = false;
                                                    this.show_scrollback_picker = false;
                                                    cx.notify();
                                                })),
                                        )
                                        // Cursor Style Dropdown Popover
                                        .children(if show_cursor_picker {
                                            Some(
                                                div()
                                                    .absolute()
                                                    .top(px(46.0))
                                                    .right(px(16.0))
                                                    .w(px(120.0))
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(border_color)
                                                    .bg(card_bg)
                                                    .shadow_lg()
                                                    .py_1()
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if cursor_style_val == "block" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-06").hover(|s| s.bg(tag_bg))
                                                            .child("Block (▮)")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_cursor_style("block".to_string(), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if cursor_style_val == "underline" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-07").hover(|s| s.bg(tag_bg))
                                                            .child("Underline (_)")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_cursor_style("underline".to_string(), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if cursor_style_val == "bar" { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-08").hover(|s| s.bg(tag_bg))
                                                            .child("Bar (|)")
                                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                                this.set_cursor_style("bar".to_string(), cx);
                                                            })),
                                                    ),
                                            )
                                        } else {
                                            None
                                        }),
                                )
                                // Divider
                                .child(div().h(px(1.0)).bg(border_color).w_full())
                                // Row 3: Cursor Blink
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_3()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    div()
                                                        .size(px(6.0))
                                                        .rounded_full()
                                                        .bg(if cursor_blink_active { primary_color } else { muted_text }),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child("Cursor Blink"),
                                                ),
                                        )
                                        // Switch Toggle Button
                                        .child(
                                            div()
                                                .w(px(36.0))
                                                .h(px(20.0))
                                                .rounded_full()
                                                .p_0p5()
                                                .cursor_pointer()
                                                .bg(if cursor_blink_active {
                                                    primary_color
                                                } else {
                                                    border_color
                                                })
                                                .flex()
                                                .items_center()
                                                .child(
                                                    div()
                                                        .size(px(16.0))
                                                        .rounded_full()
                                                        .bg(rgb(0xffffff))
                                                        .ml(if cursor_blink_active { px(16.0) } else { px(1.0) }),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.set_cursor_blink(!this.cursor_blink, cx);
                                                })),
                                        ),
                                )
                                // Divider
                                .child(div().h(px(1.0)).bg(border_color).w_full())
                                // Row 4: Scrollback Buffer
                                .child(
                                    div()
                                        .relative()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_3()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::HISTORY_SVG)
                                                        .size(px(16.0))
                                                        .text_color(muted_text),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(text_color)
                                                        .child("Scrollback Buffer"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .w(px(140.0))
                                                .h(px(32.0))
                                                .px_3()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(border_color)
                                                .bg(tag_bg)
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .cursor_pointer()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(text_color)
                                                        .child(scrollback_label),
                                                )
                                                .child(
                                                    svg()
                                                        .data(crate::icons::CHEVRON_DOWN_SVG)
                                                        .size(px(12.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.show_scrollback_picker = !this.show_scrollback_picker;
                                                    this.show_theme_mode_picker = false;
                                                    this.show_cursor_style_picker = false;
                                                    cx.notify();
                                                })),
                                        )
                                        // Scrollback Dropdown Popover
                                        .children(if show_scrollback_picker {
                                            Some(
                                                div()
                                                    .absolute()
                                                    .top(px(46.0))
                                                    .right(px(16.0))
                                                    .w(px(140.0))
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(border_color)
                                                    .bg(card_bg)
                                                    .shadow_lg()
                                                    .py_1()
                                                    .children([
                                                        (1000, "1,000 lines"),
                                                        (5000, "5,000 lines"),
                                                        (10000, "10,000 lines"),
                                                        (50000, "50,000 lines"),
                                                        (0, "Unlimited"),
                                                    ].iter().map(|&(val, label)| {
                                                        let is_selected = scrollback_val == val;
                                                        div()
                                                            .px_3()
                                                            .py_1p5()
                                                            .text_xs()
                                                            .text_color(if is_selected { primary_color } else { text_color })
                                                            .cursor_pointer()
                                                            .id("settings-09").hover(|s| s.bg(tag_bg))
                                                            .child(label)
                                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                                this.set_scrollback(val, cx);
                                                            }))
                                                    })),
                                            )
                                        } else {
                                            None
                                        }),
                                ),
                        ),
                ),
        )
        // ====================================================
        // ADVANCED: Backend binary path (override)
        // ====================================================
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            svg()
                                .data(crate::icons::TERMINAL_SVG)
                                .size(px(16.0))
                                .text_color(muted_text),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(text_color)
                                .child("Backend Binary"),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_text)
                        .child("Override the path to the backend executable. Takes effect after restarting the app."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div().flex_1().child(
                                Input::new(&app.inputs().backend_path)
                                    .with_size(gpui_component::Size::Small)
                                    .w_full()
                                    .text_size(px(12.0)),
                            ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(accent_color)
                                .id("settings-10").hover(|s| s.opacity(0.9))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(accent_fg)
                                .child("Save")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                    let value = AppState::input_value(&this.inputs().backend_path, cx);
                                    this.set_backend_path_override(&value, window, cx);
                                })),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(tag_bg)
                                .id("settings-11").hover(|s| s.bg(border_color))
                                .cursor_pointer()
                                .text_xs()
                                .text_color(text_color)
                                .child("Reset")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                    this.reset_backend_path_override(window, cx);
                                    AppState::set_input_value(&this.inputs().backend_path, "", window, cx);
                                })),
                        ),
                ),
        )
        // ====================================================
        // 3. FONT DIALOG MODAL (1:1 with FontDialog.tsx)
        // ====================================================
        .children(if show_font_dialog {
            let temp_font = app.font_dialog_family.clone();
            let temp_size = app.font_dialog_size;
            let show_font_picker = app.show_font_dialog_picker;

            Some(
                div()
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x000000_cc))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .w(px(520.0))
                            .max_w(px(520.0))
                            .rounded_xl()
                            .border_1()
                            .border_color(border_color)
                            .bg(card_bg)
                            .p_6()
                            .shadow_2xl()
                            .flex()
                            .flex_col()
                            .gap_5()
                            // Modal Header: Terminal Font
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_color)
                                    .child("Terminal Font"),
                            )
                            // Field 1: Font Family Selector
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(muted_text)
                                            .child("FONT FAMILY"),
                                    )
                                    .child(
                                        div()
                                            .relative()
                                            .child(
                                                div()
                                                    .w_full()
                                                    .h(px(36.0))
                                                    .px_3()
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(border_color)
                                                    .bg(tag_bg)
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .justify_between()
                                                    .cursor_pointer()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(text_color)
                                                            .child(temp_font.clone()),
                                                    )
                                                    .child(
                                                        svg()
                                                            .data(crate::icons::CHEVRON_DOWN_SVG)
                                                            .size(px(14.0))
                                                            .text_color(muted_text),
                                                    )
                                                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                        this.show_font_dialog_picker = !this.show_font_dialog_picker;
                                                        cx.notify();
                                                    })),
                                            )
                                            // Font Dropdown List
                                            .children(if show_font_picker {
                                                Some(
                                                    div()
                                                        .id("font-family-dropdown-scroll")
                                                        .absolute()
                                                        .top(px(40.0))
                                                        .left_0()
                                                        .w_full()
                                                        .max_h(px(180.0))
                                                        .overflow_y_scroll()
                                                        .rounded_md()
                                                        .border_1()
                                                        .border_color(border_color)
                                                        .bg(card_bg)
                                                        .shadow_2xl()
                                                        .py_1()
                                                        .children(MONO_FONTS.iter().map(|&f| {
                                                            let is_selected = temp_font == f;
                                                            div()
                                                                .px_3()
                                                                .py_1p5()
                                                                .text_xs()
                                                                .text_color(if is_selected { primary_color } else { text_color })
                                                                .cursor_pointer()
                                                                .id(ElementId::Name(format!("settings-font-{}", f).into())).hover(|s| s.bg(tag_bg))
                                                                .child(f)
                                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                                    this.font_dialog_family = f.to_string();
                                                                    this.show_font_dialog_picker = false;
                                                                    cx.notify();
                                                                }))
                                                        })),
                                                )
                                            } else {
                                                None
                                            }),
                                    ),
                            )
                            // Field 2: Font Size Selector
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(text_color)
                                                    .child(format!("Font Size ({:.0}px)", temp_size)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .children([10.0f32, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 18.0, 20.0, 22.0, 24.0].iter().map(|&sz| {
                                                let is_active = (temp_size - sz).abs() < 0.1;
                                                div()
                                                    .px_2p5()
                                                    .py_1()
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(if is_active { primary_color } else { border_color })
                                                    .bg(if is_active { tag_bg } else { secondary_bg })
                                                    .text_xs()
                                                    .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                    .text_color(if is_active { primary_color } else { text_color })
                                                    .cursor_pointer()
                                                    .child(format!("{sz:.0}"))
                                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                        this.font_dialog_size = sz;
                                                        cx.notify();
                                                    }))
                                            })),
                                    ),
                            )
                            // Field 3: Preview Container (1:1 content with FontDialog.tsx)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(muted_text)
                                            .child("PREVIEW"),
                                    )
                                    .child(
                                        div()
                                            .p_3()
                                            .rounded_md()
                                            .bg(tag_bg)
                                            .border_1()
                                            .border_color(border_color)
                                            .h(px(100.0))
                                            .overflow_hidden()
                                            .child(
                                                div()
                                                    .font_family(SharedString::from(temp_font.clone()))
                                                    .text_size(px(temp_size))
                                                    .text_color(text_color)
                                                    .child("$ ls -la /home/user\n\
                                                            drwxr-xr-x  5 user staff 160 Apr 28 10:30 .\n\
                                                            -rw-r--r--  1 user staff  42 Apr 28 10:30 .bashrc\n\
                                                            $ "),
                                            ),
                                    ),
                            )
                            // Dialog Footer: Cancel & Save Changes
                            .child(
                                div()
                                    .mt_2()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_end()
                                    .gap_3()
                                    .child(
                                        div()
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(border_color)
                                            .bg(tag_bg)
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(text_color)
                                            .cursor_pointer()
                                            .id("settings-13").hover(|s| s.bg(border_color))
                                            .child("Cancel")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                this.show_font_dialog = false;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_2()
                                            .rounded_md()
                                            .bg(primary_color)
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(app.primary_fg())
                                            .cursor_pointer()
                                            .id("settings-14").hover(|s| s.opacity(0.9))
                                            .child("Save Changes")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                let f = this.font_dialog_family.clone();
                                                let sz = this.font_dialog_size;
                                                this.set_terminal_font(f, sz, cx);
                                                this.show_font_dialog = false;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            )
        } else {
            None
        })
        .into_any_element()
}
