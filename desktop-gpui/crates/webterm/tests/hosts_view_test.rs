//! Unit tests for Hosts view filtering, tag extraction, and search logic.

use webterm::backend_client::Connection;
use webterm::views::hosts::{extract_unique_tags, filter_connections};

fn sample_connections() -> Vec<Connection> {
    vec![
        Connection {
            id: "conn-1".to_string(),
            label: "Production DB".to_string(),
            host: "10.0.0.5".to_string(),
            port: 22,
            username: "postgres".to_string(),
            tags: vec!["prod".to_string(), "database".to_string()],
            auth_method: "password".to_string(),
            ssh_key_id: None,
            created_at: Some("2026-01-01T00:00:00Z".to_string()),
            updated_at: Some("2026-01-01T00:00:00Z".to_string()),
        },
        Connection {
            id: "conn-2".to_string(),
            label: "Staging Web API".to_string(),
            host: "api-staging.internal".to_string(),
            port: 2222,
            username: "ubuntu".to_string(),
            tags: vec!["staging".to_string(), "web".to_string()],
            auth_method: "key".to_string(),
            ssh_key_id: Some("key-staging".to_string()),
            created_at: Some("2026-01-02T00:00:00Z".to_string()),
            updated_at: Some("2026-01-02T00:00:00Z".to_string()),
        },
        Connection {
            id: "conn-3".to_string(),
            label: "Production Web 01".to_string(),
            host: "web01.company.com".to_string(),
            port: 22,
            username: "deploy".to_string(),
            tags: vec!["prod".to_string(), "web".to_string()],
            auth_method: "key".to_string(),
            ssh_key_id: Some("key-prod".to_string()),
            created_at: Some("2026-01-03T00:00:00Z".to_string()),
            updated_at: Some("2026-01-03T00:00:00Z".to_string()),
        },
    ]
}

#[test]
fn test_extract_unique_tags_sorted() {
    let conns = sample_connections();
    let tags = extract_unique_tags(&conns);
    assert_eq!(tags, vec!["database", "prod", "staging", "web"]);
}

#[test]
fn test_filter_connections_empty_returns_all() {
    let conns = sample_connections();
    let res = filter_connections(&conns, "", None);
    assert_eq!(res.len(), 3);
}

#[test]
fn test_filter_connections_by_label_case_insensitive() {
    let conns = sample_connections();
    let res = filter_connections(&conns, "production", None);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].id, "conn-1");
    assert_eq!(res[1].id, "conn-3");

    let res_staging = filter_connections(&conns, "STAGING", None);
    assert_eq!(res_staging.len(), 1);
    assert_eq!(res_staging[0].id, "conn-2");
}

#[test]
fn test_filter_connections_by_host() {
    let conns = sample_connections();
    let res = filter_connections(&conns, "10.0.0", None);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "conn-1");

    let res_fqdn = filter_connections(&conns, "company.com", None);
    assert_eq!(res_fqdn.len(), 1);
    assert_eq!(res_fqdn[0].id, "conn-3");
}

#[test]
fn test_filter_connections_by_username() {
    let conns = sample_connections();
    let res = filter_connections(&conns, "deploy", None);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "conn-3");
}

#[test]
fn test_filter_connections_by_tag_query() {
    let conns = sample_connections();
    let res = filter_connections(&conns, "database", None);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "conn-1");
}

#[test]
fn test_filter_connections_by_selected_tag_pill() {
    let conns = sample_connections();
    let res_prod = filter_connections(&conns, "", Some("prod"));
    assert_eq!(res_prod.len(), 2);
    assert_eq!(res_prod[0].id, "conn-1");
    assert_eq!(res_prod[1].id, "conn-3");

    let res_staging = filter_connections(&conns, "", Some("staging"));
    assert_eq!(res_staging.len(), 1);
    assert_eq!(res_staging[0].id, "conn-2");

    let res_web = filter_connections(&conns, "", Some("web"));
    assert_eq!(res_web.len(), 2);
    assert_eq!(res_web[0].id, "conn-2");
    assert_eq!(res_web[1].id, "conn-3");
}

#[test]
fn test_filter_connections_combined_query_and_tag() {
    let conns = sample_connections();
    // In "web" tag, search for "deploy"
    let res = filter_connections(&conns, "deploy", Some("web"));
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].id, "conn-3");

    // In "prod" tag, search for "ubuntu" -> no match
    let res_empty = filter_connections(&conns, "ubuntu", Some("prod"));
    assert!(res_empty.is_empty());
}
