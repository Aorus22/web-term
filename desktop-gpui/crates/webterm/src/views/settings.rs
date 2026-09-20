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
//! - Terminal Font dialog (family selector, size presets, preview) lives in
//!   `views::font_modal` and is mounted as a shell-level overlay, so it stays
//!   pinned instead of scrolling with this page.

use crate::app_state::AppState;
use gpui::*;
use gpui_component::input::Input;
use gpui_component::scroll::{Scrollbar, ScrollbarMode};
use gpui_component::Sizable;

/// Theme grid layout: fixed-width columns and fixed-height rows so every row
/// measures identically for uniform_list virtualization (chat-list pattern
/// from WA-Bot — only visible rows are built, no matter the preset count).
const THEME_GRID_COLS: usize = 3;
const THEME_CARD_W: f32 = 204.0;
const THEME_CARD_H: f32 = 100.0;
/// Card height + 12px row gap.
const THEME_ROW_H: f32 = 112.0;
/// Four rows visible; the rest scrolls inside the grid region.
const THEME_GRID_H: f32 = 448.0;

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

    let theme_row_count =
        (filtered_presets.len() + THEME_GRID_COLS - 1) / THEME_GRID_COLS;

    // Cloned up front: handles are shared into builders below.
    let themes_handle = app.settings_themes_scroll.clone();

    let show_theme_picker = app.show_theme_mode_picker;
    let show_cursor_picker = app.show_cursor_style_picker;
    let show_scrollback_picker = app.show_scrollback_picker;

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

    // Persistent scroll state driving the visual scrollbar overlay below.
    let scroll_handle = app.settings_scroll.clone();

    // Outer relative container: scroll area + floating scrollbar overlay
    // (same pattern as WA-Bot's chat list).
    div()
        .relative()
        .size_full()
        .overflow_hidden()
        .child(
            div()
                .id("settings-scroll-area")
                .flex()
                .flex_col()
                .items_center()
                .justify_start()
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&scroll_handle)
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
                                        // Dropdown Popover Menu (deferred: paints above the
                                        // rows below, which would otherwise cover it)
                                        .children(if show_theme_picker {
                                            Some(deferred(
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
                                            ))
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
                                        // Virtualized theme grid: presets are chunked into
                                        // fixed-height rows of 3 rendered through uniform_list,
                                        // so only visible rows are built no matter how many
                                        // presets exist (chat-list pattern from WA-Bot).
                                        .child(
                                            div()
                                                .relative()
                                                .w_full()
                                                .h(px(THEME_GRID_H))
                                                .overflow_hidden()
                                                .child(
                                                    uniform_list(
                                                        "settings-theme-grid",
                                                        theme_row_count,
                                                        cx.processor(
                                                            |this: &mut AppState,
                                                             range: std::ops::Range<usize>,
                                                             _window: &mut Window,
                                                             cx: &mut Context<AppState>| {
                                                            let active = this.settings.theme_preset.clone();
                                                            let mode = this.theme_mode_filter.clone();
                                                            let presets: Vec<&'static crate::theme::ThemePreset> =
                                                                crate::theme::THEME_PRESETS
                                                                    .iter()
                                                                    .filter(|p| match mode.as_str() {
                                                                        "dark" => p.is_dark,
                                                                        "light" => !p.is_dark,
                                                                        _ => true,
                                                                    })
                                                                    .collect();
                                                            let rows: Vec<&[&'static crate::theme::ThemePreset]> =
                                                                presets.chunks(THEME_GRID_COLS).collect();
                                                            let border_color = this.border_color();
                                                            let text_color = this.text_color();
                                                            let muted_text = this.muted_text();
                                                            let tag_bg = this.muted_bg();
                                                            let secondary_bg = this.secondary_bg();
                                                            let primary_color = this.primary_color();
                                                            range
                                                                .map(|row_ix| {
                                                                    let row: &[&'static crate::theme::ThemePreset] =
                                                                        rows.get(row_ix).copied().unwrap_or(&[]);
                                                                    div()
                                                                        .h(px(THEME_ROW_H))
                                                                        .flex()
                                                                        .flex_row()
                                                                        .gap(px(12.0))
                                                                        .children(row.iter().enumerate().map(
                                                                            |(col_ix, preset)| {
                                                                                let is_active =
                                                                                    preset.id == active;
                                                                                let preset_id = preset.id;
                                                                                let global_ix =
                                                                                    row_ix * THEME_GRID_COLS + col_ix;
                                                                                let p_bg = rgb(preset.background);
                                                                                let p_primary =
                                                                                    rgb(preset.primary);
                                                                                let p_accent =
                                                                                    rgb(preset.accent);
                                                                                let p_destructive =
                                                                                    rgb(preset.destructive);
                                                                                let p_fg =
                                                                                    rgb(preset.foreground);
                                                                                let p_muted_fg =
                                                                                    rgb(preset.muted_foreground);
                                                                                let clean_label = preset
                                                                                    .label
                                                                                    .trim_end_matches(" Dark")
                                                                                    .trim_end_matches(" Light");
                                                                                div()
                                                                                    .relative()
                                                                                    .w(px(THEME_CARD_W))
                                                                                    .h(px(THEME_CARD_H))
                                                                                    .overflow_hidden()
                                                                                    .flex()
                                                                                    .flex_col()
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
                                                                                    .id(ElementId::NamedInteger(
                                                                                        "settings-preset".into(),
                                                                                        global_ix as u64,
                                                                                    ))
                                                                                    .hover(|s| {
                                                                                        s.border_color(rgb(0x71717a))
                                                                                    })
                                                                                    // Mini preview swatch box (h-14 / 56px)
                                                                                    .child(
                                                                                        div()
                                                                                            .h(px(56.0))
                                                                                            .w_full()
                                                                                            .flex_shrink_0()
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
                                                                                    // Bottom Theme Label (single line: keeps rows uniform)
                                                                                    .child(
                                                                                        div()
                                                                                            .mt_1p5()
                                                                                            .px_1()
                                                                                            .flex_1()
                                                                                            .min_h_0()
                                                                                            .overflow_hidden()
                                                                                            .flex()
                                                                                            .flex_row()
                                                                                            .items_center()
                                                                                            .justify_between()
                                                                                            .child(
                                                                                                div()
                                                                                                    .flex_1()
                                                                                                    .min_w_0()
                                                                                                    .truncate()
                                                                                                    .text_xs()
                                                                                                    .font_weight(FontWeight::MEDIUM)
                                                                                                    .text_color(if is_active {
                                                                                                        text_color
                                                                                                    } else {
                                                                                                        muted_text
                                                                                                    })
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
                                                                                    .on_mouse_down(
                                                                                        MouseButton::Left,
                                                                                        cx.listener(move |this, _, _window, cx| {
                                                                                            this.set_theme_preset(preset_id, cx);
                                                                                        }),
                                                                                    )
                                                                            },
                                                                        ))
                                                                        // Fillers keep a short last row aligned.
                                                                        .children(
                                                                            (row.len()..THEME_GRID_COLS).map(|_| {
                                                                                div()
                                                                                    .w(px(THEME_CARD_W))
                                                                                    .h(px(THEME_CARD_H))
                                                                            }),
                                                                        )
                                                                })
                                                                .collect()
                                                            }
                                                        )
                                                    )
                                                    .w_full()
                                                    .h_full()
                                                    .track_scroll(&themes_handle)
                                                )
                                                .child(
                                                    Scrollbar::vertical(&themes_handle)
                                                        .mode(ScrollbarMode::Hover)
                                                        .styles(|s| {
                                                            s.track(|t| t.bg(Hsla::from(rgba(0x00000000))))
                                                                .thumb(|th| {
                                                                    th.bg(Hsla::from(muted_text.opacity(0.35)))
                                                                        .radius(px(3.0))
                                                                        .width(px(6.0))
                                                                })
                                                                .thumb_hover(|th| {
                                                                    th.bg(Hsla::from(muted_text.opacity(0.65)))
                                                                        .radius(px(4.0))
                                                                        .width(px(8.0))
                                                                })
                                                                .thumb_active(|th| {
                                                                    th.bg(Hsla::from(primary_color.opacity(0.8)))
                                                                        .radius(px(4.0))
                                                                        .width(px(8.0))
                                                                })
                                                        }),
                                                ),
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
                                // No overflow_hidden here (unlike the APPEARANCE
                                // card): the rows are transparent and the open
                                // dropdown must not be clipped by the card edge.
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
                                        // Cursor Style Dropdown Popover (deferred: paints
                                        // above the rows below, which would otherwise cover it)
                                        .children(if show_cursor_picker {
                                            Some(deferred(
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
                                            ))
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
                                        // Scrollback Dropdown Popover (deferred: paints
                                        // above everything below, incl. the next section)
                                        .children(if show_scrollback_picker {
                                            Some(deferred(
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
                                            ))
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
        )
        .child(
            Scrollbar::vertical(&scroll_handle)
                .mode(ScrollbarMode::Always)
                .styles(|s| {
                    s.track(|t| t.bg(Hsla::from(rgba(0x00000000))))
                        .thumb(|th| {
                            th.bg(Hsla::from(muted_text.opacity(0.35)))
                                .radius(px(3.0))
                                .width(px(6.0))
                        })
                        .thumb_hover(|th| {
                            th.bg(Hsla::from(muted_text.opacity(0.65)))
                                .radius(px(4.0))
                                .width(px(8.0))
                        })
                        .thumb_active(|th| {
                            th.bg(Hsla::from(primary_color.opacity(0.8)))
                                .radius(px(4.0))
                                .width(px(8.0))
                        })
                }),
        )
        .into_any_element()
}
