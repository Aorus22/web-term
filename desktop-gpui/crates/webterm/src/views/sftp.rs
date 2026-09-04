//! SFTP Dual-Pane File Manager view: directory browsing, breadcrumbs, sorting, and source selection.

use gpui::*;
use webterm_backend_client::SftpFileInfo;
use webterm_settings::Theme as SettingsTheme;

use crate::app_state::{
    format_file_size, join_path, split_breadcrumbs, AppState, SftpActivePane,
    SftpModalState, SftpSortColumn, SftpSortOrder,
};

/// Renders the complete dual-pane SFTP file manager view.
pub fn render_sftp_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.theme != SettingsTheme::Light;
    let bg_color = if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    let left_pane = render_pane(app, SftpActivePane::Left, is_dark, cx);
    let right_pane = render_pane(app, SftpActivePane::Right, is_dark, cx);
    let modal_overlay = render_sftp_modal(app, is_dark, cx);
    let transfers_drawer = render_transfers_drawer(app, is_dark, cx);

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        .text_color(text_color)
        // Top Global SFTP Toolbar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(border_color)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(div().text_base().font_weight(FontWeight::BOLD).child("📁 SFTP Dual-Pane Manager"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child("Local Filesystem & Remote SSH Host File Transfer"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_xs()
                                .child("🔄 Refresh Both")
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.sftp_load_pane(SftpActivePane::Left, cx);
                                    this.sftp_load_pane(SftpActivePane::Right, cx);
                                })),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if app.sftp_manager.transfers_drawer_open {
                                    if is_dark { rgb(0x0284c7) } else { rgb(0x38bdf8) }
                                } else if is_dark {
                                    rgb(0x27272a)
                                } else {
                                    rgb(0xe2e8f0)
                                })
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_xs()
                                .child(format!(
                                    "⚡ Transfers ({})",
                                    app.sftp_manager.transfers.len()
                                ))
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                    this.sftp_toggle_transfers_drawer(cx);
                                })),
                        ),
                ),
        )
        // Dual Pane Content Area
        .child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .w_full()
                .overflow_hidden()
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .border_r_1()
                        .border_color(border_color)
                        .child(left_pane),
                )
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .child(right_pane),
                ),
        )
        // Collapsible Bottom Transfers Drawer
        .children(transfers_drawer)
        // Modal Overlay
        .children(modal_overlay)
        .into_any_element()
}

/// Render a single pane (Left or Right).
fn render_pane(
    app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let state = app.sftp_pane(pane);
    let is_focused = app.sftp_manager.focused_pane == pane;

    let card_bg = if is_dark { rgb(0x1e1e24) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };
    let header_bg = if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) };
    let focus_indicator = if is_focused {
        if is_dark { rgb(0x38bdf8) } else { rgb(0x0284c7) }
    } else {
        border_color
    };

    let visible_files = state.visible_files();
    let total_count = state.files.len();
    let selected_count = state.selected.len();

    let breadcrumb_segments = split_breadcrumbs(&state.current_path);

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(card_bg)
        .border_t_2()
        .border_color(focus_indicator)
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
            this.sftp_focus_pane(pane, cx);
        }))
        // Pane Header: Source Selector & Focus Badge
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .bg(header_bg)
                .border_b_1()
                .border_color(border_color)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_focused { focus_indicator } else { muted_text })
                                .child(match pane {
                                    SftpActivePane::Left => "[LEFT PANE]",
                                    SftpActivePane::Right => "[RIGHT PANE]",
                                }),
                        )
                        // Source dropdown button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) })
                                .hover(|s| s.bg(if is_dark { rgb(0x52525b) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(if state.source_id == "local" || state.source_id.is_empty() {
                                    format!("💻 {}", state.source_label)
                                } else {
                                    format!("🌐 {}", state.source_label)
                                })
                                .child(" ▾")
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                    this.sftp_toggle_source_picker(pane, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted_text)
                        .child(format!("{} items ({} selected)", total_count, selected_count)),
                ),
        )
        // Source Picker Dropdown (if open)
        .children(if state.show_source_picker {
            Some(render_source_picker_dropdown(app, pane, is_dark, cx))
        } else {
            None
        })
        // Breadcrumb Navigation Bar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .px_3()
                .py_1p5()
                .border_b_1()
                .border_color(border_color)
                .bg(if is_dark { rgb(0x18181b) } else { rgb(0xf8fafc) })
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                        .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                        .cursor_pointer()
                        .text_xs()
                        .child("⬆ Up")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                            this.sftp_navigate_up(pane, cx);
                        })),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                        .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                        .cursor_pointer()
                        .text_xs()
                        .child("🔄")
                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                            this.sftp_load_pane(pane, cx);
                        })),
                )
                // Breadcrumbs trail
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .flex_wrap()
                        .gap_1()
                        .children(breadcrumb_segments.into_iter().map(|(seg_name, target_path)| {
                            let path_for_nav = target_path.clone();
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) }))
                                        .cursor_pointer()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(seg_name)
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                            this.sftp_navigate(pane, path_for_nav.clone(), cx);
                                        })),
                                )
                                .child(div().text_xs().text_color(muted_text).child("/"))
                        })),
                ),
        )
        // Sub-toolbar: Search, Show Hidden, New Folder
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .border_b_1()
                .border_color(border_color)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_text)
                                .child(if state.search_query.is_empty() {
                                    "Filter: (all)".to_string()
                                } else {
                                    format!("Filter: '{}'", state.search_query)
                                }),
                        )
                        .children(if !state.search_query.is_empty() {
                            Some(
                                div()
                                    .cursor_pointer()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .hover(|s| s.text_color(text_color))
                                    .child("× Clear")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        this.sftp_set_search_query(pane, String::new(), cx);
                                    })),
                            )
                        } else {
                            None
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(if state.show_hidden {
                                    if is_dark { rgb(0x0284c7) } else { rgb(0xbae6fd) }
                                } else if is_dark {
                                    rgb(0x27272a)
                                } else {
                                    rgb(0xe2e8f0)
                                })
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_xs()
                                .child(if state.show_hidden { "👁 Hidden: ON" } else { "👁 Hidden: OFF" })
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                    this.sftp_toggle_hidden(pane, cx);
                                })),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                                .hover(|s| s.bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xcbd5e1) }))
                                .cursor_pointer()
                                .text_xs()
                                .child("+ Folder")
                                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                    this.sftp_open_new_folder_modal(pane, cx);
                                })),
                        ),
                ),
        )
        // Table Header: Name, Size, Modified
        .child(render_table_header(state.sort_column, state.sort_order, pane, is_dark, cx))
        // File Listing or Status State
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .children(if state.is_loading {
                    Some(
                        div()
                            .p_8()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(muted_text)
                            .child("⏳ Loading directory contents..."),
                    )
                } else if let Some(ref err) = state.error {
                    Some(
                        div()
                            .p_4()
                            .m_3()
                            .rounded_md()
                            .bg(if is_dark { rgb(0x450a0a) } else { rgb(0xfef2f2) })
                            .border_1()
                            .border_color(if is_dark { rgb(0xb91c1c) } else { rgb(0xfca5a5) })
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                    .child(format!("Error loading path: {}", err)),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .bg(if is_dark { rgb(0x7f1d1d) } else { rgb(0xfecaca) })
                                    .cursor_pointer()
                                    .text_xs()
                                    .child("Retry")
                                    .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                        this.sftp_load_pane(pane, cx);
                                    })),
                            ),
                    )
                } else if visible_files.is_empty() {
                    Some(
                        div()
                            .p_8()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(muted_text)
                            .child("Empty directory"),
                    )
                } else {
                    None
                })
                .children(if !state.is_loading && state.error.is_none() {
                    Some(render_file_rows(
                        &visible_files,
                        &state.selected,
                        &state.current_path,
                        pane,
                        is_dark,
                        cx,
                    ))
                } else {
                    None
                }),
        )
        .into_any_element()
}

/// Render table header row with clickable sort columns.
fn render_table_header(
    sort_column: SftpSortColumn,
    sort_order: SftpSortOrder,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let header_bg = if is_dark { rgb(0x18181b) } else { rgb(0xf1f5f9) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    let arrow_indicator = match sort_order {
        SftpSortOrder::Ascending => " ▲",
        SftpSortOrder::Descending => " ▼",
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_1p5()
        .bg(header_bg)
        .border_b_1()
        .border_color(border_color)
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        // Name Column (clickable)
        .child(
            div()
                .flex_1()
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::Name { text_color } else { muted_text })
                .child(format!(
                    "Name{}",
                    if sort_column == SftpSortColumn::Name { arrow_indicator } else { "" }
                ))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.sftp_toggle_sort(pane, SftpSortColumn::Name, cx);
                })),
        )
        // Size Column (clickable)
        .child(
            div()
                .w(px(80.0))
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::Size { text_color } else { muted_text })
                .child(format!(
                    "Size{}",
                    if sort_column == SftpSortColumn::Size { arrow_indicator } else { "" }
                ))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.sftp_toggle_sort(pane, SftpSortColumn::Size, cx);
                })),
        )
        // Modified Column (clickable)
        .child(
            div()
                .w(px(140.0))
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::ModTime { text_color } else { muted_text })
                .child(format!(
                    "Modified{}",
                    if sort_column == SftpSortColumn::ModTime { arrow_indicator } else { "" }
                ))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.sftp_toggle_sort(pane, SftpSortColumn::ModTime, cx);
                })),
        )
        .into_any_element()
}

/// Render list of file rows for a pane.
fn render_file_rows(
    files: &[SftpFileInfo],
    selected: &std::collections::HashSet<String>,
    current_path: &str,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let hover_bg = if is_dark { rgb(0x27272a) } else { rgb(0xf1f5f9) };
    let selected_bg = if is_dark { rgb(0x1e3a5f) } else { rgb(0xbae6fd) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    let rows: Vec<AnyElement> = files
        .iter()
        .map(|file| {
            let is_selected = selected.contains(&file.name);
            let is_dir = file.is_dir;
            let file_name = file.name.clone();
            let row_name = file.name.clone();
            let target_path = if is_dir {
                join_path(current_path, &file.name)
            } else {
                String::new()
            };

            let formatted_size = if is_dir {
                "-".to_string()
            } else {
                format_file_size(file.size)
            };

            let formatted_date = if file.mod_time.len() >= 19 {
                &file.mod_time[..19]
            } else {
                &file.mod_time
            };

            div()
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .py_1p5()
                .cursor_pointer()
                .bg(if is_selected { selected_bg } else { rgba(0x00000000) })
                .hover(|s| s.bg(hover_bg))
                .text_xs()
                .text_color(text_color)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .child(if is_dir { "📁" } else { "📄" }),
                        )
                        .child(
                            div()
                                .font_weight(if is_dir { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                                .child(row_name),
                        ),
                )
                .child(
                    div()
                        .w(px(80.0))
                        .text_color(muted_text)
                        .child(formatted_size),
                )
                .child(
                    div()
                        .w(px(140.0))
                        .text_color(muted_text)
                        .child(formatted_date.to_string()),
                )
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, ev: &MouseDownEvent, _window, cx| {
                    if is_dir && ev.click_count >= 2 {
                        this.sftp_navigate(pane, target_path.clone(), cx);
                    } else {
                        let multi = ev.modifiers.control || ev.modifiers.shift;
                        this.sftp_toggle_selection(pane, file_name.clone(), multi, cx);
                    }
                }))
                .into_any_element()
        })
        .collect();

    div().flex().flex_col().children(rows).into_any_element()
}

/// Render dropdown menu for picking source connection (Local vs saved connections).
fn render_source_picker_dropdown(
    app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let hover_bg = if is_dark { rgb(0x3f3f46) } else { rgb(0xf1f5f9) };

    let mut items = Vec::new();

    // Option 1: Local Filesystem
    items.push(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .cursor_pointer()
            .hover(|s| s.bg(hover_bg))
            .text_xs()
            .text_color(text_color)
            .child("💻 Local Filesystem")
            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                this.sftp_set_source(pane, "local".to_string(), cx);
            }))
            .into_any_element(),
    );

    // Options 2+: Saved Connections
    for conn in &app.connections {
        let conn_id = conn.id.clone();
        let conn_label = conn.label.clone();
        let conn_host = format!("{}:{}", conn.host, conn.port);
        items.push(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_2()
                .px_3()
                .py_2()
                .cursor_pointer()
                .hover(|s| s.bg(hover_bg))
                .text_xs()
                .text_color(text_color)
                .child(format!("🌐 {}", conn_label))
                .child(div().text_xs().text_color(if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) }).child(conn_host))
                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.sftp_set_source(pane, conn_id.clone(), cx);
                }))
                .into_any_element(),
        );
    }

    div()
        .flex()
        .flex_col()
        .w_full()
        .bg(card_bg)
        .border_b_1()
        .border_color(border_color)
        .shadow_md()
        .children(items)
        .into_any_element()
}

/// Render bottom collapsible transfers drawer.
fn render_transfers_drawer(
    app: &AppState,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    if !app.sftp_manager.transfers_drawer_open {
        return None;
    }

    let card_bg = if is_dark { rgb(0x1e1e24) } else { rgb(0xf8fafc) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let text_color = if is_dark { rgb(0xf4f4f5) } else { rgb(0x0f172a) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    let transfers = &app.sftp_manager.transfers;

    Some(
        div()
            .flex()
            .flex_col()
            .h(px(160.0))
            .w_full()
            .bg(card_bg)
            .border_t_1()
            .border_color(border_color)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_1p5()
                    .border_b_1()
                    .border_color(border_color)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_color)
                            .child(format!("Transfer Queue ({})", transfers.len())),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(muted_text)
                            .hover(|s| s.text_color(text_color))
                            .child("▼ Minimize")
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                this.sftp_toggle_transfers_drawer(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .p_2()
                    .children(if transfers.is_empty() {
                        Some(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_full()
                                .text_xs()
                                .text_color(muted_text)
                                .child("No transfers active or in queue"),
                        )
                    } else {
                        None
                    })
                    .children(transfers.iter().map(|item| {
                        let pct = if item.total_bytes > 0 {
                            (item.bytes_transferred as f64 / item.total_bytes as f64 * 100.0) as u32
                        } else {
                            0
                        };
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(if is_dark { rgb(0x27272a) } else { rgb(0xe2e8f0) })
                            .text_xs()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(div().font_weight(FontWeight::MEDIUM).child(item.name.clone()))
                                    .child(div().text_color(muted_text).child(format!("Status: {}", item.status))),
                            )
                            .child(
                                div()
                                    .text_color(muted_text)
                                    .child(format!("{} / {} ({}%)", format_file_size(item.bytes_transferred), format_file_size(item.total_bytes), pct)),
                            )
                            .into_any_element()
                    })),
            )
            .into_any_element(),
    )
}

/// Render SFTP operation modals (New Folder, Rename, Delete confirmation).
fn render_sftp_modal(
    app: &AppState,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    let modal = app.sftp_manager.modal.as_ref()?;

    let card_bg = if is_dark { rgb(0x27272a) } else { rgb(0xffffff) };
    let border_color = if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) };
    let muted_text = if is_dark { rgb(0xa1a1aa) } else { rgb(0x64748b) };

    match modal {
        SftpModalState::NewFolder { pane, name: _ } => {
            let pane = *pane;
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000088))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                        this.sftp_close_modal(cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(380.0))
                            .rounded_xl()
                            .bg(card_bg)
                            .border_1()
                            .border_color(border_color)
                            .p_6()
                            .gap_4()
                            .on_mouse_down(MouseButton::Left, |_, _, _| {})
                            .child(div().text_base().font_weight(FontWeight::BOLD).child("Create New Directory"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child("Enter the name of the new folder to create in the current directory:"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .justify_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Cancel")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                this.sftp_close_modal(cx);
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark { rgb(0x0284c7) } else { rgb(0x38bdf8) })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Create")
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                let target = "new-folder".to_string();
                                                let curr = this.sftp_pane(pane).current_path.clone();
                                                let full = join_path(&curr, &target);
                                                let conn = this.sftp_pane(pane).source_id.clone();
                                                if let Some(client) = this.client.clone() {
                                                    crate::app_state::TOKIO_RT.spawn(async move {
                                                        let _ = client.sftp_mkdir(&conn, &full).await;
                                                    });
                                                }
                                                this.sftp_close_modal(cx);
                                                this.sftp_load_pane(pane, cx);
                                            })),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        SftpModalState::DeleteConfirm { pane, targets } => {
            let pane = *pane;
            let targets_count = targets.len();
            let targets_clone = targets.clone();
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000088))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                        this.sftp_close_modal(cx);
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(400.0))
                            .rounded_xl()
                            .bg(card_bg)
                            .border_1()
                            .border_color(border_color)
                            .p_6()
                            .gap_4()
                            .on_mouse_down(MouseButton::Left, |_, _, _| {})
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xef4444)).child("Confirm Deletion"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child(format!(
                                        "Are you sure you want to delete {} selected item(s)? This operation cannot be undone.",
                                        targets_count
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .justify_end()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark { rgb(0x3f3f46) } else { rgb(0xe2e8f0) })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Cancel")
                                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| {
                                                this.sftp_close_modal(cx);
                                            })),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(rgb(0xef4444))
                                            .hover(|s| s.bg(rgb(0xdc2626)))
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Delete Permanently")
                                            .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                                                let curr = this.sftp_pane(pane).current_path.clone();
                                                let conn = this.sftp_pane(pane).source_id.clone();
                                                let targets = targets_clone.clone();
                                                if let Some(client) = this.client.clone() {
                                                    crate::app_state::TOKIO_RT.spawn(async move {
                                                        for item in targets {
                                                            let full = join_path(&curr, &item);
                                                            let _ = client.sftp_remove(&conn, &full).await;
                                                        }
                                                    });
                                                }
                                                this.sftp_close_modal(cx);
                                                this.sftp_clear_selection(pane, cx);
                                                this.sftp_load_pane(pane, cx);
                                            })),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        _ => None,
    }
}
