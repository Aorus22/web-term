//! Unit tests for Port Forward form validation and display helpers.

use webterm::app_state::{ForwardFormState, ForwardModalMode};
use webterm_backend_client::PortForward;

#[test]
fn test_forward_form_create_defaults() {
    let form = ForwardFormState::new_create(Some("conn-123".to_string()));
    assert_eq!(form.mode, ForwardModalMode::Create);
    assert_eq!(form.connection_id, "conn-123");
    assert_eq!(form.forward_type, "local");
    assert!(form.name.is_empty());
    assert!(form.local_port.is_empty());
    assert!(form.remote_port.is_empty());
    assert!(form.error_message.is_none());
}

#[test]
fn test_forward_form_edit_initialization() {
    let pf = PortForward {
        id: "f-456".to_string(),
        name: "Remote Postgres".to_string(),
        connection_id: "conn-999".to_string(),
        local_port: 5433,
        remote_port: 5432,
        forward_type: "reverse".to_string(),
        active: true,
        auto_start: true,
        error: String::new(),
        created_at: None,
        updated_at: None,
    };

    let form = ForwardFormState::new_edit(&pf);
    assert_eq!(form.mode, ForwardModalMode::Edit("f-456".to_string()));
    assert_eq!(form.name, "Remote Postgres");
    assert_eq!(form.connection_id, "conn-999");
    assert_eq!(form.local_port, "5433");
    assert_eq!(form.remote_port, "5432");
    assert_eq!(form.forward_type, "reverse");
}

#[test]
fn test_forward_form_port_parsing() {
    let mut form = ForwardFormState::new_create(Some("c1".to_string()));

    // Invalid local port non-numeric
    form.local_port = "abc".to_string();
    assert!(form.parse_local_port().is_err());

    // Invalid port 0
    form.local_port = "0".to_string();
    assert!(form.parse_local_port().is_err());

    // Invalid port > 65535
    form.local_port = "70000".to_string();
    assert!(form.parse_local_port().is_err());

    // Valid local port
    form.local_port = " 8080 ".to_string();
    assert_eq!(form.parse_local_port().unwrap(), 8080);

    // Invalid remote port
    form.remote_port = "-1".to_string();
    assert!(form.parse_remote_port().is_err());

    // Valid remote port
    form.remote_port = "443".to_string();
    assert_eq!(form.parse_remote_port().unwrap(), 443);
}

#[test]
fn test_forward_form_validation_and_request_conversion() {
    let mut form = ForwardFormState::new_create(None);

    // Empty name fails
    assert!(form.validate().is_err());
    assert!(form.to_create_request().is_err());

    // Empty connection fails
    form.name = "Web Dev".to_string();
    assert!(form.validate().is_err());

    // Invalid ports fail
    form.connection_id = "c-1".to_string();
    form.local_port = "".to_string();
    form.remote_port = "3000".to_string();
    assert!(form.validate().is_err());

    // Valid local forward succeeds
    form.local_port = "3000".to_string();
    assert!(form.validate().is_ok());

    let create_req = form.to_create_request().expect("should build create request");
    assert_eq!(create_req.name, "Web Dev");
    assert_eq!(create_req.connection_id, "c-1");
    assert_eq!(create_req.local_port, 3000);
    assert_eq!(create_req.remote_port, 3000);
    assert_eq!(create_req.forward_type, "local");

    let update_req = form.to_update_request().expect("should build update request");
    assert_eq!(update_req.name, "Web Dev");
    assert_eq!(update_req.forward_type, "local");

    // Invalid forward type
    form.forward_type = "invalid".to_string();
    assert!(form.validate().is_err());
}

#[test]
fn test_port_forward_mapping_display() {
    let local_forward = PortForward {
        id: "f-1".to_string(),
        name: "Local Web".to_string(),
        connection_id: "c-1".to_string(),
        local_port: 8080,
        remote_port: 80,
        forward_type: "local".to_string(),
        active: true,
        auto_start: false,
        error: String::new(),
        created_at: None,
        updated_at: None,
    };
    assert!(!local_forward.is_reverse());
    assert_eq!(local_forward.mapping_display(), "localhost:8080 \u{2192} :80");

    let reverse_forward = PortForward {
        id: "f-2".to_string(),
        name: "Reverse Tunnel".to_string(),
        connection_id: "c-1".to_string(),
        local_port: 5000,
        remote_port: 15000,
        forward_type: "reverse".to_string(),
        active: false,
        auto_start: false,
        error: String::new(),
        created_at: None,
        updated_at: None,
    };
    assert!(reverse_forward.is_reverse());
    assert_eq!(reverse_forward.mapping_display(), ":15000 \u{2190} localhost:5000");
}
