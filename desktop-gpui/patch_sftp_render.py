import io

p = r"crates\webterm\src\views\sftp.rs"
s = io.open(p, encoding="utf-8").read()

# 1. Upload File menu item -> file picker
s = s.replace(
    """                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, _cx| {
                    this.sftp_pane_mut(pane).show_actions_menu = false;
                })),""",
    """                .on_mouse_down(MouseButton::Left, cx.listener(move |this, _, _window, cx| {
                    this.sftp_upload_from_picker(pane, cx);
                })),""",
    1,
)

# 2. Context menu: add Cut / Copy / Paste Here / Download before Rename
s = s.replace(
    """                    // 4. Copy Path""",
    """                    // Cut
                    .child(
                        menu_row(|this, _window, cx| {
                            this.sftp_clipboard_copy(true, pane, cx);
                            this.sftp_manager.context_menu = None;
                        }, CUT_LABEL, is_dark, hover_bg, text_color, muted_text),
                    )
                    // Copy
                    .child(
                        menu_row(|this, _window, cx| {
                            this.sftp_clipboard_copy(false, pane, cx);
                            this.sftp_manager.context_menu = None;
                        }, COPY_LABEL, is_dark, hover_bg, text_color, muted_text),
                    )
                    // Paste Here (disabled without clipboard)
                    .child(if app.sftp_manager.clipboard.is_some() {
                        menu_row(move |this, _window, cx| {
                            this.sftp_manager.context_menu = None;
                            this.sftp_paste(pane, cx);
                        }, PASTE_LABEL, is_dark, hover_bg, text_color, muted_text)
                    } else {
                        disabled_menu_row(PASTE_LABEL, is_dark, muted_text)
                    })
                    // Download (files only)
                    .child(if !is_dir {
                        menu_row(move |this, _window, cx| {
                            this.sftp_manager.context_menu = None;
                            this.sftp_download_file(pane, filename.clone(), cx);
                        }, DOWNLOAD_LABEL, is_dark, hover_bg, text_color, muted_text)
                    } else {
                        disabled_menu_row(DOWNLOAD_LABEL, is_dark, muted_text)
                    })
                    // 4. Copy Path""",
    1,
)

io.open(p, "w", encoding="utf-8", newline="").write(s)
print("sftp.rs render hooks inserted (needs menu_row helpers + labels)")
