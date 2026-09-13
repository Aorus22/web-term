//! SSH Keys vault view and Upload/Edit Key sheets (right-side push-aside panels).

use crate::app_state::AppState;
use crate::views::sheet::{
    sheet_body, sheet_error_banner, sheet_footer, sheet_header, sheet_panel,
};
use gpui::*;

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
                            let key_id_edit = key.id.clone();

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
                                // Right: Edit + Delete icon buttons
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        // Edit icon button
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
                                                        .data(crate::icons::EDIT_SVG)
                                                        .size(px(13.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.open_edit_key_modal(&key_id_edit, cx);
                                                })),
                                        )
                                        // Delete icon button
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
                                                })),
                                        ),
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

/// Render the Upload SSH Key sheet (right-side push-aside panel).
pub fn render_add_key_sheet(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = app.muted_bg();

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

    sheet_panel(app)
        .child(sheet_header(
            app,
            cx,
            "Upload SSH Key",
            "Add a private key to your secure pool. Key material is encrypted on the server.",
            AppState::close_add_key_modal,
        ))
        .children(
            app.add_key_error
                .as_ref()
                .map(|err| sheet_error_banner(app, err.clone().into()).into_any_element()),
        )
        // Scrollable form body
        .child(
            sheet_body()
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
                                .text_color(if app.new_key_name.is_empty() {
                                    muted_text
                                } else {
                                    text_color
                                })
                                .child(name_display),
                        ),
                )
                // Private Key PEM
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child("SSH Private Key *"),
                        )
                        .child(
                            div()
                                .p_3()
                                .h(px(240.0))
                                .rounded_md()
                                .bg(input_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .font_family("JetBrains Mono")
                                .text_color(if app.new_key_pem.is_empty() {
                                    muted_text
                                } else {
                                    text_color
                                })
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
                                        .hover(|s| {
                                            s.bg(if is_dark {
                                                rgb(0x52525b)
                                            } else {
                                                rgb(0xcbd5e1)
                                            })
                                        })
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child("Load Test Ed25519 Key")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _window, cx| {
                                                this.new_key_name = "test_ed25519".to_string();
                                                this.new_key_pem = ED25519_SAMPLE_PEM.to_string();
                                                cx.notify();
                                            }),
                                        ),
                                ),
                        ),
                ),
        )
        .child(sheet_footer(
            app,
            cx,
            "Cancel",
            "Upload Key",
            AppState::close_add_key_modal,
            AppState::create_ssh_key,
        ))
        .into_any_element()
}

/// Render the Edit SSH Key sheet (rename / replace key material).
pub fn render_edit_key_sheet(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let input_bg = app.bg_color();
    let tag_bg = app.muted_bg();

    let Some(form) = app.edit_key_modal.clone() else {
        return div().into_any_element();
    };

    let name_display: SharedString = if form.name.is_empty() {
        "e.g. id_ed25519_deploy".into()
    } else {
        form.name.clone().into()
    };

    let pem_display: SharedString = if form.new_pem.is_empty() {
        "(leave empty to keep current key)".into()
    } else {
        format!("{} characters entered", form.new_pem.len()).into()
    };

    let submit_label = "Save Changes";

    sheet_panel(app)
        .child(sheet_header(
            app,
            cx,
            "Edit SSH Key",
            "Update the name for your SSH key. You can also upload a new key file to replace the existing one.",
            AppState::close_edit_key_modal,
        ))
        .children(form
            .error_message
            .map(|err| sheet_error_banner(app, err.into()).into_any_element()))
        // Scrollable form body
        .child(sheet_body()
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
                                        .text_color(if form.name.is_empty() { muted_text } else { text_color })
                                        .child(name_display),
                                ),
                        )
                        // Replace Key PEM (optional)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Replace Key File (optional)"))
                                .child(
                                    div()
                                        .p_3()
                                        .h(px(200.0))
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_xs()
                                        .font_family("JetBrains Mono")
                                        .text_color(if form.new_pem.is_empty() { muted_text } else { text_color })
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
                                                    if let Some(f) = &mut this.edit_key_modal {
                                                        f.new_pem = ED25519_SAMPLE_PEM.to_string();
                                                        cx.notify();
                                                    }
                                                })),
                                        ),
                                ),
                        ),
        )
        .child(sheet_footer(
            app,
            cx,
            "Cancel",
            submit_label,
            AppState::close_edit_key_modal,
            AppState::save_edit_key_form,
        ))
        .into_any_element()
}
