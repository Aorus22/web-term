//! SSH Keys vault view and Add Key modal dialog.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::AppState;

/// Render the SSH Keys management view.
pub fn render_keys_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let tag_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    let count = app.ssh_keys.len();
    let is_loading = app.is_loading_keys;
    let keys = app.ssh_keys.clone();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        .p_4()
        .gap_4()
        // Top Toolbar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .flex_wrap()
                .gap_3()
                // Left: Title and key count badge
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(text_color)
                                .child("SSH Key Vault"),
                        )
                        .child(
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(tag_bg)
                                .text_xs()
                                .text_color(muted_text)
                                .child(format!("{count} key{}", if count == 1 { "" } else { "s" })),
                        ),
                )
                // Right: Actions (Refresh, + Add Key)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        // Refresh
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(card_bg)
                                .border_1()
                                .border_color(border_color)
                                .text_xs()
                                .cursor_pointer()
                                .hover(|s| s.bg(tag_bg))
                                .child(if is_loading { "⟳ Loading..." } else { "⟳ Refresh" })
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.fetch_ssh_keys(cx);
                                })),
                        )
                        // + Add Key button
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) })
                                .hover(|s| s.bg(if is_dark { rgb(0x0369a1) } else { rgb(0x0284c7) }))
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xffffff))
                                .cursor_pointer()
                                .child("+ Add SSH Key")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.open_add_key_modal(cx);
                                })),
                        ),
                ),
        )
        // Main Content Area: Key Cards Grid or Empty State
        .child(
            div()
                .id("keys-scroll-area")
                .flex_1()
                .w_full()
                .overflow_y_scroll()
                .child(if count == 0 {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .py_16()
                .gap_3()
                .text_color(muted_text)
                .child(div().text_3xl().child("🔑"))
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(text_color)
                        .child("No SSH keys in vault"),
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
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) })
                        .hover(|s| s.bg(if is_dark { rgb(0x0369a1) } else { rgb(0x0284c7) }))
                        .cursor_pointer()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xffffff))
                        .child("+ Upload First Key")
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                            this.open_add_key_modal(cx);
                        })),
                )
                .into_any_element()
        } else {
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_4()
                .children(keys.into_iter().map(|key| {
                    let key_id_del = key.id.clone();
                    let key_type_upper = key.key_type.to_uppercase();
                    let created_str = key.created_at.clone().unwrap_or_else(|| "Unknown".to_string());

                    div()
                        .flex()
                        .flex_col()
                        .w(px(380.0))
                        .rounded_lg()
                        .bg(card_bg)
                        .border_1()
                        .border_color(border_color)
                        .p_4()
                        .gap_3()
                        .shadow_sm()
                        .hover(|s| s.border_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }))
                        // Card Header: Name and Type badge
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(text_color)
                                        .child(format!("🔑 {}", key.name)),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .bg(if is_dark { rgb(0x0c4a6e) } else { rgb(0xe0f2fe) })
                                        .text_color(if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) })
                                        .child(key_type_upper),
                                ),
                        )
                        // Fingerprint details
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted_text)
                                        .child("SHA256 Fingerprint:"),
                                )
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_md()
                                        .bg(if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) })
                                        .border_1()
                                        .border_color(border_color)
                                        .text_xs()
                                        .font_family("JetBrains Mono")
                                        .text_color(text_color)
                                        .child(key.fingerprint.clone()),
                                ),
                        )
                        // Created date & actions
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .pt_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted_text)
                                        .child(format!("Added: {created_str}")),
                                )
                                .child(
                                    div()
                                        .cursor_pointer()
                                        .text_xs()
                                        .text_color(rgb(0xef4444))
                                        .hover(|s| s.text_color(rgb(0xdc2626)))
                                        .child("Delete")
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.delete_ssh_key(&key_id_del, cx);
                                        })),
                                ),
                        )
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
    let is_dark = app.theme != SettingsTheme::Light;
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let input_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let tag_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

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
                                .child("×")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.close_add_key_modal(cx);
                                })),
                        ),
                )
                // Error banner (if any)
                .children(app.add_key_error.as_ref().map(|err| {
                    div()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(if is_dark { rgb(0x451a1a) } else { rgb(0xfee2e2) })
                        .border_1()
                        .border_color(rgb(0xef4444))
                        .text_xs()
                        .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                        .child(format!("⚠️ {err}"))
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
                                .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) })
                                .hover(|s| s.bg(if is_dark { rgb(0x0369a1) } else { rgb(0x0284c7) }))
                                .cursor_pointer()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xffffff))
                                .child("Upload Key")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.create_ssh_key(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
