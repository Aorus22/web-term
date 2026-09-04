//! Typed HTTP/WS client for the local WebTerm backend.
//!
//! Provides DTO definitions mirroring backend models and a typed REST client
//! for interacting with backend endpoints.

pub mod rest;
pub mod types;

pub use rest::{BackendClient, ClientError};
pub use types::{Connection, Settings};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_is_ready_false_on_connection_refused() {
        // Use a high port where no server is listening
        let client = BackendClient::new("http://127.0.0.1:59999");
        assert!(!client.is_ready().await);
    }

    #[tokio::test]
    async fn test_get_settings_parses() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = r#"{"settings":{"theme_mode":"system","font_size":"14"}}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        let client = BackendClient::new(format!("http://127.0.0.1:{}", port));
        let settings = client.get_settings().await.expect("should fetch settings");
        assert_eq!(settings.settings.get("theme_mode").map(|s| s.as_str()), Some("system"));
        assert_eq!(settings.settings.get("font_size").map(|s| s.as_str()), Some("14"));
    }

    #[tokio::test]
    async fn test_list_connections_parses_two_elements() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;
                let body = r#"[
                    {"id":"c1","label":"Prod Server","host":"10.0.0.1","port":22,"username":"admin","tags":["prod"],"auth_method":"password"},
                    {"id":"c2","label":"Dev Box","host":"10.0.0.2","port":2222,"username":"dev","tags":["dev"],"auth_method":"key"}
                ]"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        let client = BackendClient::new(format!("http://127.0.0.1:{}", port));
        let list = client.list_connections().await.expect("should list connections");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, "c1");
        assert_eq!(list[0].label, "Prod Server");
        assert_eq!(list[0].port, 22);
        assert_eq!(list[1].id, "c2");
        assert_eq!(list[1].label, "Dev Box");
        assert_eq!(list[1].port, 2222);
    }
}
