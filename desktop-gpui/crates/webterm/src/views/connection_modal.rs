//! Connection Create/Edit sheet (right-side push-aside panel) and JSON Import modal.

use crate::app_state::{AppState, ConnectionModalMode};
use crate::views::sheet::{
    sheet_body, sheet_error_banner, sheet_footer, sheet_header, sheet_input_row, sheet_panel,
};
use gpui::*;

/// Render the Connection Create/Edit sheet (pushes the content pane aside).
pub fn render_connection_sheet(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let tag_bg = app.muted_bg();
    let primary_color = app.primary_color();
    let primary_fg = app.primary_fg();

    let form = match &app.connection_modal {
        Some(f) => f.clone(),
        None => return div().into_any_element(),
    };

    let is_edit = matches!(form.mode, ConnectionModalMode::Edit(_));
    let title = if is_edit {
        "Edit Connection"
    } else {
        "New Connection"
    };
    let description = if is_edit {
        "Update your saved connection details."
    } else {
        "Add a new SSH connection to your library."
    };
    let submit_label = if is_edit {
        "Save Changes"
    } else {
        "Create Connection"
    };

    let is_key_auth = form.auth_method == "key";
    let selected_key_id = form.ssh_key_id.clone();
    let available_keys = app.ssh_keys.clone();

    let popular_tags = ["prod", "staging", "web", "database", "infra"];

    sheet_panel(app)
        .child(sheet_header(
            app,
            cx,
            title,
            description,
            AppState::close_connection_modal,
        ))
        .children(
            form.error_message
                .map(|err| sheet_error_banner(app, err.into()).into_any_element()),
        )
        .child(
            // Scrollable form body
            sheet_body().child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    // Label Row
                    .child(sheet_input_row(
                        app,
                        "Connection Label *",
                        &app.inputs().conn_label,
                    ))
                    // Host & Port Row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(div().flex_1().child(sheet_input_row(
                                app,
                                "Host / IP *",
                                &app.inputs().conn_host,
                            )))
                            .child(div().w(px(90.0)).child(sheet_input_row(
                                app,
                                "Port *",
                                &app.inputs().conn_port,
                            ))),
                    )
                    // Username Row
                    .child(sheet_input_row(
                        app,
                        "Username *",
                        &app.inputs().conn_username,
                    ))
                    // Password Row
                    .child(sheet_input_row(
                        app,
                        if is_edit {
                            "Password (leave blank to keep current)"
                        } else {
                            "Password (optional)"
                        },
                        &app.inputs().conn_password,
                    ))
                    // Auth Method Toggle (Password vs SSH Key)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child("Authentication Method"),
                            )
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
                                            .bg(if !is_key_auth { primary_color } else { tag_bg })
                                            .text_color(if !is_key_auth {
                                                primary_fg
                                            } else {
                                                muted_text
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(
                                                svg()
                                                    .data(crate::icons::LOCK_SVG)
                                                    .size(px(13.0))
                                                    .text_color(if !is_key_auth {
                                                        primary_fg
                                                    } else {
                                                        muted_text
                                                    }),
                                            )
                                            .child("Password")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.auth_method = "password".to_string();
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
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
                                            .bg(if is_key_auth { primary_color } else { tag_bg })
                                            .text_color(if is_key_auth {
                                                primary_fg
                                            } else {
                                                muted_text
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(
                                                svg()
                                                    .data(crate::icons::KEY_SVG)
                                                    .size(px(13.0))
                                                    .text_color(if is_key_auth {
                                                        primary_fg
                                                    } else {
                                                        muted_text
                                                    }),
                                            )
                                            .child("SSH Key")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.auth_method = "key".to_string();
                                                        if f.ssh_key_id.is_none()
                                                            && !this.ssh_keys.is_empty()
                                                        {
                                                            f.ssh_key_id =
                                                                Some(this.ssh_keys[0].id.clone());
                                                        }
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                    ),
                            ),
                    )
                    // Dynamic Auth Section (Key selector when key auth)
                    .child(if is_key_auth {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child("Select SSH Key *"),
                            )
                            .child(if available_keys.is_empty() {
                                div()
                                    .p_3()
                                    .rounded_md()
                                    .bg(app.bg_color())
                                    .border_1()
                                    .border_color(border_color)
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child(
                                        "No SSH keys found. Upload a key in the 'SSH Keys' view.",
                                    )
                                    .into_any_element()
                            } else {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(available_keys.into_iter().map(|key| {
                                        let is_selected =
                                            selected_key_id.as_deref() == Some(&key.id);
                                        let key_id_clone = key.id.clone();
                                        div()
                                            .px_3()
                                            .py_1p5()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .border_1()
                                            .border_color(if is_selected {
                                                primary_color
                                            } else {
                                                border_color
                                            })
                                            .bg(if is_selected { tag_bg } else { app.bg_color() })
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
                                                    .child(
                                                        svg()
                                                            .data(crate::icons::KEY_SVG)
                                                            .size(px(13.0))
                                                            .text_color(if is_selected {
                                                                primary_color
                                                            } else {
                                                                muted_text
                                                            }),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(if is_selected {
                                                                FontWeight::BOLD
                                                            } else {
                                                                FontWeight::NORMAL
                                                            })
                                                            .text_color(text_color)
                                                            .child(format!(
                                                                "{} ({})",
                                                                key.name, key.key_type
                                                            )),
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
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    if let Some(f) = &mut this.connection_modal {
                                                        f.ssh_key_id = Some(key_id_clone.clone());
                                                        cx.notify();
                                                    }
                                                }),
                                            )
                                    }))
                                    .into_any_element()
                            })
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    // Tags Row & Quick Presets
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(sheet_input_row(app, "Tags", &app.inputs().conn_tags))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .pt_1()
                                    .child(
                                        div().text_xs().text_color(muted_text).child("Quick add:"),
                                    )
                                    .children(popular_tags.iter().map(|tag| {
                                        let tag_str = tag.to_string();
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .bg(tag_bg)
                                            .hover(|s| s.bg(border_color))
                                            .cursor_pointer()
                                            .text_xs()
                                            .text_color(text_color)
                                            .child(format!("+ {}", tag))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, window, cx| {
                                                    let current = AppState::input_value(
                                                        &this.inputs().conn_tags,
                                                        cx,
                                                    );
                                                    let mut tags: Vec<String> = current
                                                        .split(',')
                                                        .map(|s| s.trim().to_string())
                                                        .filter(|s| !s.is_empty())
                                                        .collect();
                                                    if !tags.contains(&tag_str) {
                                                        tags.push(tag_str.clone());
                                                    }
                                                    let joined = tags.join(", ");
                                                    AppState::set_input_value(
                                                        &this.inputs().conn_tags,
                                                        &joined,
                                                        window,
                                                        cx,
                                                    );
                                                }),
                                            )
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
            |this, _window, cx| this.close_connection_modal(cx),
            |this, window, cx| this.save_connection_form(window, cx),
        ))
        .into_any_element()
}
