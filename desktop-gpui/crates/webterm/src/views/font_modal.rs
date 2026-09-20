//! Terminal Font dialog (1:1 with the web `FontDialog.tsx`).
//!
//! Rendered as a shell-level overlay (see `views::nav`) rather than inside the
//! settings page: the settings body is a scroll container, and a dialog nested
//! inside it scrolls away with the page instead of staying pinned. Mounting it
//! next to the other modals keeps the backdrop fixed over the whole window.

use crate::app_state::AppState;
use gpui::*;

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

/// Modal backdrop + dialog panel for the terminal font picker.
pub fn render_font_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let secondary_bg = app.secondary_bg();
    let primary_fg = app.primary_fg();
    let temp_font = app.font_dialog_family.clone();
    let temp_size = app.font_dialog_size;
    let show_font_picker = app.show_font_dialog_picker;

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x000000_cc))
        .flex()
        .items_center()
        .justify_center()
        // `occlude` flags this hitbox as BlockMouse, which is the only thing
        // that actually blocks: every hitbox painted before it stops counting
        // as hovered, so clicks, hovers and scroll wheels cannot reach the
        // settings page underneath. A plain listener would not block anything.
        .occlude()
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
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _window, cx| {
                                                this.show_font_dialog_picker =
                                                    !this.show_font_dialog_picker;
                                                cx.notify();
                                            }),
                                        ),
                                )
                                // Font Dropdown List (deferred: paints above the
                                // Font Size and Preview fields below it)
                                .children(if show_font_picker {
                                    Some(deferred(
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
                                                    .text_color(if is_selected {
                                                        primary_color
                                                    } else {
                                                        text_color
                                                    })
                                                    .cursor_pointer()
                                                    .id(ElementId::Name(
                                                        format!("settings-font-{}", f).into(),
                                                    ))
                                                    .hover(|s| s.bg(tag_bg))
                                                    .child(f)
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, _window, cx| {
                                                            this.font_dialog_family = f.to_string();
                                                            this.show_font_dialog_picker = false;
                                                            cx.notify();
                                                        }),
                                                    )
                                            })),
                                    ))
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
                            div().flex().justify_between().child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(text_color)
                                    .child(format!("Font Size ({:.0}px)", temp_size)),
                            ),
                        )
                        .child(
                            div().flex().flex_row().items_center().gap_2().children(
                                [
                                    10.0f32, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 18.0, 20.0, 22.0,
                                    24.0,
                                ]
                                .iter()
                                .map(|&sz| {
                                    let is_active = (temp_size - sz).abs() < 0.1;
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(if is_active {
                                            primary_color
                                        } else {
                                            border_color
                                        })
                                        .bg(if is_active { tag_bg } else { secondary_bg })
                                        .text_xs()
                                        .font_weight(if is_active {
                                            FontWeight::BOLD
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(if is_active {
                                            primary_color
                                        } else {
                                            text_color
                                        })
                                        .cursor_pointer()
                                        .child(format!("{sz:.0}"))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _window, cx| {
                                                this.font_dialog_size = sz;
                                                cx.notify();
                                            }),
                                        )
                                }),
                            ),
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
                                        .child(
                                            "$ ls -la /home/user\n\
                                             drwxr-xr-x  5 user staff 160 Apr 28 10:30 .\n\
                                             -rw-r--r--  1 user staff  42 Apr 28 10:30 .bashrc\n\
                                             $ ",
                                        ),
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
                                .id("settings-13")
                                .hover(|s| s.bg(border_color))
                                .child("Cancel")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        this.show_font_dialog = false;
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(primary_color)
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(primary_fg)
                                .cursor_pointer()
                                .id("settings-14")
                                .hover(|s| s.opacity(0.9))
                                .child("Save Changes")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        let f = this.font_dialog_family.clone();
                                        let sz = this.font_dialog_size;
                                        this.set_terminal_font(f, sz, cx);
                                        this.show_font_dialog = false;
                                        cx.notify();
                                    }),
                                ),
                        ),
                ),
        )
        .into_any_element()
}
