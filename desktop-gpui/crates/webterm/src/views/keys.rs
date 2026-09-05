//! SSH Keys vault view and Add Key modal dialog.

use gpui::*;
use crate::app_state::AppState;

/// Render the SSH Keys management view.
pub fn render_keys_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let bg_color = app.bg_color();
    let toolbar_bg = app.bg_color();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let muted_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    let count = app.ssh_keys.len();
    let is_loading = app.is_loading_keys;
    let keys = app.ssh_keys.clone();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        // Top Toolbar matching fe/ SSHKeysPage header
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .px_6()
                .py_3()
                .bg(toolbar_bg)
                .border_b_1()
                .border_color(border_color)
                // Left: Title and "SECURE STORAGE" subtitle
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("SSH Keys"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(muted_text)
                                .child("SECURE STORAGE"),
                        ),
                )
                // Right: Action button (+ Upload Key)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_3p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(primary_color)
                        .hover(|s| s.opacity(0.9))
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(primary_fg)
                        .cursor_pointer()
                        .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                        .child("Upload Key")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                            this.open_add_key_modal(cx);
                        })),
                ),
        )
        // Main Content Area: Key Cards Grid or Empty State
        .child(
            div()
                .id("keys-scroll-area")
                .flex_1()
                .w_full()
                .overflow_y_scroll()
                .p_6()
                .child(if count == 0 {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .py_24()
                .gap_3()
                .text_color(muted_text)
                .child(
                    div()
                        .size(px(48.0))
                        .rounded_full()
                        .bg(muted_bg)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(svg().data(crate::icons::KEY_SVG).size(px(24.0)).text_color(muted_text)),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_color)
                        .child(if is_loading { "Loading SSH keys..." } else { "No SSH keys in vault" }),
                )
                .child(
                    div()
                        .text_xs()
                        .max_w(px(400.0))
                        .text_center()
                        .child("Add private keys to the pool to enable key-based SSH authentication across your hosts."),
                )
                .child(
                    div()
                        .mt_2()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(primary_color)
                        .hover(|s| s.opacity(0.9))
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(primary_fg)
                        .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                        .child("Upload First Key")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                            this.open_add_key_modal(cx);
                        })),
                )
                .into_any_element()
        } else {
            // Chunk keys into rows of 3 to produce 1:1 grid-cols-3 layout matching Electron
            let chunks: Vec<Vec<_>> = keys
                .chunks(3)
                .map(|c| c.to_vec())
                .collect();

            div()
                .flex()
                .flex_col()
                .gap_4()
                .w_full()
                .pb_12()
                .children(chunks.into_iter().map(|chunk| {
                    let chunk_len = chunk.len();
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .w_full()
                        .children(chunk.into_iter().map(|key| {
                            let key_id_del = key.id.clone();

                            div()
                                .flex_1()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .py_4()
                                .px_4()
                                .gap_3()
                                .rounded_xl()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .hover(move |s| s.border_color(primary_color))
                                // Left: Key icon box & info
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .size(px(34.0))
                                                .rounded_md()
                                                .bg(muted_bg)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .flex_shrink_0()
                                                .child(
                                                    svg()
                                                        .data(crate::icons::KEY_SVG)
                                                        .size(px(16.0))
                                                        .text_color(muted_text),
                                                ),
                                        )
                                        // Middle: Name + Encrypted pill + Fingerprint
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .min_w_0()
                                                .gap_0p5()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(text_color)
                                                                .truncate()
                                                                .child(key.name.clone()),
                                                        )
                                                        .children(if key.has_passphrase {
                                                            Some(
                                                                div()
                                                                    .px_1p5()
                                                                    .py_0()
                                                                    .rounded_sm()
                                                                    .text_xs()
                                                                    .bg(muted_bg)
                                                                    .text_color(primary_color)
                                                                    .child("Encrypted"),
                                                            )
                                                        } else {
                                                            None
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(muted_text)
                                                        .truncate()
                                                        .child(key.fingerprint.clone()),
                                                ),
                                        ),
                                )
                                // Right: Delete icon button
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(24.0))
                                        .rounded_md()
                                        .cursor_pointer()
                                        .hover(move |s| s.bg(muted_bg))
                                        .child(
                                            svg()
                                                .data(crate::icons::TRASH_SVG)
                                                .size(px(13.0))
                                                .text_color(muted_text),
                                        )
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.delete_ssh_key(&key_id_del, cx);
                                        }))
                                )
                        }))
                        // Spacer cards for last row so cards stay 1/3 width
                        .children((0..(3 - chunk_len)).map(|_| {
                            div().flex_1()
                        }))
                }))
                .into_any_element()
        })
        )
        .into_any_element()
}

/// Sample PEM template for quick filling in testing/dev environments.
const ED25519_SAMPLE_PEM: &str = "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW\nQyNTUxOQAAACD7V0bUeE/mZlZ1j3RXZ2hvbWVoZXJlAAAAIAAAAAtzc2gtZWQyNTUxOQAA\nACD7V0bUeE/mZlZ1j3RXZ2hvbWVoZXJlAAAA\n-----END OPENSSH PRIVATE KEY-----";

/// Render the Add SSH Key modal dialog overlay.
pub fn render_add_key_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    let name_display: SharedString = if app.new_key_name.is_empty() {
        "e.g. id_ed25519_deploy".into()
    } else {
        app.new_key_name.clone().into()
    };

    let pem_display: SharedString = if app.new_key_pem.is_empty() {
        "Paste OpenSSH private key PEM content here...".into()
    } else {
        format!("{} characters entered", app.new_key_pem.len()).into()
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
            this.close_add_key_modal(cx);
        }))
        // Modal card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(540.0))
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
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("Upload New SSH Key"),
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
                                    this.close_add_key_modal(cx);
                                })),
                        ),
                )
                // Error banner (if any)
                .children(app.add_key_error.as_ref().map(|err| {
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
                // Form Fields
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        // Key Name
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Key Name *"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(if app.new_key_name.is_empty() { muted_text } else { text_color })
                                        .child(name_display),
                                ),
                        )
                        // Private Key PEM
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Private Key (PEM Format) *"))
                                .child(
                                    div()
                                        .p_3()
                                        .h(px(120.0))
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_xs()
                                        .font_family("JetBrains Mono")
                                        .text_color(if app.new_key_pem.is_empty() { muted_text } else { text_color })
                                        .child(pem_display),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .pt_1()
                                        .child(div().text_xs().text_color(muted_text).child("Quick fill:"))
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(tag_bg)
                                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child("Load Test Ed25519 Key")
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    this.new_key_name = "test_ed25519".to_string();
                                                    this.new_key_pem = ED25519_SAMPLE_PEM.to_string();
                                                    cx.notify();
                                                })),
                                        ),
                                ),
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
                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_sm()
                                .text_color(text_color)
                                .child("Cancel")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_add_key_modal(cx);
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
                                .child("Upload Key")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.create_ssh_key(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
