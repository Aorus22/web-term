//! Unit tests for SFTP dual-pane view helpers, breadcrumbs, path calculations, and sorting/filtering.

use webterm::app_state::{
    format_file_size, join_path, parent_path, split_breadcrumbs, SftpPaneState, SftpSortColumn,
    SftpSortOrder,
};
use webterm::backend_client::SftpFileInfo;

#[test]
fn test_split_breadcrumbs_unix_and_windows() {
    // Empty or dot
    assert_eq!(split_breadcrumbs(""), vec![(".".to_string(), ".".to_string())]);
    assert_eq!(split_breadcrumbs("."), vec![(".".to_string(), ".".to_string())]);

    // Unix root
    assert_eq!(split_breadcrumbs("/"), vec![("/".to_string(), "/".to_string())]);

    // Unix absolute path
    let unix_crumbs = split_breadcrumbs("/home/user/documents");
    assert_eq!(
        unix_crumbs,
        vec![
            ("/".to_string(), "/".to_string()),
            ("home".to_string(), "/home".to_string()),
            ("user".to_string(), "/home/user".to_string()),
            ("documents".to_string(), "/home/user/documents".to_string()),
        ]
    );

    // Windows absolute path
    let win_crumbs = split_breadcrumbs("C:\\Users\\admin\\Desktop");
    assert_eq!(
        win_crumbs,
        vec![
            ("C:".to_string(), "C:/".to_string()),
            ("Users".to_string(), "C:/Users".to_string()),
            ("admin".to_string(), "C:/Users/admin".to_string()),
            ("Desktop".to_string(), "C:/Users/admin/Desktop".to_string()),
        ]
    );

    // Relative path
    let rel_crumbs = split_breadcrumbs("folder/subfolder");
    assert_eq!(
        rel_crumbs,
        vec![
            ("folder".to_string(), "folder".to_string()),
            ("subfolder".to_string(), "folder/subfolder".to_string()),
        ]
    );
}

#[test]
fn test_parent_path_navigation() {
    assert_eq!(parent_path("/"), "/");
    assert_eq!(parent_path("/home"), "/");
    assert_eq!(parent_path("/home/user/docs"), "/home/user");
    assert_eq!(parent_path("C:/Users/test"), "C:/Users");
    assert_eq!(parent_path("C:/"), "/");
    assert_eq!(parent_path("."), ".");
}

#[test]
fn test_join_path() {
    assert_eq!(join_path(".", "file.txt"), "file.txt");
    assert_eq!(join_path("", "file.txt"), "file.txt");
    assert_eq!(join_path("/var/log", "app.log"), "/var/log/app.log");
    assert_eq!(join_path("/var/log/", "app.log"), "/var/log/app.log");
    assert_eq!(join_path("C:/Users", "file.txt"), "C:/Users/file.txt");
}

#[test]
fn test_format_file_size() {
    assert_eq!(format_file_size(-1), "-");
    assert_eq!(format_file_size(0), "0 B");
    assert_eq!(format_file_size(512), "512 B");
    assert_eq!(format_file_size(1024), "1.0 KB");
    assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
    assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.0 GB");
}

fn sample_files() -> Vec<SftpFileInfo> {
    vec![
        SftpFileInfo {
            name: "zebra.txt".to_string(),
            size: 500,
            mode: 0o644,
            mod_time: "2026-09-01T10:00:00Z".to_string(),
            is_dir: false,
        },
        SftpFileInfo {
            name: "alpha.txt".to_string(),
            size: 2500,
            mode: 0o644,
            mod_time: "2026-09-03T12:00:00Z".to_string(),
            is_dir: false,
        },
        SftpFileInfo {
            name: ".hidden_file".to_string(),
            size: 100,
            mode: 0o644,
            mod_time: "2026-09-02T08:00:00Z".to_string(),
            is_dir: false,
        },
        SftpFileInfo {
            name: "beta_folder".to_string(),
            size: 4096,
            mode: 0o755,
            mod_time: "2026-09-04T15:00:00Z".to_string(),
            is_dir: true,
        },
        SftpFileInfo {
            name: ".git".to_string(),
            size: 4096,
            mode: 0o755,
            mod_time: "2026-09-01T00:00:00Z".to_string(),
            is_dir: true,
        },
    ]
}

#[test]
fn test_directories_first_and_name_sorting() {
    let mut pane = SftpPaneState::new("local", "Local", "/test");
    pane.files = sample_files();
    pane.show_hidden = false;
    pane.sort_column = SftpSortColumn::Name;
    pane.sort_order = SftpSortOrder::Ascending;

    let visible = pane.visible_files();
    // Hidden files filtered out (.hidden_file, .git)
    // Directories must come first: beta_folder, then alpha.txt, zebra.txt
    assert_eq!(visible.len(), 3);
    assert_eq!(visible[0].name, "beta_folder");
    assert!(visible[0].is_dir);
    assert_eq!(visible[1].name, "alpha.txt");
    assert!(!visible[1].is_dir);
    assert_eq!(visible[2].name, "zebra.txt");
    assert!(!visible[2].is_dir);

    // Descending Name
    pane.sort_order = SftpSortOrder::Descending;
    let visible_desc = pane.visible_files();
    assert_eq!(visible_desc.len(), 3);
    // Folder still first even in descending order
    assert_eq!(visible_desc[0].name, "beta_folder");
    assert_eq!(visible_desc[1].name, "zebra.txt");
    assert_eq!(visible_desc[2].name, "alpha.txt");
}

#[test]
fn test_show_hidden_files() {
    let mut pane = SftpPaneState::new("local", "Local", "/test");
    pane.files = sample_files();

    pane.show_hidden = false;
    assert_eq!(pane.visible_files().len(), 3);

    pane.show_hidden = true;
    let visible = pane.visible_files();
    assert_eq!(visible.len(), 5);
    // Both folders (.git, beta_folder) must be on top
    assert!(visible[0].is_dir);
    assert!(visible[1].is_dir);
}

#[test]
fn test_size_and_modtime_sorting() {
    let mut pane = SftpPaneState::new("local", "Local", "/test");
    pane.files = sample_files();
    pane.show_hidden = false;

    // Size Ascending (directories first, then smallest file to largest)
    pane.sort_column = SftpSortColumn::Size;
    pane.sort_order = SftpSortOrder::Ascending;
    let by_size = pane.visible_files();
    assert_eq!(by_size[0].name, "beta_folder");
    assert_eq!(by_size[1].name, "zebra.txt"); // 500 B
    assert_eq!(by_size[2].name, "alpha.txt"); // 2500 B

    // ModTime Descending (newest files first)
    pane.sort_column = SftpSortColumn::ModTime;
    pane.sort_order = SftpSortOrder::Descending;
    let by_time = pane.visible_files();
    assert_eq!(by_time[0].name, "beta_folder"); // Folder first
    assert_eq!(by_time[1].name, "alpha.txt"); // 2026-09-03
    assert_eq!(by_time[2].name, "zebra.txt"); // 2026-09-01
}

#[test]
fn test_search_query_filtering() {
    let mut pane = SftpPaneState::new("local", "Local", "/test");
    pane.files = sample_files();
    pane.show_hidden = true;

    pane.search_query = "ALPHA".to_string(); // case-insensitive
    let res = pane.visible_files();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "alpha.txt");

    pane.search_query = "folder".to_string();
    let res_folder = pane.visible_files();
    assert_eq!(res_folder.len(), 1);
    assert_eq!(res_folder[0].name, "beta_folder");
}
