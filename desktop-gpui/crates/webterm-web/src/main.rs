//! WebTerm Web Server runner binary (`webterm-web.exe`).
//!
//! Spawns the Golang backend (`backend.exe`) via `webterm-supervisor`,
//! starts an Axum HTTP server on `127.0.0.1:8080`,
//! serves the WebAssembly GPUI client from `dist/`,
//! and reverse-proxies `/api/*` and `/ws` to the Golang backend.

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
mod server {
    use std::path::PathBuf;
    use std::sync::Arc;
    use axum::extract::State;
    use axum::routing::{any, get};
    use axum::Router;
    use futures_util::{SinkExt, StreamExt};
    use tower_http::cors::CorsLayer;
    use tower_http::services::ServeDir;
    use webterm_settings::DesktopSettings;
    use webterm_supervisor::{SpawnOptions, Supervisor};

    pub fn resolve_backend_path(settings: &DesktopSettings) -> PathBuf {
        if let Some(ref path) = settings.backend_path {
            return path.clone();
        }

        let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
        let bin_name = format!("backend{}", exe_suffix);

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let adjacent = parent.join(&bin_name);
                if adjacent.exists() {
                    return adjacent.canonicalize().unwrap_or(adjacent);
                }
            }
        }

        let candidates = [
            PathBuf::from("test-support").join(&bin_name),
            PathBuf::from("desktop-gpui/test-support").join(&bin_name),
            PathBuf::from("../../desktop-gpui/test-support").join(&bin_name),
            PathBuf::from("../test-support").join(&bin_name),
            PathBuf::from("dist/webterm-windows-x64").join(&bin_name),
            PathBuf::from("../../dist/webterm-windows-x64").join(&bin_name),
            PathBuf::from("be/compiled/bin").join(&bin_name),
            PathBuf::from("../../be/compiled/bin").join(&bin_name),
        ];

        for c in candidates {
            if c.exists() {
                return c.canonicalize().unwrap_or(c);
            }
        }

        PathBuf::from(format!("test-support/{}", bin_name))
    }

    pub fn resolve_dist_path() -> PathBuf {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let adjacent = parent.join("dist");
                if adjacent.exists() {
                    return adjacent;
                }
            }
        }

        let candidates = [
            PathBuf::from("dist"),
            PathBuf::from("crates/webterm-web/dist"),
            PathBuf::from("desktop-gpui/crates/webterm-web/dist"),
            PathBuf::from("../../desktop-gpui/crates/webterm-web/dist"),
            PathBuf::from("../webterm-web/dist"),
        ];

        for c in candidates {
            if c.exists() {
                return c;
            }
        }

        PathBuf::from("dist")
    }

    #[derive(Clone)]
    pub struct AppServerState {
        pub backend_port: u16,
        pub client: reqwest::Client,
    }

    async fn proxy_api(
        State(state): State<Arc<AppServerState>>,
        method: axum::http::Method,
        uri: axum::http::Uri,
        headers: axum::http::HeaderMap,
        body: axum::body::Bytes,
    ) -> Result<axum::response::Response, axum::http::StatusCode> {
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
        let target_url = format!("http://127.0.0.1:{}{}", state.backend_port, path_and_query);

        let mut req = state.client.request(method, &target_url);
        for (k, v) in headers.iter() {
            let key_str = k.as_str().to_lowercase();
            if key_str != "host"
                && key_str != "content-length"
                && key_str != "connection"
                && key_str != "transfer-encoding"
            {
                req = req.header(k, v);
            }
        }
        req = req.body(body);

        let resp = req.send().await.map_err(|_| axum::http::StatusCode::BAD_GATEWAY)?;
        let status = axum::http::StatusCode::from_u16(resp.status().as_u16())
            .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);

        let mut res_builder = axum::response::Response::builder().status(status);
        for (k, v) in resp.headers().iter() {
            let name = k.as_str().to_lowercase();
            if name != "transfer-encoding"
                && name != "connection"
                && name != "keep-alive"
                && name != "content-length"
            {
                res_builder = res_builder.header(k, v);
            }
        }

        let bytes = resp.bytes().await.map_err(|_| axum::http::StatusCode::BAD_GATEWAY)?;
        Ok(res_builder.body(axum::body::Body::from(bytes)).unwrap())
    }

    async fn proxy_ws(
        State(state): State<Arc<AppServerState>>,
        ws: axum::extract::ws::WebSocketUpgrade,
        uri: axum::http::Uri,
    ) -> axum::response::Response {
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/ws");
        let target_url = format!("ws://127.0.0.1:{}{}", state.backend_port, path_and_query);

        ws.on_upgrade(move |client_ws| async move {
            if let Ok((backend_ws, _)) = tokio_tungstenite::connect_async(&target_url).await {
                let (mut backend_tx, mut backend_rx) = backend_ws.split();
                let (mut client_tx, mut client_rx) = client_ws.split();

                let client_to_backend = async {
                    while let Some(Ok(msg)) = client_rx.next().await {
                        match msg {
                            axum::extract::ws::Message::Text(t) => {
                                if backend_tx
                                    .send(tokio_tungstenite::tungstenite::Message::Text(t.as_str().into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            axum::extract::ws::Message::Binary(b) => {
                                if backend_tx
                                    .send(tokio_tungstenite::tungstenite::Message::Binary(b.to_vec().into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            axum::extract::ws::Message::Ping(p) => {
                                let _ = backend_tx
                                    .send(tokio_tungstenite::tungstenite::Message::Ping(p.to_vec().into()))
                                    .await;
                            }
                            axum::extract::ws::Message::Pong(p) => {
                                let _ = backend_tx
                                    .send(tokio_tungstenite::tungstenite::Message::Pong(p.to_vec().into()))
                                    .await;
                            }
                            axum::extract::ws::Message::Close(c) => {
                                let _ = backend_tx
                                    .send(tokio_tungstenite::tungstenite::Message::Close(
                                        c.map(|c| tokio_tungstenite::tungstenite::protocol::CloseFrame {
                                            code: c.code.into(),
                                            reason: c.reason.as_str().into(),
                                        }),
                                    ))
                                    .await;
                                break;
                            }
                        }
                    }
                };

                let backend_to_client = async {
                    while let Some(Ok(msg)) = backend_rx.next().await {
                        match msg {
                            tokio_tungstenite::tungstenite::Message::Text(t) => {
                                if client_tx.send(axum::extract::ws::Message::Text(t.as_str().into())).await.is_err() {
                                    break;
                                }
                            }
                            tokio_tungstenite::tungstenite::Message::Binary(b) => {
                                if client_tx
                                    .send(axum::extract::ws::Message::Binary(b.to_vec().into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            tokio_tungstenite::tungstenite::Message::Ping(p) => {
                                let _ = client_tx.send(axum::extract::ws::Message::Ping(p.to_vec().into())).await;
                            }
                            tokio_tungstenite::tungstenite::Message::Pong(p) => {
                                let _ = client_tx.send(axum::extract::ws::Message::Pong(p.to_vec().into())).await;
                            }
                            tokio_tungstenite::tungstenite::Message::Close(c) => {
                                let _ = client_tx
                                    .send(axum::extract::ws::Message::Close(
                                        c.map(|c| axum::extract::ws::CloseFrame {
                                            code: c.code.into(),
                                            reason: c.reason.as_str().into(),
                                        }),
                                    ))
                                    .await;
                                break;
                            }
                            _ => {}
                        }
                    }
                };

                tokio::select! {
                    _ = client_to_backend => {}
                    _ = backend_to_client => {}
                }
            }
        })
    }

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = DesktopSettings::load().unwrap_or_default();
        let encryption_key = settings.ensure_encryption_key();
        let backend_path = resolve_backend_path(&settings);
        let db_path = webterm_settings::paths::db_path();

        println!("Resolving backend path: {}", backend_path.display());
        println!("Database path:          {}", db_path.display());

        let spawn_opts = SpawnOptions::new(backend_path, db_path, encryption_key)
            .map_err(|e| format!("Spawn options error: {e:?}"))?;

        let mut supervisor = Supervisor::new();
        let info = supervisor.spawn(spawn_opts).await
            .map_err(|e| format!("Supervisor spawn failed: {e:?}"))?;

        let backend_port = info.port;

        let dist_dir = resolve_dist_path();
        println!("Serving static dist:    {}", dist_dir.display());

        let state = Arc::new(AppServerState {
            backend_port,
            client: reqwest::Client::new(),
        });

        let app = Router::new()
            .route("/api/{*path}", any(proxy_api))
            .route("/ws", get(proxy_ws))
            .fallback_service(ServeDir::new(&dist_dir).append_index_html_on_directories(true))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let web_port = 8080;
        let addr = format!("127.0.0.1:{}", web_port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        println!("\n=======================================================");
        println!("  WebTerm Web Server is running!");
        println!("  -> Web Client: http://127.0.0.1:{}", web_port);
        println!("  -> Golang BE:  http://127.0.0.1:{} (managed)", backend_port);
        println!("=======================================================\n");

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &format!("http://127.0.0.1:{}", web_port)])
                .spawn();
        }

        axum::serve(listener, app).await?;

        let _ = supervisor.stop(std::time::Duration::from_millis(500)).await;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::run().await
}
