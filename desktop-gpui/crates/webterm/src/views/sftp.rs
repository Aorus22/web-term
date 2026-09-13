//! SFTP Dual-Pane File Manager view: directory browsing, breadcrumbs, sorting, and source selection.

use gpui::*;
use gpui_component::input::Input;
use gpui_component::Sizable;
use webterm_backend_client::SftpFileInfo;

use crate::app_state::{
    format_file_size, join_path, split_breadcrumbs, AppState, SftpActivePane, SftpDraggedItem,
    SftpModalState, SftpSortColumn, SftpSortOrder,
};

/// Drag preview element displayed under mouse during drag-and-drop.
#[derive(Clone)]
pub struct SftpDragPreview {
    pub label: String,
}

impl Render for SftpDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1p5()
            .px_3()
            .py_1p5()
            .rounded_md()
            .bg(rgb(0x0284c7))
            .text_color(rgb(0xffffff))
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .shadow_lg()
            .child(
                svg()
                    .data(crate::icons::FILE_SVG)
                    .size(px(14.0))
                    .text_color(rgb(0xffffff)),
            )
            .child(self.label.clone())
    }
}

/// Renders the complete dual-pane SFTP file manager view.
pub fn render_sftp_view(app: &mut AppState, cx: &mut Context<AppState>) -> AnyElement {
    let is_dark = app.is_dark();
    let bg_color = app.bg_color();
    let _card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();

    let focus_handle = app
        .sftp_manager
        .focus_handle
        .get_or_insert_with(|| cx.focus_handle())
        .clone();

    let left_pane = render_pane(app, SftpActivePane::Left, is_dark, cx);
    let right_pane = render_pane(app, SftpActivePane::Right, is_dark, cx);
    let transfers_drawer = render_transfers_drawer(app, is_dark, cx);

    div()
        .track_focus(&focus_handle)
        .key_context("Sftp")
        .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
            let key = ev.keystroke.key.to_lowercase();
            let is_alt = ev.keystroke.modifiers.alt;
            this.sftp_handle_key(&key, is_alt, window, cx);
        }))
        .relative()
        .flex()
        .flex_col()
        .size_full()
        .bg(bg_color)
        .text_color(text_color)
        // Dual Pane Content Area (resizable horizontal split with divider and
        // grabber handle; drag to resize, matching the web's split pane)
        .child(
            div()
                .flex()
                .flex_row()
                .flex_1()
                .size_full()
                .overflow_hidden()
                .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, cx| {
                    if this.sftp_manager.split_dragging {
                        this.sftp_split_drag_move(f32::from(ev.position.x), window, cx);
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                        this.sftp_split_drag_end(cx);
                    }),
                )
                .child(
                    div()
                        .w(gpui::relative(app.sftp_manager.split_ratio))
                        .h_full()
                        .overflow_hidden()
                        .child(left_pane),
                )
                // 1px central divider with grabber handle (9px hit area)
                .child(
                    div()
                        .w(px(9.0))
                        .h_full()
                        .cursor_col_resize()
                        .bg(border_color)
                        .relative()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, ev: &MouseDownEvent, _window, cx| {
                                this.sftp_split_drag_start(f32::from(ev.position.x), cx);
                            }),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(260.0))
                                .left(px(-10.0))
                                .w(px(21.0))
                                .h(px(26.0))
                                .rounded_sm()
                                .bg(if is_dark {
                                    rgb(0x27272a)
                                } else {
                                    rgb(0xe2e8f0)
                                })
                                .border_1()
                                .border_color(border_color)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .data(crate::icons::GRABBER_SVG)
                                        .size(px(12.0))
                                        .text_color(muted_text),
                                ),
                        ),
                )
                .child(div().flex_1().h_full().overflow_hidden().child(right_pane)),
        )
        // Collapsible Bottom Transfers Drawer
        .children(transfers_drawer)
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
    let _is_focused = app.sftp_manager.focused_pane == pane;

    let card_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let header_bg = app.card_bg();

    let any_popover_open = state.show_source_picker
        || state.show_actions_menu
        || state.show_drive_picker
        || app.sftp_manager.context_menu.is_some()
        || app.sftp_manager.modal.is_some();
    let is_interactive = !any_popover_open;

    let visible_files = state.visible_files();

    div()
        .flex()
        .flex_col()
        .size_full()
        .bg(card_bg)
        .relative()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, window, cx| {
                this.sftp_focus_pane(pane, cx);
                if let Some(ref fh) = this.sftp_manager.focus_handle {
                    window.focus(fh, cx);
                }
            }),
        )
        .on_drop::<SftpDraggedItem>(cx.listener(
            move |this, dragged: &SftpDraggedItem, _window, cx| {
                if dragged.source_pane != pane {
                    this.sftp_transfer_between_panes(
                        dragged.source_pane,
                        pane,
                        dragged.filenames.clone(),
                        cx,
                    );
                }
            },
        ))
        .on_drop::<ExternalPaths>(
            cx.listener(move |this, paths: &ExternalPaths, _window, cx| {
                for path in paths.paths() {
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if let Ok(data) = std::fs::read(path) {
                                this.sftp_upload_file(pane, name.to_string(), data, cx);
                            }
                        }
                    }
                }
            }),
        )
        // 1. Pane Header: Machine Selector (Left) & Actions Menu (Right)
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
                    // Source selector button (Local Machine v)
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .hover(|s| {
                            s.bg(if is_dark {
                                rgb(0x27272a)
                            } else {
                                rgb(0xe2e8f0)
                            })
                        })
                        .cursor_pointer()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(text_color)
                                .child(state.source_label.clone()),
                        )
                        .child(
                            svg()
                                .data(crate::icons::CHEVRON_DOWN_SVG)
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_toggle_source_picker(pane, cx);
                            }),
                        ),
                )
                .child(
                    // Actions dropdown trigger (Actions v)
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .hover(|s| {
                            s.bg(if is_dark {
                                rgb(0x27272a)
                            } else {
                                rgb(0xe2e8f0)
                            })
                        })
                        .cursor_pointer()
                        .child(div().text_xs().text_color(muted_text).child("Actions"))
                        .child(
                            svg()
                                .data(crate::icons::CHEVRON_DOWN_SVG)
                                .size(px(12.0))
                                .text_color(muted_text),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_toggle_actions_menu(pane, cx);
                            }),
                        ),
                ),
        )
        // 2. Breadcrumbs Bar
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1p5()
                .px_3()
                .py_1p5()
                .bg(header_bg)
                .border_b_1()
                .border_color(border_color)
                .overflow_hidden()
                // Back button `<`
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .cursor(if state.can_go_back() {
                            CursorStyle::PointingHand
                        } else {
                            CursorStyle::Arrow
                        })
                        .opacity(if state.can_go_back() { 1.0 } else { 0.25 })
                        .hover(|s| {
                            if state.can_go_back() {
                                s.bg(if is_dark {
                                    rgb(0x27272a)
                                } else {
                                    rgb(0xe2e8f0)
                                })
                            } else {
                                s
                            }
                        })
                        .text_sm()
                        .text_color(text_color)
                        .child("‹")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_navigate_back(pane, cx);
                            }),
                        ),
                )
                // Forward button `>`
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .cursor(if state.can_go_forward() {
                            CursorStyle::PointingHand
                        } else {
                            CursorStyle::Arrow
                        })
                        .opacity(if state.can_go_forward() { 1.0 } else { 0.25 })
                        .hover(|s| {
                            if state.can_go_forward() {
                                s.bg(if is_dark {
                                    rgb(0x27272a)
                                } else {
                                    rgb(0xe2e8f0)
                                })
                            } else {
                                s
                            }
                        })
                        .text_sm()
                        .text_color(text_color)
                        .child("›")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_navigate_forward(pane, cx);
                            }),
                        ),
                )
                // Breadcrumbs trail
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .overflow_hidden()
                        .children(render_breadcrumbs_segments(
                            app,
                            pane,
                            &state.current_path,
                            is_dark,
                            cx,
                        )),
                ),
        )
        // 3. Column headers (Name, Date Modified, Size)
        .child(render_table_header(
            app,
            state.sort_column,
            state.sort_order,
            pane,
            cx,
        ))
        // 4. File listing
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .children(if state.is_loading {
                    Some(
                        div().p_8().flex().items_center().justify_center().child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    svg()
                                        .data(crate::icons::REFRESH_CW_SVG)
                                        .size(px(14.0))
                                        .text_color(muted_text),
                                )
                                .child("Loading directory contents..."),
                        ),
                    )
                } else if let Some(ref err) = state.error {
                    Some(
                        div()
                            .p_4()
                            .m_3()
                            .rounded_md()
                            .bg(if is_dark {
                                rgb(0x450a0a)
                            } else {
                                rgb(0xfef2f2)
                            })
                            .border_1()
                            .border_color(if is_dark {
                                rgb(0xb91c1c)
                            } else {
                                rgb(0xfca5a5)
                            })
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_dark {
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0xb91c1c)
                                    })
                                    .child(format!("Error loading path: {}", err)),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_sm()
                                    .bg(if is_dark {
                                        rgb(0x7f1d1d)
                                    } else {
                                        rgb(0xfecaca)
                                    })
                                    .cursor_pointer()
                                    .text_xs()
                                    .child("Retry")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _window, cx| {
                                            this.sftp_load_pane(pane, cx);
                                        }),
                                    ),
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
                            .child("Directory is empty"),
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
                        is_interactive,
                        cx,
                    ))
                } else {
                    None
                }),
        )
        // Source Picker Dropdown (if open) - rendered last to float on top of file rows
        .children(if state.show_source_picker {
            Some(render_source_picker_dropdown(app, pane, is_dark, cx))
        } else {
            None
        })
        // Actions Dropdown Menu (if open) - rendered last to float on top of file rows
        .children(if state.show_actions_menu {
            Some(render_actions_dropdown(app, pane, is_dark, cx))
        } else {
            None
        })
        // Drive Picker Dropdown (if open) - rendered last to float on top of file rows
        .children(if state.show_drive_picker {
            Some(render_drive_picker_dropdown(app, pane, is_dark, cx))
        } else {
            None
        })
        // Path Picker Dropdown (if open) - rendered last to float on top of file rows
        .children(if state.show_path_picker {
            Some(render_path_picker_dropdown(app, pane, is_dark, cx))
        } else {
            None
        })
        .into_any_element()
}

/// Render table header row with clickable sort columns (Name, Date Modified, Size).
fn render_table_header(
    app: &AppState,
    sort_column: SftpSortColumn,
    sort_order: SftpSortOrder,
    pane: SftpActivePane,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let header_bg = app.card_bg();
    let border_color = app.border_color();
    let text_color = app.text_color();
    let muted_text = app.muted_text();

    let is_asc = matches!(sort_order, SftpSortOrder::Ascending);

    div()
        .flex()
        .flex_row()
        .items_center()
        .px_3()
        .py_2()
        .bg(header_bg)
        .border_b_1()
        .border_color(border_color)
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        // 1. Name Column (flex-1)
        .child(
            div()
                .flex_1()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::Name {
                    text_color
                } else {
                    muted_text
                })
                .child("Name")
                .child(
                    div()
                        .text_xs()
                        .text_color(if sort_column == SftpSortColumn::Name {
                            text_color
                        } else {
                            rgb(0x71717a)
                        })
                        .child(if sort_column == SftpSortColumn::Name {
                            if is_asc {
                                "▲"
                            } else {
                                "▼"
                            }
                        } else {
                            "⇅"
                        }),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.sftp_toggle_sort(pane, SftpSortColumn::Name, cx);
                    }),
                ),
        )
        // 2. Date Modified Column (130px)
        .child(
            div()
                .w(px(130.0))
                .flex_shrink_0()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::ModTime {
                    text_color
                } else {
                    muted_text
                })
                .child("Date Modified")
                .child(
                    div()
                        .text_xs()
                        .text_color(if sort_column == SftpSortColumn::ModTime {
                            text_color
                        } else {
                            rgb(0x71717a)
                        })
                        .child(if sort_column == SftpSortColumn::ModTime {
                            if is_asc {
                                "▲"
                            } else {
                                "▼"
                            }
                        } else {
                            "⇅"
                        }),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.sftp_toggle_sort(pane, SftpSortColumn::ModTime, cx);
                    }),
                ),
        )
        // 3. Size Column (80px)
        .child(
            div()
                .w(px(80.0))
                .flex_shrink_0()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_1()
                .cursor_pointer()
                .hover(|s| s.text_color(text_color))
                .text_color(if sort_column == SftpSortColumn::Size {
                    text_color
                } else {
                    muted_text
                })
                .child("Size")
                .child(
                    div()
                        .text_xs()
                        .text_color(if sort_column == SftpSortColumn::Size {
                            text_color
                        } else {
                            rgb(0x71717a)
                        })
                        .child(if sort_column == SftpSortColumn::Size {
                            if is_asc {
                                "▲"
                            } else {
                                "▼"
                            }
                        } else {
                            "⇅"
                        }),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.sftp_toggle_sort(pane, SftpSortColumn::Size, cx);
                    }),
                ),
        )
        .into_any_element()
}

/// Render list of file rows for a pane matching DirectoryBrowser.tsx 1:1.
fn render_file_rows(
    files: &[SftpFileInfo],
    selected: &std::collections::HashSet<String>,
    current_path: &str,
    pane: SftpActivePane,
    is_dark: bool,
    is_interactive: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xf1f5f9)
    };
    let selected_bg = if is_dark {
        rgb(0x1e3a5f)
    } else {
        rgb(0xbae6fd)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };

    let pane_u64 = match pane {
        SftpActivePane::Left => 0u64,
        SftpActivePane::Right => 1u64,
    };

    let is_root = current_path.is_empty()
        || current_path == "/"
        || current_path == "."
        || (current_path.len() <= 3 && current_path.chars().nth(1) == Some(':'));

    let mut rows: Vec<AnyElement> = Vec::new();

    // 1. Parent navigation row "⤴ .." when not in root
    if !is_root {
        rows.push(
            div()
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .h(px(40.0))
                .cursor(if is_interactive {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::Arrow
                })
                .hover(|s| if is_interactive { s.bg(hover_bg) } else { s })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2p5()
                        .child(
                            svg()
                                .data(crate::icons::CORNER_LEFT_UP_SVG)
                                .size(px(16.0))
                                .text_color(muted_text),
                        )
                        .child(div().text_sm().text_color(text_color).child("..")),
                )
                .child(div().w(px(130.0)).flex_shrink_0())
                .child(div().w(px(80.0)).flex_shrink_0())
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _window, cx| {
                        if !is_interactive {
                            return;
                        }
                        this.sftp_focus_pane(pane, cx);
                        if ev.click_count >= 2 {
                            this.sftp_navigate_up(pane, cx);
                        }
                    }),
                )
                .into_any_element(),
        );
    }

    // 2. File and Directory rows
    for (idx, file) in files.iter().enumerate() {
        let is_selected = selected.contains(&file.name);
        let is_dir = file.is_dir;
        let file_name = file.name.clone();
        let file_name_right = file.name.clone();
        let is_drive_item = is_dir && file.name.len() == 2 && file.name.ends_with(':');
        let target_path = if is_dir {
            if (current_path == "/" || current_path.is_empty()) && is_drive_item {
                format!("{}/", file.name)
            } else {
                join_path(current_path, &file.name)
            }
        } else {
            String::new()
        };

        let formatted_size = if is_dir {
            "- -".to_string()
        } else {
            format_file_size(file.size)
        };

        let formatted_date = crate::app_state::format_date_modified(&file.mod_time);
        let formatted_perm = crate::app_state::format_permissions(file.mode, is_dir);

        let drag_filenames = if selected.contains(&file.name) && selected.len() > 1 {
            selected.iter().cloned().collect()
        } else {
            vec![file.name.clone()]
        };

        rows.push(
            div()
                .id(ElementId::NamedInteger(
                    "sftp-row".into(),
                    (pane_u64 << 32) | (idx as u64),
                ))
                .on_drag(
                    SftpDraggedItem {
                        source_pane: pane,
                        filenames: drag_filenames,
                    },
                    move |dragged: &SftpDraggedItem, _offset, _window, cx: &mut App| {
                        let label = if dragged.filenames.len() == 1 {
                            dragged.filenames[0].clone()
                        } else {
                            format!("{} items", dragged.filenames.len())
                        };
                        cx.new(|_| SftpDragPreview { label })
                    },
                )
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .h(px(44.0))
                .cursor(if is_interactive {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::Arrow
                })
                .bg(if is_selected {
                    selected_bg
                } else {
                    rgba(0x00000000)
                })
                .hover(|s| if is_interactive { s.bg(hover_bg) } else { s })
                // Column 1: Icon + Name + Permissions (flex-1)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2p5()
                        .child(if is_dir {
                            if is_drive_item {
                                svg()
                                    .data(crate::icons::DRIVE_SVG)
                                    .size(px(16.0))
                                    .text_color(rgb(0xf59e0b))
                            } else {
                                svg()
                                    .data(crate::icons::BLUE_FOLDER_SVG)
                                    .size(px(16.0))
                                    .text_color(rgb(0x3b82f6))
                            }
                        } else {
                            svg()
                                .data(crate::icons::FILE_SVG)
                                .size(px(16.0))
                                .text_color(muted_text)
                        })
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(text_color)
                                        .truncate()
                                        .child(file_name.clone()),
                                )
                                .child(
                                    div().text_xs().text_color(muted_text).child(formatted_perm),
                                ),
                        ),
                )
                // Column 2: Date Modified (130px)
                .child(
                    div()
                        .w(px(130.0))
                        .flex_shrink_0()
                        .text_xs()
                        .text_color(muted_text)
                        .child(formatted_date),
                )
                // Column 3: Size (80px, centered)
                .child(
                    div()
                        .w(px(80.0))
                        .flex_shrink_0()
                        .text_xs()
                        .text_color(muted_text)
                        .text_center()
                        .child(formatted_size),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                        if !is_interactive {
                            return;
                        }
                        this.sftp_focus_pane(pane, cx);
                        if let Some(ref fh) = this.sftp_manager.focus_handle {
                            window.focus(fh, cx);
                        }
                        if is_dir && ev.click_count >= 2 {
                            this.sftp_navigate(pane, target_path.clone(), cx);
                        } else {
                            let multi = ev.modifiers.control || ev.modifiers.shift;
                            this.sftp_toggle_selection(pane, file_name.clone(), multi, cx);
                        }
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                        if !is_interactive {
                            return;
                        }
                        let x = ev.position.x / px(1.0);
                        let y = ev.position.y / px(1.0);
                        this.sftp_select_single(pane, file_name_right.clone(), cx);
                        if let Some(ref fh) = this.sftp_manager.focus_handle {
                            window.focus(fh, cx);
                        }
                        this.sftp_open_context_menu(
                            pane,
                            file_name_right.clone(),
                            is_dir,
                            (x, y),
                            cx,
                        );
                    }),
                )
                .into_any_element(),
        );
    }

    div()
        .id(ElementId::NamedInteger("sftp-file-list".into(), pane_u64))
        .flex()
        .flex_col()
        .size_full()
        .overflow_y_scroll()
        .children(rows)
        .into_any_element()
}

/// Render breadcrumb segments trail with amber drive indicator and blue folders.
fn render_breadcrumbs_segments(
    app: &AppState,
    pane: SftpActivePane,
    path: &str,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> Vec<AnyElement> {
    let state = app.sftp_pane(pane);
    let text_color = app.text_color();
    let muted_text = app.muted_text();
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xe2e8f0)
    };
    let active_trigger_bg = if is_dark {
        rgb(0x9a3412)
    } else {
        rgb(0xfef3c7)
    };
    let active_trigger_text = if is_dark {
        rgb(0xffedd5)
    } else {
        rgb(0x9a3412)
    };

    let segments = split_breadcrumbs(path);
    let mut elements = Vec::new();

    let is_windows_local =
        state.source_id == "local" && path.len() >= 2 && path.chars().nth(1) == Some(':');
    let max_segments = 4;
    let tail_count = 2;
    let collapsed = segments.len() > max_segments;

    let sep = || {
        div()
            .text_xs()
            .text_color(rgb(0x71717a))
            .px_0p5()
            .child("›")
            .into_any_element()
    };

    if !collapsed {
        for (idx, (seg_name, target_path)) in segments.into_iter().enumerate() {
            let is_first = idx == 0;
            let nav_path = target_path.clone();

            if !is_first {
                elements.push(sep());
            }

            if is_first && is_windows_local {
                let is_open = state.show_drive_picker;
                elements.push(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .child(
                            svg()
                                .data(crate::icons::DRIVE_SVG)
                                .size(px(14.0))
                                .text_color(rgb(0xf59e0b)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_0p5()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .bg(if is_open {
                                    active_trigger_bg
                                } else {
                                    rgba(0x00000000)
                                })
                                .hover(move |s| if is_open { s } else { s.bg(hover_bg) })
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if is_open {
                                            active_trigger_text
                                        } else {
                                            text_color
                                        })
                                        .child(seg_name),
                                )
                                .child(
                                    svg()
                                        .data(crate::icons::CHEVRON_DOWN_SVG)
                                        .size(px(10.0))
                                        .text_color(if is_open {
                                            active_trigger_text
                                        } else {
                                            muted_text
                                        }),
                                )
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.sftp_toggle_drive_picker(pane, cx);
                                    }),
                                ),
                        )
                        .into_any_element(),
                );
            } else {
                elements.push(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .hover(|s| s.bg(hover_bg))
                        .cursor_pointer()
                        .child(
                            svg()
                                .data(crate::icons::BLUE_FOLDER_SVG)
                                .size(px(14.0))
                                .text_color(rgb(0x3b82f6)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_color)
                                .max_w(px(140.0))
                                .truncate()
                                .child(seg_name),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_navigate(pane, nav_path.clone(), cx);
                            }),
                        )
                        .into_any_element(),
                );
            }
        }
    } else {
        // Collapsed: head + ... + tail
        // 1. Head (first segment)
        let (head_name, head_path) = segments[0].clone();
        if is_windows_local {
            let is_open = state.show_drive_picker;
            elements.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        svg()
                            .data(crate::icons::DRIVE_SVG)
                            .size(px(14.0))
                            .text_color(rgb(0xf59e0b)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_0p5()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .cursor_pointer()
                            .bg(if is_open {
                                active_trigger_bg
                            } else {
                                rgba(0x00000000)
                            })
                            .hover(move |s| if is_open { s } else { s.bg(hover_bg) })
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_open {
                                        active_trigger_text
                                    } else {
                                        text_color
                                    })
                                    .child(head_name),
                            )
                            .child(
                                svg()
                                    .data(crate::icons::CHEVRON_DOWN_SVG)
                                    .size(px(10.0))
                                    .text_color(if is_open {
                                        active_trigger_text
                                    } else {
                                        muted_text
                                    }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_toggle_drive_picker(pane, cx);
                                }),
                            ),
                    )
                    .into_any_element(),
            );
        } else {
            elements.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .child(
                        svg()
                            .data(crate::icons::BLUE_FOLDER_SVG)
                            .size(px(14.0))
                            .text_color(rgb(0x3b82f6)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_color)
                            .max_w(px(140.0))
                            .truncate()
                            .child(head_name),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _window, cx| {
                            this.sftp_navigate(pane, head_path.clone(), cx);
                        }),
                    )
                    .into_any_element(),
            );
        }

        // 2. Ellipsis button ("...")
        let is_path_open = state.show_path_picker;
        elements.push(
            div()
                .flex()
                .items_center()
                .justify_center()
                .mx_0p5()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .cursor_pointer()
                .bg(if is_path_open {
                    active_trigger_bg
                } else {
                    rgba(0x00000000)
                })
                .hover(move |s| if is_path_open { s } else { s.bg(hover_bg) })
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_path_open {
                            active_trigger_text
                        } else {
                            muted_text
                        })
                        .child("..."),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.sftp_toggle_path_picker(pane, cx);
                    }),
                )
                .into_any_element(),
        );

        // 3. Tail segments
        let tail_start = segments.len() - tail_count;
        for (seg_name, target_path) in &segments[tail_start..] {
            let nav_path = target_path.clone();
            let name = seg_name.clone();
            elements.push(sep());
            elements.push(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .hover(|s| s.bg(hover_bg))
                    .cursor_pointer()
                    .child(
                        svg()
                            .data(crate::icons::BLUE_FOLDER_SVG)
                            .size(px(14.0))
                            .text_color(rgb(0x3b82f6)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_color)
                            .max_w(px(140.0))
                            .truncate()
                            .child(name),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _window, cx| {
                            this.sftp_navigate(pane, nav_path.clone(), cx);
                        }),
                    )
                    .into_any_element(),
            );
        }
    }

    elements
}

/// Render dropdown menu for picking source connection (Local vs saved connections).
fn render_source_picker_dropdown(
    app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let card_bg = if is_dark {
        rgb(0x18181b)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xf1f5f9)
    };

    let mut items = Vec::new();

    // Option 1: Local Machine
    items.push(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .px_2p5()
            .py_1p5()
            .rounded_sm()
            .cursor_pointer()
            .hover(|s| s.bg(hover_bg))
            .text_xs()
            .text_color(text_color)
            .child(
                svg()
                    .data(crate::icons::MONITOR_SVG)
                    .size(px(14.0))
                    .text_color(muted_text),
            )
            .child("Local Machine")
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _window, cx| {
                    this.sftp_set_source(pane, "local".to_string(), cx);
                }),
            )
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
                .px_2p5()
                .py_1p5()
                .rounded_sm()
                .cursor_pointer()
                .hover(|s| s.bg(hover_bg))
                .text_xs()
                .text_color(text_color)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .child(
                            svg()
                                .data(crate::icons::GLOBE_SVG)
                                .size(px(14.0))
                                .text_color(muted_text),
                        )
                        .child(conn_label),
                )
                .child(div().text_xs().text_color(muted_text).child(conn_host))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _window, cx| {
                        this.sftp_set_source(pane, conn_id.clone(), cx);
                    }),
                )
                .into_any_element(),
        );
    }

    div()
        .absolute()
        .inset_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.sftp_pane_mut(pane).show_source_picker = false;
                cx.notify();
            }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(260.0))
                .absolute()
                .top(px(38.0))
                .left(px(12.0))
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .rounded_md()
                .shadow_xl()
                .p_1()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                .children(items),
        )
        .into_any_element()
}

/// Render Actions dropdown menu matching DirectoryBrowser.tsx.
fn render_actions_dropdown(
    app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let card_bg = if is_dark {
        rgb(0x18181b)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xf1f5f9)
    };
    let destructive_text = if is_dark {
        rgb(0xf87171)
    } else {
        rgb(0xef4444)
    };

    let state = app.sftp_pane(pane);
    let has_selection = !state.selected.is_empty();
    let has_single_selection = state.selected.len() == 1;
    let sel_targets: Vec<String> = state.selected.iter().cloned().collect();
    let single_item = state.selected.iter().next().cloned();

    div()
        .absolute()
        .inset_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.sftp_pane_mut(pane).show_actions_menu = false;
                cx.notify();
            }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(180.0))
                .absolute()
                .top(px(38.0))
                .right(px(12.0))
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .rounded_md()
                .shadow_xl()
                .p_1()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                // 1. Refresh
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::REFRESH_CW_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child("Refresh")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_pane_mut(pane).show_actions_menu = false;
                                this.sftp_load_pane(pane, cx);
                            }),
                        ),
                )
                // 2. Upload File
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::UPLOAD_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child("Upload File")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_upload_from_picker(pane, cx);
                            }),
                        ),
                )
                // 3. New Folder
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::FOLDER_PLUS_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child("New Folder")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.sftp_pane_mut(pane).show_actions_menu = false;
                                this.sftp_open_new_folder_modal(pane, window, cx);
                            }),
                        ),
                )
                // Separator
                .child(div().h(px(1.0)).bg(border_color).my_1())
                // 4. Rename
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor(if has_single_selection {
                            CursorStyle::PointingHand
                        } else {
                            CursorStyle::Arrow
                        })
                        .opacity(if has_single_selection { 1.0 } else { 0.4 })
                        .hover(|s| {
                            if has_single_selection {
                                s.bg(hover_bg)
                            } else {
                                s
                            }
                        })
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::EDIT_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child("Rename")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                if let Some(ref item) = single_item {
                                    this.sftp_pane_mut(pane).show_actions_menu = false;
                                    this.sftp_open_rename_modal(pane, item.clone(), window, cx);
                                }
                            }),
                        ),
                )
                // 5. Delete
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor(if has_selection {
                            CursorStyle::PointingHand
                        } else {
                            CursorStyle::Arrow
                        })
                        .opacity(if has_selection { 1.0 } else { 0.4 })
                        .hover(|s| if has_selection { s.bg(hover_bg) } else { s })
                        .text_xs()
                        .text_color(destructive_text)
                        .child(
                            svg()
                                .data(crate::icons::TRASH_SVG)
                                .size(px(13.0))
                                .text_color(destructive_text),
                        )
                        .child("Delete")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                if has_selection {
                                    this.sftp_pane_mut(pane).show_actions_menu = false;
                                    this.sftp_open_delete_modal(pane, sel_targets.clone(), cx);
                                }
                            }),
                        ),
                )
                // Separator
                .child(div().h(px(1.0)).bg(border_color).my_1())
                // 6. Show / Hide Hidden Files
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::EYE_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child(if state.show_hidden {
                            "Hide Hidden Files"
                        } else {
                            "Show Hidden Files"
                        })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_pane_mut(pane).show_actions_menu = false;
                                this.sftp_toggle_hidden(pane, cx);
                            }),
                        ),
                )
                // 7. Transfers Drawer
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::ZAP_SVG)
                                .size(px(13.0))
                                .text_color(muted_text),
                        )
                        .child(format!("Transfers ({})", app.sftp_manager.transfers.len()))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_pane_mut(pane).show_actions_menu = false;
                                this.sftp_toggle_transfers_drawer(cx);
                            }),
                        ),
                ),
        )
        .into_any_element()
}

/// Render Windows drive volumes dropdown picker.
fn render_drive_picker_dropdown(
    _app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let card_bg = if is_dark {
        rgb(0x18181b)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xf1f5f9)
    };

    let drives = crate::app_state::get_available_drives();

    div()
        .absolute()
        .inset_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.sftp_pane_mut(pane).show_drive_picker = false;
                cx.notify();
            }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(160.0))
                .absolute()
                .top(px(72.0))
                .left(px(54.0))
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .rounded_lg()
                .shadow_xl()
                .p_1()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(muted_text)
                        .child("Volumes"),
                )
                .children(drives.into_iter().map(|drive| {
                    let drive_path = format!("{}/", drive);
                    let drive_label = drive.clone();
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::DRIVE_SVG)
                                .size(px(14.0))
                                .text_color(rgb(0xf59e0b)),
                        )
                        .child(drive_label)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_pane_mut(pane).show_drive_picker = false;
                                this.sftp_navigate(pane, drive_path.clone(), cx);
                            }),
                        )
                })),
        )
        .into_any_element()
}

/// Render popover dropdown for collapsed intermediate breadcrumb folders.
fn render_path_picker_dropdown(
    app: &AppState,
    pane: SftpActivePane,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> AnyElement {
    let card_bg = if is_dark {
        rgb(0x18181b)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let hover_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xf1f5f9)
    };

    let state = app.sftp_pane(pane);
    let segments = split_breadcrumbs(&state.current_path);
    let tail_count = 2;
    let hidden_segments: Vec<(String, String)> = if segments.len() > 4 {
        segments[1..segments.len() - tail_count].to_vec()
    } else {
        Vec::new()
    };

    div()
        .absolute()
        .inset_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _window, cx| {
                this.sftp_pane_mut(pane).show_path_picker = false;
                cx.notify();
            }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(200.0))
                .absolute()
                .top(px(72.0))
                .left(px(96.0))
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .rounded_lg()
                .shadow_xl()
                .p_1()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .text_xs()
                        .text_color(muted_text)
                        .child("Path"),
                )
                .children(hidden_segments.into_iter().map(|(name, target_path)| {
                    let nav_path = target_path.clone();
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1p5()
                        .rounded_sm()
                        .cursor_pointer()
                        .hover(|s| s.bg(hover_bg))
                        .text_xs()
                        .text_color(text_color)
                        .child(
                            svg()
                                .data(crate::icons::BLUE_FOLDER_SVG)
                                .size(px(14.0))
                                .text_color(rgb(0x3b82f6)),
                        )
                        .child(div().truncate().child(name))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _window, cx| {
                                this.sftp_pane_mut(pane).show_path_picker = false;
                                this.sftp_navigate(pane, nav_path.clone(), cx);
                            }),
                        )
                })),
        )
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

    let card_bg = if is_dark {
        rgb(0x1e1e24)
    } else {
        rgb(0xf8fafc)
    };
    let border_color = if is_dark {
        rgb(0x3f3f46)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };

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
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(muted_text)
                            .hover(|s| s.text_color(text_color))
                            .child(
                                svg()
                                    .data(crate::icons::ARROW_DOWN_SVG)
                                    .size(px(11.0))
                                    .text_color(muted_text),
                            )
                            .child("Minimize")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.sftp_toggle_transfers_drawer(cx);
                                }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .cursor_pointer()
                            .text_xs()
                            .text_color(muted_text)
                            .hover(|s| s.text_color(text_color))
                            .child("Clear History")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _window, cx| {
                                    this.sftp_clear_transfer_history(cx);
                                }),
                            ),
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
                            .flex_col()
                            .gap_0p5()
                            .px_2()
                            .py_1()
                            .rounded_sm()
                            .bg(if is_dark {
                                rgb(0x27272a)
                            } else {
                                rgb(0xe2e8f0)
                            })
                            .text_xs()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .child(item.name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_color(muted_text)
                                                    .child(format!("Status: {}", item.status)),
                                            ),
                                    )
                                    .child(div().text_color(muted_text).child(format!(
                                        "{} / {} ({}%)",
                                        format_file_size(item.bytes_transferred),
                                        format_file_size(item.total_bytes),
                                        pct
                                    ))),
                            )
                            // Progress bar
                            .child(
                                div()
                                    .w_full()
                                    .h(px(3.0))
                                    .rounded_full()
                                    .bg(if is_dark {
                                        rgb(0x3f3f46)
                                    } else {
                                        rgb(0xcbd5e1)
                                    })
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded_full()
                                            .bg(rgb(0x38bdf8))
                                            .w(px((pct as f32 / 100.0) * 720.0)),
                                    ),
                            )
                            .into_any_element()
                    })),
            )
            .into_any_element(),
    )
}

/// Render SFTP operation modals (New Folder, Rename, Delete confirmation).
pub fn render_sftp_modal(
    app: &AppState,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    let modal = app.sftp_manager.modal.as_ref()?;

    let card_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x3f3f46)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let input_bg = if is_dark {
        rgb(0x18181b)
    } else {
        rgb(0xf8fafc)
    };

    match modal {
        SftpModalState::NewFolder {
            pane,
            name: _name,
            error,
        } => {
            let pane = *pane;
            let err_opt = error.clone();
            let presets = ["docs", "assets", "src", "build", "backup", "temp"];

            Some(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000088))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.sftp_close_modal(cx);
                        }),
                    )
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
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_color)
                                    .child("Create New Directory"),
                            )
                            .child(div().text_xs().text_color(muted_text).child("Folder name:"))
                            // Input field
                            .child(
                                Input::new(&app.inputs().sftp_modal_name)
                                    .with_size(gpui_component::Size::Small)
                                    .w_full()
                                    .text_size(px(13.0)),
                            )
                            // Quick presets
                            .child(div().flex().flex_row().flex_wrap().gap_1().children(
                                presets.iter().map(|preset| {
                                    let p = preset.to_string();
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_sm()
                                        .bg(if is_dark {
                                            rgb(0x3f3f46)
                                        } else {
                                            rgb(0xe2e8f0)
                                        })
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
                                        .child(format!("+ {}", p))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, window, cx| {
                                                AppState::set_input_value(
                                                    &this.inputs().sftp_modal_name,
                                                    &p,
                                                    window,
                                                    cx,
                                                );
                                            }),
                                        )
                                }),
                            ))
                            // Error banner
                            .children(err_opt.map(|err| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(if is_dark {
                                        rgb(0x450a0a)
                                    } else {
                                        rgb(0xfee2e2)
                                    })
                                    .border_1()
                                    .border_color(rgb(0xef4444))
                                    .text_xs()
                                    .text_color(if is_dark {
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0xb91c1c)
                                    })
                                    .child(
                                        svg()
                                            .data(crate::icons::ALERT_TRIANGLE_SVG)
                                            .size(px(13.0))
                                            .text_color(rgb(0xef4444)),
                                    )
                                    .child(err)
                            }))
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
                                            .bg(if is_dark {
                                                rgb(0x3f3f46)
                                            } else {
                                                rgb(0xe2e8f0)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    this.sftp_close_modal(cx);
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark {
                                                rgb(0x0284c7)
                                            } else {
                                                rgb(0x38bdf8)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Create")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    let name = AppState::input_value(
                                                        &this.inputs().sftp_modal_name,
                                                        cx,
                                                    );
                                                    this.sftp_create_folder(pane, &name, cx);
                                                }),
                                            ),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        SftpModalState::Rename {
            pane,
            old_name,
            new_name: _new_name,
            error,
        } => {
            let pane = *pane;
            let old_name_clone = old_name.clone();
            let err_opt = error.clone();

            Some(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000088))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.sftp_close_modal(cx);
                        }),
                    )
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
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_color)
                                    .child("Rename Item"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted_text)
                                    .child(format!("Original name: {}", old_name_clone)),
                            )
                            // Input field
                            .child(
                                Input::new(&app.inputs().sftp_modal_name)
                                    .with_size(gpui_component::Size::Small)
                                    .w_full()
                                    .text_size(px(13.0)),
                            )
                            // Error banner
                            .children(err_opt.map(|err| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(if is_dark {
                                        rgb(0x450a0a)
                                    } else {
                                        rgb(0xfee2e2)
                                    })
                                    .border_1()
                                    .border_color(rgb(0xef4444))
                                    .text_xs()
                                    .text_color(if is_dark {
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0xb91c1c)
                                    })
                                    .child(
                                        svg()
                                            .data(crate::icons::ALERT_TRIANGLE_SVG)
                                            .size(px(13.0))
                                            .text_color(rgb(0xef4444)),
                                    )
                                    .child(err)
                            }))
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
                                            .bg(if is_dark {
                                                rgb(0x3f3f46)
                                            } else {
                                                rgb(0xe2e8f0)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    this.sftp_close_modal(cx);
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_4()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark {
                                                rgb(0x0284c7)
                                            } else {
                                                rgb(0x38bdf8)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Rename")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _window, cx| {
                                                    let new_name = AppState::input_value(
                                                        &this.inputs().sftp_modal_name,
                                                        cx,
                                                    );
                                                    this.sftp_rename_entry(
                                                        pane,
                                                        &old_name_clone,
                                                        &new_name,
                                                        cx,
                                                    );
                                                }),
                                            ),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        SftpModalState::Conflict {
            conflict_name,
            remaining_transfers,
            ..
        } => {
            let conflict_name = conflict_name.clone();
            let count = remaining_transfers.len();
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x00000088))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.sftp_conflict_cancel(cx);
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(420.0))
                            .rounded_xl()
                            .bg(card_bg)
                            .border_1()
                            .border_color(border_color)
                            .p_6()
                            .gap_3()
                            .on_mouse_down(MouseButton::Left, |_, _, _| {})
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_color)
                                    .child("Replace existing file?"),
                            )
                            .child(div().text_xs().text_color(muted_text).child(format!(
                                "\"{}\" already exists in the destination.{}",
                                conflict_name,
                                if count > 1 {
                                    format!(" ({} items total)", count)
                                } else {
                                    String::new()
                                }
                            )))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .justify_end()
                                    .gap_2()
                                    .mt_2()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark {
                                                rgb(0x3f3f46)
                                            } else {
                                                rgb(0xe2e8f0)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Cancel")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    this.sftp_conflict_cancel(cx);
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark {
                                                rgb(0x3f3f46)
                                            } else {
                                                rgb(0xe2e8f0)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .child("Keep Both")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    this.sftp_conflict_keep_both(cx);
                                                }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_1p5()
                                            .rounded_md()
                                            .bg(if is_dark {
                                                rgb(0x0284c7)
                                            } else {
                                                rgb(0x38bdf8)
                                            })
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xffffff))
                                            .child("Overwrite")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(|this, _, _window, cx| {
                                                    this.sftp_conflict_overwrite(cx);
                                                }),
                                            ),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
        SftpModalState::DeleteConfirm {
            pane,
            targets,
            error,
        } => {
            let pane = *pane;
            let targets_count = targets.len();
            let targets_clone = targets.clone();
            let err_opt = error.clone();

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
                            // Targets list
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .max_h(px(120.0))
                                    .overflow_hidden()
                                    .children(targets_clone.iter().map(|name| {
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded_sm()
                                            .bg(input_bg)
                                            .text_xs()
                                            .text_color(text_color)
                                            .child(format!("• {}", name))
                                    })),
                            )
                            .children(err_opt.map(|err| {
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(if is_dark { rgb(0x450a0a) } else { rgb(0xfee2e2) })
                                    .border_1()
                                    .border_color(rgb(0xef4444))
                                    .text_xs()
                                    .text_color(if is_dark { rgb(0xfca5a5) } else { rgb(0xb91c1c) })
                                    .child(svg().data(crate::icons::ALERT_TRIANGLE_SVG).size(px(13.0)).text_color(rgb(0xef4444)))
                                    .child(err)
                            }))
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
                                                this.sftp_delete_selected(pane, targets_clone.clone(), cx);
                                            })),
                                    ),
                            ),
                    )
                    .into_any_element(),
            )
        }
    }
}

/// Render SFTP right-click context menu overlay.
pub fn render_sftp_context_menu(
    app: &AppState,
    is_dark: bool,
    cx: &mut Context<AppState>,
) -> Option<AnyElement> {
    let menu = app.sftp_manager.context_menu.as_ref()?;
    let pane = menu.pane;
    let filename = menu.filename.clone();
    let other_pane = match pane {
        SftpActivePane::Left => SftpActivePane::Right,
        SftpActivePane::Right => SftpActivePane::Left,
    };
    let transfer_target_label = match other_pane {
        SftpActivePane::Left => "Left Pane",
        SftpActivePane::Right => "Right Pane",
    };

    let card_bg = if is_dark {
        rgb(0x27272a)
    } else {
        rgb(0xffffff)
    };
    let border_color = if is_dark {
        rgb(0x3f3f46)
    } else {
        rgb(0xe2e8f0)
    };
    let text_color = if is_dark {
        rgb(0xf4f4f5)
    } else {
        rgb(0x0f172a)
    };
    let muted_text = if is_dark {
        rgb(0xa1a1aa)
    } else {
        rgb(0x64748b)
    };
    let hover_bg = if is_dark {
        rgb(0x3f3f46)
    } else {
        rgb(0xf1f5f9)
    };

    let menu_w = 220.0;
    let menu_h = 160.0;

    let raw_x = (menu.position.0 - 4.0).max(8.0);
    let raw_y = (menu.position.1 - 4.0).max(8.0);

    // If near right window edge, flip to left of cursor so it never gets clipped
    let x = if raw_x + menu_w > 1200.0 {
        (raw_x - menu_w).max(8.0)
    } else {
        raw_x
    };

    // If near bottom window edge, flip upwards so it never gets pushed off
    let y = if raw_y + menu_h > 720.0 {
        (raw_y - menu_h).max(8.0)
    } else {
        raw_y
    };

    let fn_transfer = filename.clone();
    let fn_rename = filename.clone();
    let fn_delete = filename.clone();
    let fn_copy = filename.clone();

    Some(
        div()
            .absolute()
            .inset_0()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _window, cx| {
                    this.sftp_close_context_menu(cx);
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, _window, cx| {
                    this.sftp_close_context_menu(cx);
                }),
            )
            .child(
                div()
                    .absolute()
                    .top(px(y))
                    .left(px(x))
                    .w(px(menu_w))
                    .rounded_lg()
                    .bg(card_bg)
                    .border_1()
                    .border_color(border_color)
                    .shadow_xl()
                    .p_1()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .on_mouse_down(MouseButton::Left, |_, _, _| {})
                    .on_mouse_down(MouseButton::Right, |_, _, _| {})
                    // 1. Transfer to opposite pane
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(text_color)
                            .child(
                                svg()
                                    .data(crate::icons::ARROW_RIGHT_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child(format!("Transfer to {}", transfer_target_label))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_close_context_menu(cx);
                                    this.sftp_transfer_between_panes(
                                        pane,
                                        other_pane,
                                        vec![fn_transfer.clone()],
                                        cx,
                                    );
                                }),
                            ),
                    )
                    // 2. Rename
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(text_color)
                            .child(
                                svg()
                                    .data(crate::icons::EDIT_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child("Rename")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    this.sftp_close_context_menu(cx);
                                    this.sftp_open_rename_modal(
                                        pane,
                                        fn_rename.clone(),
                                        window,
                                        cx,
                                    );
                                }),
                            ),
                    )
                    // 3. Delete
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(if is_dark {
                                rgb(0xfca5a5)
                            } else {
                                rgb(0xb91c1c)
                            })
                            .child(
                                svg()
                                    .data(crate::icons::TRASH_SVG)
                                    .size(px(12.0))
                                    .text_color(if is_dark {
                                        rgb(0xfca5a5)
                                    } else {
                                        rgb(0xb91c1c)
                                    }),
                            )
                            .child("Delete")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_close_context_menu(cx);
                                    this.sftp_open_delete_modal(pane, vec![fn_delete.clone()], cx);
                                }),
                            ),
                    )
                    // Divider
                    .child(div().h_px().w_full().bg(border_color).my_0p5())
                    // Cut
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(text_color)
                            .child(
                                svg()
                                    .data(crate::icons::TRASH_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child("Cut")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_close_context_menu(cx);
                                    this.sftp_clipboard_copy(true, pane, cx);
                                }),
                            ),
                    )
                    // Copy
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(text_color)
                            .child(
                                svg()
                                    .data(crate::icons::COPY_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child("Copy")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_close_context_menu(cx);
                                    this.sftp_clipboard_copy(false, pane, cx);
                                }),
                            ),
                    )
                    // Paste Here (dimmed without clipboard)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(if app.sftp_manager.clipboard.is_some() {
                                text_color
                            } else {
                                muted_text
                            })
                            .child(
                                svg()
                                    .data(crate::icons::FILES_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child("Paste Here")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    if this.sftp_manager.clipboard.is_some() {
                                        this.sftp_close_context_menu(cx);
                                        this.sftp_paste(pane, cx);
                                    }
                                }),
                            ),
                    )
                    // Download (files only)
                    .children(if !menu.is_dir {
                        Some(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .cursor_pointer()
                                .hover(|s| s.bg(hover_bg))
                                .text_xs()
                                .text_color(text_color)
                                .child(
                                    svg()
                                        .data(crate::icons::ARROW_DOWN_SVG)
                                        .size(px(12.0))
                                        .text_color(muted_text),
                                )
                                .child("Download")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        this.sftp_close_context_menu(cx);
                                        this.sftp_download_file(pane, filename.clone(), cx);
                                    }),
                                ),
                        )
                    } else {
                        None
                    })
                    // 4. Copy Path
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(hover_bg))
                            .text_xs()
                            .text_color(text_color)
                            .child(
                                svg()
                                    .data(crate::icons::COPY_SVG)
                                    .size(px(12.0))
                                    .text_color(muted_text),
                            )
                            .child("Copy Path")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _window, cx| {
                                    this.sftp_copy_path(pane, &fn_copy, cx);
                                    this.sftp_close_context_menu(cx);
                                }),
                            ),
                    ),
            )
            .into_any_element(),
    )
}
