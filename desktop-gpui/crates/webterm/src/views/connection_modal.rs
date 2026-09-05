//! Connection Create/Edit modal dialog and JSON Import modal.

use gpui::*;
use webterm_settings::Theme as SettingsTheme;
use crate::app_state::{AppState, ConnectionModalMode};

/// Render the Connection Create/Edit modal dialog overlay.
pub fn render_connection_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let input_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let tag_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    let form = match &app.connection_modal {
        Some(f) => f.clone(),
        None => return div().into_any_element(),
    };

    let is_edit = matches!(form.mode, ConnectionModalMode::Edit(_));
    let title = if is_edit { "Edit SSH Host" } else { "Add New SSH Host" };

    let label_display: SharedString = if form.label.is_empty() {
        "e.g. Production Web".into()
    } else {
        form.label.clone().into()
    };

    let host_display: SharedString = if form.host.is_empty() {
        "e.g. 192.168.1.50 or host.domain.com".into()
    } else {
        form.host.clone().into()
    };

    let port_display: SharedString = form.port.clone().into();
    let user_display: SharedString = if form.username.is_empty() {
        "root".into()
    } else {
        form.username.clone().into()
    };

    let tags_display: SharedString = if form.tags.is_empty() {
        "e.g. prod, web, us-east (click tags below to add)".into()
    } else {
        form.tags.clone().into()
    };

    let is_key_auth = form.auth_method == "key";
    let selected_key_id = form.ssh_key_id.clone();
    let available_keys = app.ssh_keys.clone();

    let popular_tags = ["prod", "staging", "web", "database", "infra"];

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        // Dismiss on background click
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.close_connection_modal(cx);
        }))
        // Modal Card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(540.0))
                .max_h(px(640.0))
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .shadow_lg()
                .p_6()
                .gap_4()
                .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop propagation
                // Modal Header
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
                                .child(title),
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
                                    this.close_connection_modal(cx);
                                })),
                        ),
                )
                // Error banner (if validation failed)
                .children(form.error_message.map(|err| {
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
                        .child(err)
                }))
                // Form Fields
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        // Label Row
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Connection Label *"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(if form.label.is_empty() { muted_text } else { text_color })
                                        .child(label_display),
                                ),
                        )
                        // Host & Port Row
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_3()
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().text_xs().text_color(muted_text).child("Host / IP *"))
                                        .child(
                                            div()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(input_bg)
                                                .border_1()
                                                .border_color(border_color)
                                                .text_sm()
                                                .text_color(if form.host.is_empty() { muted_text } else { text_color })
                                                .child(host_display),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(80.0))
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().text_xs().text_color(muted_text).child("Port *"))
                                        .child(
                                            div()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(input_bg)
                                                .border_1()
                                                .border_color(border_color)
                                                .text_sm()
                                                .text_color(text_color)
                                                .child(port_display),
                                        ),
                                ),
                        )
                        // Username Row
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Username *"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(text_color)
                                        .child(user_display),
                                ),
                        )
                        // Auth Method Toggle (Password vs SSH Key)
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Authentication Method"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        // Password Pill
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1p5()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(if !is_key_auth {
                                                    if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) }
                                                } else {
                                                    tag_bg
                                                })
                                                .text_color(if !is_key_auth { rgb(0xffffff) } else { muted_text })
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(svg().data(crate::icons::LOCK_SVG).size(px(13.0)).text_color(if !is_key_auth { rgb(0xffffff) } else { muted_text }))
                                                .child("Password")
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.auth_method = "password".to_string();
                                                        cx.notify();
                                                    }
                                                })),
                                        )
                                        // SSH Key Pill
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_1p5()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .bg(if is_key_auth {
                                                    if is_dark { rgb(0x0284c7) } else { rgb(0x0ea5e9) }
                                                } else {
                                                    tag_bg
                                                })
                                                .text_color(if is_key_auth { rgb(0xffffff) } else { muted_text })
                                                .cursor_pointer()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(svg().data(crate::icons::KEY_SVG).size(px(13.0)).text_color(if is_key_auth { rgb(0xffffff) } else { muted_text }))
                                                .child("SSH Key")
                                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.auth_method = "key".to_string();
                                                        if f.ssh_key_id.is_none() && !this.ssh_keys.is_empty() {
                                                            f.ssh_key_id = Some(this.ssh_keys[0].id.clone());
                                                        }
                                                        cx.notify();
                                                    }
                                                })),
                                        ),
                                ),
                        )
                        // Dynamic Auth Section (Password input or Key selector)
                        .child(if is_key_auth {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Select SSH Key *"))
                                .child(if available_keys.is_empty() {
                                    div()
                                        .p_3()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_xs()
                                        .text_color(muted_text)
                                        .child("No SSH keys found. Upload a key in the 'SSH Keys' view.")
                                        .into_any_element()
                                } else {
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .children(available_keys.into_iter().map(|key| {
                                            let is_selected = selected_key_id.as_deref() == Some(&key.id);
                                            let key_id_clone = key.id.clone();
                                            div()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .border_1()
                                                .border_color(if is_selected {
                                                    if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
                                                } else {
                                                    border_color
                                                })
                                                .bg(if is_selected {
                                                    if is_dark { rgb(0x0c4a6e) } else { rgb(0xe0f2fe) }
                                                } else {
                                                    input_bg
                                                })
                                                .hover(|s| s.bg(tag_bg))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_1p5()
                                                        .child(svg().data(crate::icons::KEY_SVG).size(px(13.0)).text_color(if is_selected { rgb(0x38bdf8) } else { muted_text }))
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                                .text_color(text_color)
                                                                .child(format!("{} ({})", key.name, key.key_type)),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_family("JetBrains Mono")
                                                        .text_color(muted_text)
                                                        .child(if key.fingerprint.len() > 16 {
                                                            format!("{}...", &key.fingerprint[..16])
                                                        } else {
                                                            key.fingerprint
                                                        }),
                                                )
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.ssh_key_id = Some(key_id_clone.clone());
                                                        cx.notify();
                                                    }
                                                }))
                                        }))
                                        .into_any_element()
                                })
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child(if is_edit {
                                    "Password (leave blank to keep current)"
                                } else {
                                    "Password (optional)"
                                }))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_sm()
                                        .text_color(muted_text)
                                        .child(if form.password.is_empty() {
                                            "••••••••"
                                        } else {
                                            "●●●●●●●●"
                                        }),
                                )
                        })
                        // Tags Row & Quick Presets
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(muted_text).child("Tags"))
                                .child(
                                    div()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(input_bg)
                                        .border_1()
                                        .border_color(border_color)
                                        .text_xs()
                                        .text_color(if form.tags.is_empty() { muted_text } else { text_color })
                                        .child(tags_display),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .pt_1()
                                        .child(div().text_xs().text_color(muted_text).child("Quick add:"))
                                        .children(popular_tags.iter().map(|tag| {
                                            let tag_str = tag.to_string();
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .bg(tag_bg)
                                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                                .cursor_pointer()
                                                .text_xs()
                                                .text_color(text_color)
                                                .child(format!("+ {}", tag))
                                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        let mut current_tags = f.parse_tags();
                                                        if !current_tags.contains(&tag_str) {
                                                            current_tags.push(tag_str.clone());
                                                            f.tags = current_tags.join(", ");
                                                            cx.notify();
                                                        }
                                                    }
                                                }))
                                        })),
                                ),
                        ),
                )
                // Action Buttons (Cancel / Save)
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
                                    this.close_connection_modal(cx);
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
                                .child("Save Host")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.save_connection_form(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

/// Render the JSON Import modal dialog overlay.
pub fn render_import_modal(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let input_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let tag_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };

    let export_file_exists = std::path::Path::new("webterm-connections-export.json").exists();

    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x00000088))
        // Dismiss on background click
        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
            this.close_import_modal(cx);
        }))
        // Modal Card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(520.0))
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
                                .child("Import Connections (JSON)"),
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
                                    this.close_import_modal(cx);
                                })),
                        ),
                )
                // Instructions
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_text)
                        .child("WebTerm connects to existing hosts exported as JSON array. Connections with matching labels will be updated, while new labels are added."),
                )
                // File status / quick import
                .child(
                    div()
                        .p_3()
                        .rounded_lg()
                        .bg(input_bg)
                        .border_1()
                        .border_color(border_color)
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .child(svg().data(crate::icons::FOLDER_SVG).size(px(14.0)).text_color(muted_text))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(text_color)
                                        .child(if export_file_exists {
                                            "Found: webterm-connections-export.json"
                                        } else {
                                            "Place export file at: webterm-connections-export.json"
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if export_file_exists {
                                    if is_dark { rgb(0x15803d) } else { rgb(0x22c55e) }
                                } else {
                                    tag_bg
                                })
                                .text_color(if export_file_exists { rgb(0xffffff) } else { muted_text })
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Import File")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.submit_import_connections(cx);
                                })),
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
                                    this.close_import_modal(cx);
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
                                .child("Run Import")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.submit_import_connections(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
