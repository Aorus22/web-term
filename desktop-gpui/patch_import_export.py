import io

p = r"crates\webterm\src\app_state.rs"
s = io.open(p, encoding="utf-8").read()

s = s.replace(
    """        if self.sftp_manager.show_actions_menu {
            self.sftp_manager.show_actions_menu = false;
            cx.notify();
            return;
        }""",
    """        if self.sftp_manager.left_pane.show_actions_menu
            || self.sftp_manager.right_pane.show_actions_menu
        {
            self.sftp_manager.left_pane.show_actions_menu = false;
            self.sftp_manager.right_pane.show_actions_menu = false;
            cx.notify();
            return;
        }""",
)

start = s.index("    pub fn open_import_modal(")
alt_end = "    /// Open the delete-connection confirmation dialog."
end = s.index(alt_end, start)
new_fns = """    /// Import connections from a JSON file chosen with the native file
    /// picker (web parity: hidden file input + result alert).
    pub fn import_connections_from_file(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => {
                self.push_notification("Backend client is not connected".to_string(), true, cx);
                cx.notify();
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<Vec<Connection>, String>>();
        TOKIO_RT.spawn(async move {
            let picked = rfd::AsyncFileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
                .await;
            let Some(handle) = picked else {
                return; // user cancelled
            };
            match handle.read().await {
                Ok(bytes) => match serde_json::from_slice::<Vec<Connection>>(&bytes) {
                    Ok(conns) => {
                        let _ = tx.send(Ok(conns));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("Invalid JSON array: {e}")));
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to read file: {e}")));
                }
            }
        });

        let view_weak = cx.entity().downgrade();
        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    let conns = match res {
                        Ok(c) => c,
                        Err(e) => {
                            cx_handle.update(|cx: &mut App| {
                                if let Some(app) = view_weak.upgrade() {
                                    app.update(cx, |this, cx| {
                                        this.push_notification(e, true, cx);
                                        cx.notify();
                                    });
                                }
                            });
                            return;
                        }
                    };
                    let (tx2, mut rx2) =
                        tokio::sync::mpsc::unbounded_channel::<Result<ImportResult, String>>();
                    TOKIO_RT.spawn(async move {
                        match client.import_connections(&conns).await {
                            Ok(r) => {
                                let _ = tx2.send(Ok(r));
                            }
                            Err(e) => {
                                let _ = tx2.send(Err(e.to_string()));
                            }
                        }
                    });
                    if let Some(res2) = rx2.recv().await {
                        cx_handle.update(|cx: &mut App| {
                            if let Some(app) = view_weak.upgrade() {
                                app.update(cx, |this, cx| {
                                    match res2 {
                                        Ok(result) => {
                                            this.push_notification(
                                                format!(
                                                    "Import finished: {} imported, {} skipped",
                                                    result.imported, result.skipped
                                                ),
                                                false,
                                                cx,
                                            );
                                            this.fetch_connections(cx);
                                        }
                                        Err(e) => {
                                            this.push_notification(
                                                format!("Import error: {e}"),
                                                true,
                                                cx,
                                            );
                                        }
                                    }
                                    cx.notify();
                                });
                            }
                        });
                    }
                }
            }
        })
        .detach();
    }

"""
s = s[:start] + new_fns + s[end:]

start = s.index("    pub fn export_connections_to_disk(")
end = s.index("    /// Import connections from a JSON file", start)
new_export = """    pub fn export_connections_to_disk(&mut self, cx: &mut Context<Self>) {
        let client = match self.client.clone() {
            Some(c) => c,
            None => return,
        };

        let view_weak = cx.entity().downgrade();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Result<usize, String>>();

        TOKIO_RT.spawn(async move {
            match client.export_connections().await {
                Ok(conns) => {
                    let count = conns.len();
                    match serde_json::to_string_pretty(&conns) {
                        Ok(json_str) => {
                            let picked = rfd::AsyncFileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("webterm-connections-export.json")
                                .save_file()
                                .await;
                            let Some(handle) = picked else {
                                return; // user cancelled
                            };
                            match handle.write(json_str.as_bytes()).await {
                                Ok(_) => {
                                    let _ = tx.send(Ok(count));
                                }
                                Err(e) => {
                                    let _ =
                                        tx.send(Err(format!("Failed to write export file: {e}")));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(format!("Failed to serialize connections: {e}")));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });

        cx.spawn(move |_view, cx: &mut AsyncApp| {
            let view_weak = view_weak.clone();
            let cx_handle = cx.clone();
            async move {
                if let Some(res) = rx.recv().await {
                    cx_handle.update(|cx: &mut App| {
                        if let Some(app) = view_weak.upgrade() {
                            app.update(cx, |this, cx| {
                                match res {
                                    Ok(count) => {
                                        this.push_notification(
                                            format!("Exported {count} connection(s)"),
                                            false,
                                            cx,
                                        );
                                    }
                                    Err(e) => {
                                        this.push_notification(
                                            format!("Export failed: {e}"),
                                            true,
                                            cx,
                                        );
                                    }
                                }
                                cx.notify();
                            });
                        }
                    });
                }
            }
        })
        .detach();
    }

"""
s = s[:start] + new_export + s[end:]

io.open(p, "w", encoding="utf-8", newline="").write(s)
print("import/export rewritten")

p = r"crates\webterm\src\views\hosts.rs"
s = io.open(p, encoding="utf-8").read()
s = s.replace("this.open_import_modal(cx);", "this.import_connections_from_file(cx);")
io.open(p, "w", encoding="utf-8", newline="").write(s)
print("hosts import button rewired")
