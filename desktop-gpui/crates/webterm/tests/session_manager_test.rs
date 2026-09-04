//! Unit and contract tests for TerminalSessionManager.

use webterm::session::{SessionStatus, TerminalSessionManager, TerminalTab};

#[test]
fn test_alloc_tab_ids_sequential() {
    let mut manager = TerminalSessionManager::new();
    assert_eq!(manager.alloc_tab_id(), 1);
    assert_eq!(manager.alloc_tab_id(), 2);
    assert_eq!(manager.alloc_tab_id(), 3);
}

#[test]
fn test_add_and_switch_tabs() {
    let mut manager = TerminalSessionManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.tab_count(), 0);

    let id1 = manager.alloc_tab_id();
    let tab1 = TerminalTab::new_headless(id1, "Tab 1", "local", None);
    assert_eq!(manager.add_tab(tab1), 0);
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_tab().unwrap().title, "Tab 1");

    let id2 = manager.alloc_tab_id();
    let tab2 = TerminalTab::new_headless(id2, "Tab 2", "ssh", Some("conn-1".into()));
    assert_eq!(manager.add_tab(tab2), 1);
    assert_eq!(manager.active_index(), 1);
    assert_eq!(manager.active_tab().unwrap().title, "Tab 2");

    let id3 = manager.alloc_tab_id();
    let tab3 = TerminalTab::new_headless(id3, "Tab 3", "local", None);
    assert_eq!(manager.add_tab(tab3), 2);
    assert_eq!(manager.tab_count(), 3);
    assert_eq!(manager.active_index(), 2);

    // Switch to tab 0
    assert!(manager.switch_tab(0));
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_tab().unwrap().title, "Tab 1");

    // Out-of-bounds switch fails safely
    assert!(!manager.switch_tab(10));
    assert_eq!(manager.active_index(), 0);
}

#[test]
fn test_close_tabs_and_index_clamping() {
    let mut manager = TerminalSessionManager::new();

    for i in 1..=3 {
        let id = manager.alloc_tab_id();
        manager.add_tab(TerminalTab::new_headless(
            id,
            format!("Tab {}", i),
            "local",
            None,
        ));
    }
    assert_eq!(manager.tab_count(), 3);

    // Set active to middle tab (index 1 = "Tab 2")
    manager.switch_tab(1);
    assert_eq!(manager.active_index(), 1);

    // Close middle tab -> index 1 now points to "Tab 3"
    let closed = manager.close_tab(1).expect("should close tab 1");
    assert_eq!(closed.title, "Tab 2");
    assert_eq!(manager.tab_count(), 2);
    assert_eq!(manager.active_index(), 1);
    assert_eq!(manager.active_tab().unwrap().title, "Tab 3");

    // Close the last tab (index 1) -> active_index clamps to 0 ("Tab 1")
    let closed = manager.close_tab(1).expect("should close last tab");
    assert_eq!(closed.title, "Tab 3");
    assert_eq!(manager.tab_count(), 1);
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_tab().unwrap().title, "Tab 1");

    // Close remaining tab -> empty list, active_index reset to 0
    let closed = manager.close_tab(0).expect("should close remaining tab");
    assert_eq!(closed.title, "Tab 1");
    assert!(manager.is_empty());
    assert_eq!(manager.active_index(), 0);
    assert!(manager.active_tab().is_none());

    // Closing on empty returns None
    assert!(manager.close_tab(0).is_none());
}

#[test]
fn test_cycle_tabs() {
    let mut manager = TerminalSessionManager::new();

    // Cycling empty manager returns 0
    assert_eq!(manager.cycle_next(), 0);
    assert_eq!(manager.cycle_prev(), 0);

    for i in 0..3 {
        let id = manager.alloc_tab_id();
        manager.add_tab(TerminalTab::new_headless(
            id,
            format!("Tab {}", i),
            "local",
            None,
        ));
    }
    // Started at index 2 (last added)
    manager.switch_tab(0);

    // Forward cycle (Ctrl+Tab)
    assert_eq!(manager.cycle_next(), 1);
    assert_eq!(manager.cycle_next(), 2);
    assert_eq!(manager.cycle_next(), 0); // Wrap around to start

    // Backward cycle (Ctrl+Shift+Tab)
    assert_eq!(manager.cycle_prev(), 2); // Wrap around to end
    assert_eq!(manager.cycle_prev(), 1);
    assert_eq!(manager.cycle_prev(), 0);
}

#[test]
fn test_jump_to_tabs() {
    let mut manager = TerminalSessionManager::new();

    for i in 0..4 {
        let id = manager.alloc_tab_id();
        manager.add_tab(TerminalTab::new_headless(
            id,
            format!("Server {}", i + 1),
            "ssh",
            None,
        ));
    }

    // Direct jump (Alt+1..9 mapping)
    assert!(manager.jump_to(2));
    assert_eq!(manager.active_index(), 2);
    assert_eq!(manager.active_tab().unwrap().title, "Server 3");

    assert!(manager.jump_to(0));
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_tab().unwrap().title, "Server 1");

    // Invalid jump index ignored
    assert!(!manager.jump_to(8));
    assert_eq!(manager.active_index(), 0);
}

#[test]
fn test_status_and_title_updates() {
    let mut manager = TerminalSessionManager::new();
    let id = manager.alloc_tab_id();
    manager.add_tab(TerminalTab::new_headless(id, "Initial Title", "ssh", None));

    assert_eq!(manager.active_tab().unwrap().status, SessionStatus::Connecting);

    // Update status to Connected
    manager.set_tab_status(id, SessionStatus::Connected);
    assert_eq!(manager.active_tab().unwrap().status, SessionStatus::Connected);
    assert!(manager.active_tab().unwrap().is_connected());

    // Update status to Disconnected with reason
    manager.set_tab_status(
        id,
        SessionStatus::Disconnected(Some("Connection reset by peer".into())),
    );
    assert_eq!(
        manager.active_tab().unwrap().status,
        SessionStatus::Disconnected(Some("Connection reset by peer".into()))
    );
    assert!(!manager.active_tab().unwrap().is_connected());

    // Update title
    manager.set_tab_title(id, "prod-db-01.us-east");
    assert_eq!(manager.active_tab().unwrap().title, "prod-db-01.us-east");
}
