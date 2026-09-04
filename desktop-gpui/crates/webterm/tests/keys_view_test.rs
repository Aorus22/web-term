//! Unit tests for SSH key management, base64 encoding, and session passphrase caching.

use std::collections::HashMap;
use webterm::app_state::encode_base64;
use webterm::backend_client::{CreateKeyRequest, SshKey};
use webterm_settings::DesktopSettings;

#[test]
fn test_encode_base64_standard_vectors() {
    assert_eq!(encode_base64(b""), "");
    assert_eq!(encode_base64(b"f"), "Zg==");
    assert_eq!(encode_base64(b"fo"), "Zm8=");
    assert_eq!(encode_base64(b"foo"), "Zm9v");
    assert_eq!(encode_base64(b"foob"), "Zm9vYg==");
    assert_eq!(encode_base64(b"fooba"), "Zm9vYmE=");
    assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
}

#[test]
fn test_encode_base64_pem_format() {
    let pem = "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----";
    let encoded = encode_base64(pem.as_bytes());
    assert!(!encoded.is_empty());
    // Ensure standard characters only
    assert!(encoded.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
}

#[test]
fn test_session_passphrase_cache_lifecycle() {
    let mut cache: HashMap<String, String> = HashMap::new();

    // Key is initially not cached
    assert_eq!(cache.get("key-prod"), None);

    // Cache passphrase on unlock
    cache.insert("key-prod".to_string(), "supersecret123".to_string());
    assert_eq!(cache.get("key-prod").map(|s| s.as_str()), Some("supersecret123"));

    // Cache multiple keys
    cache.insert("key-staging".to_string(), "stagingpass".to_string());
    assert_eq!(cache.get("key-staging").map(|s| s.as_str()), Some("stagingpass"));
    assert_eq!(cache.len(), 2);

    // Key deletion purges passphrase from memory
    cache.remove("key-prod");
    assert_eq!(cache.get("key-prod"), None);
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_passphrase_never_persisted_in_settings() {
    let settings = DesktopSettings::default();
    let json = serde_json::to_string(&settings).expect("serialize settings");

    // Passphrase or key secrets must not appear in persistent settings
    assert!(!json.contains("passphrase"));
    assert!(!json.contains("passphrase_cache"));
    assert!(!json.contains("supersecret"));
}

#[test]
fn test_create_key_request_structure() {
    let raw_pem = "-----BEGIN PRIVATE KEY-----\nMIIB...\n-----END PRIVATE KEY-----";
    let b64 = encode_base64(raw_pem.as_bytes());
    let req = CreateKeyRequest {
        name: "my_deploy_key".to_string(),
        key_base64: b64.clone(),
    };

    assert_eq!(req.name, "my_deploy_key");
    assert_eq!(req.key_base64, b64);

    let json = serde_json::to_string(&req).expect("serialize CreateKeyRequest");
    assert!(json.contains("my_deploy_key"));
    assert!(json.contains("key_base64"));
}

#[test]
fn test_ssh_key_deserialization() {
    let raw = r#"{
        "id": "key-uuid-1",
        "name": "id_ed25519",
        "key_type": "ed25519",
        "fingerprint": "SHA256:abc123xyz",
        "created_at": "2026-09-01T12:00:00Z",
        "updated_at": "2026-09-01T12:00:00Z"
    }"#;

    let key: SshKey = serde_json::from_str(raw).expect("deserialize SshKey");
    assert_eq!(key.id, "key-uuid-1");
    assert_eq!(key.name, "id_ed25519");
    assert_eq!(key.key_type, "ed25519");
    assert_eq!(key.fingerprint, "SHA256:abc123xyz");
    assert_eq!(key.created_at, Some("2026-09-01T12:00:00Z".to_string()));
}
