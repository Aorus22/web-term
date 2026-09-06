//! Session-scoped SSH Key Passphrase prompt dialog.

use gpui::*;
use crate::app_state::AppState;

/// Render the passphrase prompt dialog overlay for an encrypted SSH key.
pub fn render_passphrase_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    let pass_display: SharedString = if app.passphrase_input.is_empty() {
        "Enter private key passphrase...".into()
    } else {
        "●".repeat(app.passphrase_input.len()).into()
    };

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        // Dismiss on background click
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.cancel_passphrase(cx);
        }))
        // Modal Card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(460.0))
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .shadow_lg()
                .p_6()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                // Header
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
                                .child(svg().data(crate::icons::LOCK_SVG).size(px(18.0)).text_color(primary_color))
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_color)
                                        .child("Unlock SSH Key"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(24.0))
                                .rounded_md()
                                .hover(|s| s.bg(tag_bg))
                                .cursor_pointer()
                                .text_color(muted_text)
                                .child(svg().data(crate::icons::X_SVG).size(px(14.0)).text_color(muted_text))
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.cancel_passphrase(cx);
                                })),
                        ),
                )
                // Security context note
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_text)
                        .child("This private key is passphrase-protected. Passphrases are cached in session memory only and are never saved to disk."),
                )
                // Error banner (if any)
                .children(app.passphrase_error.as_ref().map(|err| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                        .border_1()
                        .border_color(rgb(0xef4444))
                        .text_xs()
                        .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                        .child(svg().data(crate::icons::ALERT_TRIANGLE_SVG).size(px(13.0)).text_color(rgb(0xef4444)))
                        .child(err.clone())
                }))
                // Passphrase Input Field
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_xs().text_color(muted_text).child("Passphrase"))
                        .child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .bg(input_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_sm()
                                .text_color(if app.passphrase_input.is_empty() { muted_text } else { text_color })
                                .child(pass_display),
                        ),
                )
                // Action Buttons
                .child(
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
                                .hover(|s| s.bg(border_color))
                                .cursor_pointer()
                                .text_sm()
                                .text_color(text_color)
                                .child("Cancel")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.cancel_passphrase(cx);
                                })),
                        )
                        .child(
                            div()
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(primary_color)
                                .hover(|s| s.opacity(0.9))
                                .cursor_pointer()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(primary_fg)
                                .child("Unlock & Connect")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.submit_passphrase(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
