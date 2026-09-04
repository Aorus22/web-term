//! Unit tests for Connection modal form validation and Import/Export JSON compatibility.

use webterm::app_state::{ConnectionFormState, ConnectionModalMode};
use webterm::backend_client::{Connection, ImportResult};

#[test]
fn test_form_validation_missing_label() {
    let mut form = ConnectionFormState::new_create();
    form.host = "192.168.1.1".to_string();
    form.username = "root".to_string();
    let err = form.validate().unwrap_err();
    assert!(err.contains("label is required"));
}

#[test]
fn test_form_validation_missing_host() {
    let mut form = ConnectionFormState::new_create();
    form.label = "My Server".to_string();
    form.username = "root".to_string();
    let err = form.validate().unwrap_err();
    assert!(err.contains("Host / IP"));
}

#[test]
fn test_form_validation_missing_username() {
    let mut form = ConnectionFormState::new_create();
    form.label = "My Server".to_string();
    form.host = "192.168.1.1".to_string();
    form.username = "   ".to_string();
    let err = form.validate().unwrap_err();
    assert!(err.contains("Username is required"));
}

#[test]
fn test_form_validation_key_auth_requires_key_id() {
    let mut form = ConnectionFormState::new_create();
    form.label = "Prod Key Host".to_string();
    form.host = "10.0.0.1".to_string();
    form.username = "deploy".to_string();
    form.auth_method = "key".to_string();
    form.ssh_key_id = None;

    let err = form.validate().unwrap_err();
    assert!(err.contains("select an SSH key"));

    form.ssh_key_id = Some("key-123".to_string());
    assert!(form.validate().is_ok());
}

#[test]
fn test_form_validation_password_auth_success() {
    let mut form = ConnectionFormState::new_create();
    form.label = "Prod Password Host".to_string();
    form.host = "10.0.0.1".to_string();
    form.username = "deploy".to_string();
    form.auth_method = "password".to_string();
    form.password = "secret123".to_string();

    assert!(form.validate().is_ok());
}

#[test]
fn test_parse_port_defaults() {
    let mut form = ConnectionFormState::new_create();
    form.port = "".to_string();
    assert_eq!(form.parse_port(), 22);

    form.port = "invalid".to_string();
    assert_eq!(form.parse_port(), 22);

    form.port = " 2222 ".to_string();
    assert_eq!(form.parse_port(), 2222);
}

#[test]
fn test_parse_tags_cleaning() {
    let mut form = ConnectionFormState::new_create();
    form.tags = "  prod, web , , database  , infra  ".to_string();
    let tags = form.parse_tags();
    assert_eq!(tags, vec!["prod", "web", "database", "infra"]);
}

#[test]
fn test_to_create_request_mapping() {
    let mut form = ConnectionFormState::new_create();
    form.label = " Web Server ".to_string();
    form.host = " 1.2.3.4 ".to_string();
    form.port = "8022".to_string();
    form.username = " admin ".to_string();
    form.tags = "web, frontend".to_string();
    form.auth_method = "key".to_string();
    form.ssh_key_id = Some("key-prod".to_string());

    let req = form.to_create_request();
    assert_eq!(req.label, "Web Server");
    assert_eq!(req.host, "1.2.3.4");
    assert_eq!(req.port, 8022);
    assert_eq!(req.username, "admin");
    assert_eq!(req.tags, vec!["web", "frontend"]);
    assert_eq!(req.auth_method, "key");
    assert_eq!(req.ssh_key_id, Some("key-prod".to_string()));
    assert_eq!(req.password, None);
}

#[test]
fn test_to_update_request_mapping() {
    let mut form = ConnectionFormState::new_create();
    form.mode = ConnectionModalMode::Edit("conn-99".to_string());
    form.label = "Updated Server".to_string();
    form.host = "5.6.7.8".to_string();
    form.port = "22".to_string();
    form.username = "root".to_string();
    form.auth_method = "password".to_string();
    form.password = "newpassword".to_string();

    let req = form.to_update_request();
    assert_eq!(req.label, "Updated Server");
    assert_eq!(req.host, "5.6.7.8");
    assert_eq!(req.port, 22);
    assert_eq!(req.username, "root");
    assert_eq!(req.auth_method, "password");
    assert_eq!(req.password, Some("newpassword".to_string()));
    assert_eq!(req.ssh_key_id, None);
}

#[test]
fn test_connection_import_export_json_roundtrip() {
    let conns = vec![
        Connection {
            id: "c-1".to_string(),
            label: "Host A".to_string(),
            host: "10.0.0.1".to_string(),
            port: 22,
            username: "root".to_string(),
            tags: vec!["prod".to_string()],
            auth_method: "password".to_string(),
            ssh_key_id: None,
            created_at: Some("2026-01-01T00:00:00Z".to_string()),
            updated_at: Some("2026-01-01T00:00:00Z".to_string()),
        },
        Connection {
            id: "c-2".to_string(),
            label: "Host B".to_string(),
            host: "10.0.0.2".to_string(),
            port: 2200,
            username: "ubuntu".to_string(),
            tags: vec!["staging".to_string(), "k8s".to_string()],
            auth_method: "key".to_string(),
            ssh_key_id: Some("key-staging".to_string()),
            created_at: Some("2026-01-02T00:00:00Z".to_string()),
            updated_at: Some("2026-01-02T00:00:00Z".to_string()),
        },
    ];

    let json_str = serde_json::to_string(&conns).expect("serialize connections");
    let deserialized: Vec<Connection> =
        serde_json::from_str(&json_str).expect("deserialize connections");

    assert_eq!(conns, deserialized);
}

#[test]
fn test_import_result_deserialization() {
    let raw_json = r#"{"imported": 4, "skipped": 1}"#;
    let res: ImportResult = serde_json::from_str(raw_json).expect("deserialize ImportResult");
    assert_eq!(res.imported, 4);
    assert_eq!(res.skipped, 1);
}
