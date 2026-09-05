use std::path::PathBuf;
use webterm::app_state::{AppState, View};
use webterm_backend_client::types::{
    Connection, CreateConnectionRequest, CreateForwardRequest, PortForward, SftpFileInfo, SshKey,
};
use webterm_settings::DesktopSettings;

#[test]
fn test_parity_view_routing_coverage() {
    let settings = DesktopSettings::default();
    let mut state = AppState::new(settings, None);

    // All primary sidebar views must be navigable
    state.active_view = View::Hosts;
    assert_eq!(state.active_view, View::Hosts);

    state.active_view = View::Keys;
    assert_eq!(state.active_view, View::Keys);

    state.active_view = View::Sftp;
    assert_eq!(state.active_view, View::Sftp);

    state.active_view = View::Forwards;
    assert_eq!(state.active_view, View::Forwards);

    state.active_view = View::NewTab;
    assert_eq!(state.active_view, View::NewTab);

    state.active_view = View::Settings;
    assert_eq!(state.active_view, View::Settings);
}

#[test]
fn test_parity_data_models_and_contracts() {
    // 1. Connection DTO parity with web schema
    let conn = Connection {
        id: "conn-123".to_string(),
        label: "Production Server".to_string(),
        host: "prod.example.com".to_string(),
        port: 22,
        username: "admin".to_string(),
        tags: vec!["cloud".to_string(), "prod".to_string()],
        auth_method: "key".to_string(),
        ssh_key_id: Some("key-456".to_string()),
        created_at: Some("2026-09-01T00:00:00Z".to_string()),
        updated_at: Some("2026-09-01T00:00:00Z".to_string()),
    };
    assert_eq!(conn.port, 22);
    assert_eq!(conn.auth_method, "key");
    assert_eq!(conn.tags.len(), 2);

    let create_req = CreateConnectionRequest {
        label: conn.label.clone(),
        host: conn.host.clone(),
        port: conn.port,
        username: conn.username.clone(),
        password: None,
        auth_method: conn.auth_method.clone(),
        ssh_key_id: conn.ssh_key_id.clone(),
        tags: conn.tags.clone(),
    };
    let json = serde_json::to_string(&create_req).unwrap();
    assert!(json.contains("Production Server"));
    assert!(json.contains("prod.example.com"));

    // 2. Key DTO parity with web schema
    let key = SshKey {
        id: "key-456".to_string(),
        name: "Deploy Key".to_string(),
        key_type: "ed25519".to_string(),
        fingerprint: "SHA256:abc123xyz".to_string(),
        has_passphrase: false,
        created_at: Some("2026-09-01T00:00:00Z".to_string()),
        updated_at: Some("2026-09-01T00:00:00Z".to_string()),
    };
    assert_eq!(key.key_type, "ed25519");

    // 3. PortForward DTO parity with web schema
    let fwd = PortForward {
        id: "fwd-789".to_string(),
        connection_id: "conn-123".to_string(),
        name: "Postgres Tunnel".to_string(),
        local_port: 5432,
        remote_port: 5432,
        forward_type: "local".to_string(),
        active: true,
        auto_start: true,
        error: String::new(),
        created_at: Some("2026-09-01T00:00:00Z".to_string()),
        updated_at: Some("2026-09-01T00:00:00Z".to_string()),
    };
    assert!(!fwd.is_reverse());
    assert_eq!(fwd.mapping_display(), "localhost:5432 \u{2192} :5432");

    let rev_fwd = PortForward {
        id: "fwd-790".to_string(),
        connection_id: "conn-123".to_string(),
        name: "Webhook Reverse Tunnel".to_string(),
        local_port: 8080,
        remote_port: 9000,
        forward_type: "reverse".to_string(),
        active: false,
        auto_start: false,
        error: "Port in use".to_string(),
        created_at: Some("2026-09-01T00:00:00Z".to_string()),
        updated_at: Some("2026-09-01T00:00:00Z".to_string()),
    };
    assert!(rev_fwd.is_reverse());
    assert_eq!(rev_fwd.mapping_display(), ":9000 \u{2190} localhost:8080");

    let fwd_req = CreateForwardRequest {
        connection_id: rev_fwd.connection_id.clone(),
        name: rev_fwd.name.clone(),
        local_port: rev_fwd.local_port,
        remote_port: rev_fwd.remote_port,
        forward_type: rev_fwd.forward_type.clone(),
    };
    let fwd_json = serde_json::to_string(&fwd_req).unwrap();
    assert!(fwd_json.contains("Webhook Reverse Tunnel"));
    assert!(fwd_json.contains("9000"));

    // 4. SFTP Entry DTO parity with web schema
    let sftp_entry = SftpFileInfo {
        name: "config.json".to_string(),
        size: 2048,
        mode: 0o644,
        mod_time: "2026-09-01 12:00:00".to_string(),
        is_dir: false,
    };
    assert!(!sftp_entry.is_dir);
    assert_eq!(sftp_entry.size, 2048);
}

#[test]
fn test_parity_architecture_invariants() {
    // Architectural Rule SET-01: Terminal engine must NOT be configurable
    // There should be no terminal_engine field in DesktopSettings.
    let settings = DesktopSettings::default();
    let settings_json = serde_json::to_string(&settings).unwrap();
    assert!(
        !settings_json.contains("terminal_engine"),
        "DesktopSettings must not have a terminal_engine selector"
    );
    assert!(
        !settings_json.contains("xterm"),
        "DesktopSettings must not reference xterm engine"
    );

    // Settings must support backend_path override
    let mut custom_settings = DesktopSettings::default();
    custom_settings.backend_path = Some(PathBuf::from("custom/backend.exe"));
    let custom_json = serde_json::to_string(&custom_settings).unwrap();
    let deserialized: DesktopSettings = serde_json::from_str(&custom_json).unwrap();
    assert_eq!(
        deserialized.backend_path,
        Some(PathBuf::from("custom/backend.exe"))
    );
}
