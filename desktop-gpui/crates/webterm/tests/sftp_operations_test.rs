//! Unit tests for SFTP file operations: folder creation, rename, delete modals and transfer tracking.

use webterm::app_state::{
    format_file_size, SftpActivePane, SftpContextMenu, SftpDraggedItem, SftpModalState,
    SftpPaneState, SftpTransferItem,
};
use webterm::backend_client::SftpTransferStatus;

#[test]
fn test_new_folder_modal_state_and_validation() {
    let mut modal = SftpModalState::NewFolder {
        pane: SftpActivePane::Left,
        name: String::new(),
        error: None,
    };

    // Simulate empty name error validation
    if let SftpModalState::NewFolder { name, error, .. } = &mut modal {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            *error = Some("Folder name cannot be empty".to_string());
        }
    }

    match &modal {
        SftpModalState::NewFolder { error, .. } => {
            assert_eq!(error.as_deref(), Some("Folder name cannot be empty"));
        }
        _ => panic!("Expected NewFolder modal"),
    }

    // Set valid name and clear error
    if let SftpModalState::NewFolder { name, error, .. } = &mut modal {
        *name = "  documents  ".to_string();
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            *error = None;
        }
        assert_eq!(trimmed, "documents");
    }

    match &modal {
        SftpModalState::NewFolder { name, error, .. } => {
            assert_eq!(name, "  documents  ");
            assert!(error.is_none());
        }
        _ => panic!("Expected NewFolder modal"),
    }
}

#[test]
fn test_rename_modal_validation() {
    let mut modal = SftpModalState::Rename {
        pane: SftpActivePane::Right,
        old_name: "original.txt".to_string(),
        new_name: "".to_string(),
        error: None,
    };

    // 1. Empty new name -> error
    if let SftpModalState::Rename {
        new_name, error, ..
    } = &mut modal
    {
        if new_name.trim().is_empty() {
            *error = Some("New name cannot be empty".to_string());
        }
    }
    match &modal {
        SftpModalState::Rename { error, .. } => {
            assert_eq!(error.as_deref(), Some("New name cannot be empty"));
        }
        _ => panic!("Expected Rename modal"),
    }

    // 2. Same name -> no-op / dismiss modal
    if let SftpModalState::Rename {
        old_name,
        new_name,
        error,
        ..
    } = &mut modal
    {
        *new_name = "original.txt".to_string();
        *error = None;
        assert_eq!(new_name.trim(), old_name);
    }

    // 3. Valid different new name
    if let SftpModalState::Rename {
        old_name,
        new_name,
        error,
        ..
    } = &mut modal
    {
        *new_name = "renamed.txt".to_string();
        *error = None;
        assert_ne!(new_name.trim(), old_name);
    }
}

#[test]
fn test_delete_modal_state_targets() {
    let targets = vec!["file1.txt".to_string(), "folderA".to_string()];
    let modal = SftpModalState::DeleteConfirm {
        pane: SftpActivePane::Left,
        targets: targets.clone(),
        error: None,
    };

    match modal {
        SftpModalState::DeleteConfirm {
            pane,
            targets: modal_targets,
            error,
        } => {
            assert_eq!(pane, SftpActivePane::Left);
            assert_eq!(modal_targets.len(), 2);
            assert_eq!(modal_targets[0], "file1.txt");
            assert_eq!(modal_targets[1], "folderA");
            assert!(error.is_none());
        }
        _ => panic!("Expected DeleteConfirm modal"),
    }
}

#[test]
fn test_pane_selection_and_clearing() {
    let mut pane = SftpPaneState::new("local", "Local Filesystem", "/test");

    pane.selected.insert("file_a.txt".to_string());
    pane.selected.insert("file_b.txt".to_string());
    assert_eq!(pane.selected.len(), 2);

    // Selection clear
    pane.selected.clear();
    assert!(pane.selected.is_empty());
}

#[test]
fn test_transfer_progress_calculation() {
    let item = SftpTransferItem {
        delete_source_after: None,
        id: "tx-1".to_string(),
        name: "video.mp4".to_string(),
        from_source: "local".to_string(),
        to_source: "conn-1".to_string(),
        bytes_transferred: 50 * 1024 * 1024,
        total_bytes: 100 * 1024 * 1024,
        status: "transferring".to_string(),
        error: None,
    };

    let pct = if item.total_bytes > 0 {
        (item.bytes_transferred as f64 / item.total_bytes as f64 * 100.0) as u32
    } else {
        0
    };
    assert_eq!(pct, 50);
    assert_eq!(format_file_size(item.bytes_transferred), "50.0 MB");
    assert_eq!(format_file_size(item.total_bytes), "100.0 MB");
    assert_eq!(item.status, "transferring");
}

#[test]
fn test_transfer_status_conversion() {
    let status = SftpTransferStatus {
        id: "tx-99".to_string(),
        bytes_transferred: 1024 * 1024,
        total_bytes: 1024 * 1024 * 4,
        status: "completed".to_string(),
        error: None,
    };

    let item = SftpTransferItem::from(status);
    assert_eq!(item.id, "tx-99");
    assert_eq!(item.bytes_transferred, 1048576);
    assert_eq!(item.total_bytes, 4194304);
    assert_eq!(item.status, "completed");
    assert!(item.error.is_none());
}

#[test]
fn test_context_menu_state_and_dragged_item() {
    let menu = SftpContextMenu {
        pane: SftpActivePane::Left,
        filename: "document.pdf".to_string(),
        is_dir: false,
        position: (150.0, 300.0),
    };
    assert_eq!(menu.pane, SftpActivePane::Left);
    assert_eq!(menu.filename, "document.pdf");
    assert!(!menu.is_dir);
    assert_eq!(menu.position, (150.0, 300.0));

    let drag = SftpDraggedItem {
        source_pane: SftpActivePane::Right,
        filenames: vec!["file1.txt".to_string(), "file2.txt".to_string()],
    };
    assert_eq!(drag.source_pane, SftpActivePane::Right);
    assert_eq!(drag.filenames.len(), 2);
    assert_eq!(drag.filenames[0], "file1.txt");
    assert_eq!(drag.filenames[1], "file2.txt");
}
