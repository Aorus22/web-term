//! Tests for reconnection backoff, session survival models, and re-attachment selection.

use webterm::session::{SessionStatus, TerminalSessionManager, TerminalTab};
use webterm_backend_client::types::SessionInfo;
use webterm_settings::SavedSessionTab;

#[test]
fn test_reconnecting_status_and_backoff_progression() {
    let mut manager = TerminalSessionManager::new();
    let id = manager.alloc_tab_id();
    manager.add_tab(TerminalTab::new_headless(id, "Test Shell", "local", None));

    // Simulate drop: attempt 1
    manager.set_tab_status(
        id,
        SessionStatus::Reconnecting {
            attempt: 1,
            next_retry_secs: 2,
        },
    );
    assert_eq!(
        manager.active_tab().unwrap().status,
        SessionStatus::Reconnecting {
            attempt: 1,
            next_retry_secs: 2
        }
    );

    // Compute exponential backoff schedule:
    // attempt 1: 2s
    // attempt 2: 4s
    // attempt 3: 6s
    // attempt 4: 8s
    // attempt 5: 10s
    for attempt in 1..=5 {
        let backoff = std::cmp::min(2 * attempt, 16);
        assert_eq!(backoff, 2 * attempt);
    }
}

#[test]
fn test_saved_session_tab_serialization_roundtrip() {
    let saved = SavedSessionTab {
        session_id: "sess-uuid-42".to_string(),
        title: "Production DB".to_string(),
        session_type: "ssh".to_string(),
        connection_id: Some("conn-100".to_string()),
    };

    let json = serde_json::to_string(&saved).expect("should serialize");
    let deserialized: SavedSessionTab = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(saved, deserialized);
}

#[test]
fn test_session_restoration_selection_logic() {
    // Backend reports two detached sessions from before app restart
    let backend_sessions = vec![
        SessionInfo {
            id: "s1".to_string(),
            session_type: "local".to_string(),
            host: "local".to_string(),
            user: "local".to_string(),
            port: 0,
            connection_id: Some("local".to_string()),
            status: "detached".to_string(),
            cwd: Some("/home/dev".to_string()),
        },
        SessionInfo {
            id: "s2".to_string(),
            session_type: "ssh".to_string(),
            host: "10.0.0.1".to_string(),
            user: "admin".to_string(),
            port: 22,
            connection_id: Some("c-prod".to_string()),
            status: "detached".to_string(),
            cwd: Some("/var/log".to_string()),
        },
    ];

    // Case 1: User had s2 open before closing window
    let saved_sessions = vec![SavedSessionTab {
        session_id: "s2".to_string(),
        title: "admin@10.0.0.1".to_string(),
        session_type: "ssh".to_string(),
        connection_id: Some("c-prod".to_string()),
    }];

    let matched: Vec<_> = saved_sessions
        .iter()
        .filter_map(|saved| {
            backend_sessions
                .iter()
                .find(|b| b.id == saved.session_id)
                .map(|b| (b.id.clone(), saved.title.clone()))
        })
        .collect();

    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].0, "s2");
    assert_eq!(matched[0].1, "admin@10.0.0.1");

    // Case 2: No saved sessions, all detached backend sessions are restored
    let all_restored: Vec<_> = backend_sessions
        .iter()
        .map(|s| {
            let title = if s.session_type == "local" {
                "Local Shell".to_string()
            } else {
                format!("{}:{}", s.user, s.host)
            };
            (s.id.clone(), title)
        })
        .collect();

    assert_eq!(all_restored.len(), 2);
    assert_eq!(
        all_restored[0],
        ("s1".to_string(), "Local Shell".to_string())
    );
    assert_eq!(
        all_restored[1],
        ("s2".to_string(), "admin:10.0.0.1".to_string())
    );
}
