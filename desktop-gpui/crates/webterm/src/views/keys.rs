//! SSH Keys vault view and Upload/Edit Key sheets (right-side push-aside panels).

use crate::app_state::AppState;
use crate::views::sheet::{
    sheet_body, sheet_error_banner, sheet_footer, sheet_header, sheet_input_row, sheet_panel,
    sheet_textarea_row,
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

    // The key sheets squeeze the toolbar; hiding the action button keeps it
    // from clipping behind the sheet edge.
    let sheet_open = app.show_add_key_modal
        || app.add_key_sheet_closing
        || app.edit_key_modal.is_some()
        || app.edit_key_sheet_closing;

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
                // Right: Action button (+ Upload Key, hidden while a sheet is open)
                .children(if sheet_open {
                    None
                } else {
                    Some(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .px_3p5()
                            .py_1p5()
                            .rounded_lg()
                            .bg(primary_color)
                            .id("keys-01")
                            .hover(|s| s.opacity(0.9))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(primary_fg)
                            .cursor_pointer()
                            .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                            .child("Upload Key")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                                this.open_add_key_modal(window, cx);
                            }))
                            .into_any_element(),
                    )
                }),
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
                        .id("keys-02").hover(|s| s.opacity(0.9))
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(primary_fg)
                        .child(svg().data(crate::icons::PLUS_SVG).size(px(13.0)).text_color(primary_fg))
                        .child("Upload First Key")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, window, cx| {
                            this.open_add_key_modal(window, cx);
                        })),
                )
                .into_any_element()
        } else {
            // Responsive wrapping grid (same fix as hosts view: fixed
            // chunks-of-3 rows overflow on narrow windows).
            // NOTE: invisible fillers below share the card flex sizing so
            // an orphan card on the last row keeps the same width instead
            // of stretching full-width via flex_grow_1.
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .w_full()
                .pb_12()
                .children(keys.into_iter().map(|key| {
                    let key_id_del = key.id.clone();
                    let key_id_edit = key.id.clone();

                    div()
                        .flex_basis(px(300.0))
                        .flex_grow_1()
                        .flex_shrink_1()
                        .min_w(px(240.0))
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
                                .id(ElementId::Name(format!("key-card-{}", key.id).into())).hover(move |s| s.border_color(primary_color))
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
                                                .id("keys-04").hover(move |s| s.bg(muted_bg))
                                                .child(
                                                    svg()
                                                        .data(crate::icons::EDIT_SVG)
                                                        .size(px(13.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, window, cx| {
                                                    this.open_edit_key_modal(&key_id_edit, window, cx);
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
                                                .id("keys-05").hover(move |s| s.bg(muted_bg))
                                                .child(
                                                    svg()
                                                        .data(crate::icons::TRASH_SVG)
                                                        .size(px(13.0))
                                                        .text_color(muted_text),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    this.open_delete_key_modal(&key_id_del, cx);
                                                })),
                                        ),
                                )
                        }))
                        // Invisible fillers: same flex sizing as cards, zero
                        // height, so they only absorb leftover grow space on
                        // the last row (covers 2-col and 3-col orphan cases).
                        .child(
                            div()
                                .flex_basis(px(300.0))
                                .flex_grow_1()
                                .flex_shrink_1()
                                .min_w(px(240.0))
                                .h(px(0.0))
                                .overflow_hidden(),
                        )
                        .child(
                            div()
                                .flex_basis(px(300.0))
                                .flex_grow_1()
                                .flex_shrink_1()
                                .min_w(px(240.0))
                                .h(px(0.0))
                                .overflow_hidden(),
                        )
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
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();

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
                .child(sheet_input_row(
                    app,
                    "Key Name *",
                    &app.inputs().new_key_name,
                ))
                // SSH Private Key PEM
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(sheet_textarea_row(
                            app,
                            "SSH Private Key *",
                            &app.inputs().new_key_pem,
                            240.0,
                        ))
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
                                        .id("keys-06").hover(|s| {
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
                                            cx.listener(|this, _, window, cx| {
                                                AppState::set_input_value(
                                                    &this.inputs().new_key_name,
                                                    "test_ed25519",
                                                    window,
                                                    cx,
                                                );
                                                AppState::set_textarea_value(
                                                    &this.inputs().new_key_pem,
                                                    ED25519_SAMPLE_PEM,
                                                    window,
                                                    cx,
                                                );
                                                cx.notify();
                                            }),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(tag_bg)
                                        .id("keys-07").hover(|s| {
                                            s.bg(if is_dark {
                                                rgb(0x52525b)
                                            } else {
                                                rgb(0xcbd5e1)
                                            })
                                        })
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child("Browse File…")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.browse_key_file_for_add(window, cx);
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
            |this, _window, cx| this.close_add_key_modal(cx),
            |this, window, cx| this.create_ssh_key(window, cx),
        ))
        .into_any_element()
}

/// Render the Edit SSH Key sheet (rename / replace key material).
pub fn render_edit_key_sheet(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();

    let Some(form) = app.edit_key_modal.clone() else {
        return div().into_any_element();
    };
    let _ = form;

    sheet_panel(app)
        .child(sheet_header(
            app,
            cx,
            "Edit SSH Key",
            "Update the name for your SSH key. You can also upload a new key file to replace the existing one.",
            AppState::close_edit_key_modal,
        ))
        .children(
            app.edit_key_modal
                .as_ref()
                .and_then(|f| f.error_message.clone())
                .map(|err| sheet_error_banner(app, err.into()).into_any_element()),
        )
        // Scrollable form body
        .child(
            sheet_body()
                // Key Name
                .child(sheet_input_row(app, "Key Name *", &app.inputs().edit_key_name))
                // Replace Key PEM (optional)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(sheet_textarea_row(
                            app,
                            "Replace Key File (optional)",
                            &app.inputs().edit_key_pem,
                            200.0,
                        ))
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
                                        .id("keys-08").hover(|s| {
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
                                            cx.listener(|this, _, window, cx| {
                                                AppState::set_textarea_value(
                                                    &this.inputs().edit_key_pem,
                                                    ED25519_SAMPLE_PEM,
                                                    window,
                                                    cx,
                                                );
                                                cx.notify();
                                            }),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(tag_bg)
                                        .id("keys-09").hover(|s| {
                                            s.bg(if is_dark {
                                                rgb(0x52525b)
                                            } else {
                                                rgb(0xcbd5e1)
                                            })
                                        })
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child("Browse File…")
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, window, cx| {
                                                this.browse_key_file_for_edit(window, cx);
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
            "Save Changes",
            |this, _window, cx| this.close_edit_key_modal(cx),
            |this, window, cx| this.save_edit_key_form(window, cx),
        ))
        .into_any_element()
}
