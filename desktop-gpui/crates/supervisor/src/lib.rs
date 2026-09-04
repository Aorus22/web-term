//! WebTerm backend supervisor — spawn, port handshake, readiness, lifecycle.
//!
//! Framework-free (tokio only, no GPUI) so the spawn/handshake/readiness/kill
//! logic is testable headless in CI (20-RESEARCH §Validation Architecture).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use tokio::io::AsyncBufReadExt;
use tokio::process::Child;

/// Maximum characters stored in the stderr ring buffer (newest-wins).
pub const MAX_STDERR_TAIL_CHARS: usize = 2000;

/// Lifecycle status of the managed backend process.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BackendStatus {
    /// Process launched; waiting for the `BACKEND_PORT` handshake.
    Starting,
    /// Handshake received and readiness probe passing; backend serving.
    Ready,
    /// Startup failed before readiness (invalid path, early exit, probe timeout).
    /// `stderr_tail` is the newest-wins tail of child stderr, capped at 2000 chars.
    Failed {
        reason: String,
        stderr_tail: String,
    },
    /// Backend exited unexpectedly after readiness.
    Crashed { exit_code: Option<i32> },
}

/// Errors originating from the supervisor.
#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("startup failed: {reason} (stderr: {stderr_tail})")]
    Failed {
        reason: String,
        stderr_tail: String,
    },
    #[error("invalid encryption key: {0}")]
    InvalidEncryptionKey(String),
    #[error("process IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("stop error: {0}")]
    Stop(String),
}

/// Options controlling backend spawn.
#[derive(Debug, Clone)]
pub struct SpawnOptions {
    /// Path to the backend executable (bundled binary, or a dev override).
    pub backend_path: PathBuf,
    /// Absolute SQLite path under the app data dir (cwd must not matter).
    pub db_path: PathBuf,
    /// 32-byte hex (64 chars) encryption key. Passed via env only —
    /// never argv (process-listing exposure), never logged.
    pub encryption_key: String,
    /// Value for WEBTERM_ALLOWED_ORIGINS (desktop default: "null").
    pub allowed_origins: String,
    /// Max time to wait for readiness after the handshake (default 15s).
    pub readiness_timeout: Duration,
    /// Max time to wait for the BACKEND_PORT handshake (default 15s).
    pub handshake_timeout: Duration,
}

impl SpawnOptions {
    /// Create new SpawnOptions with validated 64-hex-char encryption key.
    pub fn new(
        backend_path: impl Into<PathBuf>,
        db_path: impl Into<PathBuf>,
        encryption_key: impl Into<String>,
    ) -> Result<Self, SupervisorError> {
        let encryption_key = encryption_key.into();
        validate_hex_key(&encryption_key)?;

        Ok(Self {
            backend_path: backend_path.into(),
            db_path: db_path.into(),
            encryption_key,
            allowed_origins: "null".to_string(),
            readiness_timeout: Duration::from_secs(15),
            handshake_timeout: Duration::from_secs(15),
        })
    }

    pub fn with_allowed_origins(mut self, origins: impl Into<String>) -> Self {
        self.allowed_origins = origins.into();
        self
    }

    pub fn with_readiness_timeout(mut self, timeout: Duration) -> Self {
        self.readiness_timeout = timeout;
        self
    }

    pub fn with_handshake_timeout(mut self, timeout: Duration) -> Self {
        self.handshake_timeout = timeout;
        self
    }

    /// Return the environment variable pairs injected into the child process.
    pub fn env_vars(&self) -> Vec<(&'static str, String)> {
        vec![
            ("WEBTERM_HOST", "127.0.0.1".to_string()),
            ("WEBTERM_PORT", ":0".to_string()),
            ("WEBTERM_DB_PATH", self.db_path.to_string_lossy().to_string()),
            ("WEBTERM_ENCRYPTION_KEY", self.encryption_key.clone()),
            ("WEBTERM_ALLOWED_ORIGINS", self.allowed_origins.clone()),
        ]
    }

    /// Construct the child Command ensuring secrets are env-only (never argv).
    pub fn build_command(&self) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new(&self.backend_path);
        // Do not add arguments containing key material!
        for (k, v) in self.env_vars() {
            cmd.env(k, v);
        }
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);
        cmd
    }
}

fn validate_hex_key(key: &str) -> Result<(), SupervisorError> {
    if key.len() != 64 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SupervisorError::InvalidEncryptionKey(
            "encryption key must be exactly 64 hex characters (32 bytes)".to_string(),
        ));
    }
    Ok(())
}

/// Handle information for a running backend child process.
#[derive(Debug, Clone, PartialEq)]
pub struct BackendInfo {
    /// Loopback base URL derived from the handshake, e.g. "http://127.0.0.1:51234".
    pub base_url: String,
    pub pid: u32,
    pub port: u16,
}

/// Parse the first occurrence of `BACKEND_PORT:<port>` from a line.
pub fn parse_handshake_line(line: &str) -> Option<u16> {
    if let Some(idx) = line.find("BACKEND_PORT:") {
        let rest = &line[idx + "BACKEND_PORT:".len()..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u16>().ok()
    } else {
        None
    }
}

/// Process supervisor managing the Go backend child process lifecycle.
pub struct Supervisor {
    status_tx: tokio::sync::watch::Sender<BackendStatus>,
    status_rx: tokio::sync::watch::Receiver<BackendStatus>,
    info: Option<BackendInfo>,
    child: Arc<Mutex<Option<Child>>>,
    stderr_tail: Arc<Mutex<String>>,
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl Supervisor {
    /// Create a new Supervisor instance in Starting status.
    pub fn new() -> Self {
        let (status_tx, status_rx) = tokio::sync::watch::channel(BackendStatus::Starting);
        Self {
            status_tx,
            status_rx,
            info: None,
            child: Arc::new(Mutex::new(None)),
            stderr_tail: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Read current status.
    pub fn status(&self) -> BackendStatus {
        self.status_rx.borrow().clone()
    }

    /// Subscribe to status change notifications.
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<BackendStatus> {
        self.status_rx.clone()
    }

    /// Return backend info if currently ready/running.
    pub fn info(&self) -> Option<&BackendInfo> {
        self.info.as_ref()
    }

    /// Return snapshot of newest stderr tail.
    pub fn stderr_tail(&self) -> String {
        self.stderr_tail.lock().clone()
    }

    /// Spawn the backend process, perform port handshake and readiness polling.
    pub async fn spawn(&mut self, opts: SpawnOptions) -> Result<BackendInfo, SupervisorError> {
        let _ = self.status_tx.send(BackendStatus::Starting);
        self.stderr_tail.lock().clear();

        let mut cmd = opts.build_command();
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let reason = format!("Failed to spawn backend binary '{}': {}", opts.backend_path.display(), e);
                let tail = format!("Execution error: {}", e);
                let status = BackendStatus::Failed {
                    reason: reason.clone(),
                    stderr_tail: tail.clone(),
                };
                let _ = self.status_tx.send(status);
                return Err(SupervisorError::Failed {
                    reason,
                    stderr_tail: tail,
                });
            }
        };

        let pid = child.id().unwrap_or(0);
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Spawn stderr collector into ring buffer
        let stderr_tail = self.stderr_tail.clone();
        if let Some(stderr) = stderr {
            tokio::spawn(async move {
                let mut reader = tokio::io::BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let mut lock = stderr_tail.lock();
                    lock.push_str(&line);
                    lock.push('\n');
                    let len = lock.chars().count();
                    if len > MAX_STDERR_TAIL_CHARS {
                        let to_drop = len - MAX_STDERR_TAIL_CHARS;
                        *lock = lock.chars().skip(to_drop).collect();
                    }
                }
            });
        }

        // Wait for BACKEND_PORT handshake
        let mut port_found: Option<u16> = None;
        if let Some(stdout) = stdout {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            let handshake_fut = async {
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Some(port) = parse_handshake_line(&line) {
                        return Ok(port);
                    }
                }
                Err("Stdout closed without BACKEND_PORT handshake")
            };

            match tokio::time::timeout(opts.handshake_timeout, handshake_fut).await {
                Ok(Ok(port)) => {
                    port_found = Some(port);
                }
                Ok(Err(err_msg)) => {
                    let tail = self.stderr_tail();
                    let reason = format!("Backend handshake missing: {}", err_msg);
                    let status = BackendStatus::Failed {
                        reason: reason.clone(),
                        stderr_tail: tail.clone(),
                    };
                    let _ = self.status_tx.send(status);
                    let _ = child.kill().await;
                    return Err(SupervisorError::Failed { reason, stderr_tail: tail });
                }
                Err(_) => {
                    let tail = self.stderr_tail();
                    let reason = format!("Backend handshake timed out after {:?}", opts.handshake_timeout);
                    let status = BackendStatus::Failed {
                        reason: reason.clone(),
                        stderr_tail: tail.clone(),
                    };
                    let _ = self.status_tx.send(status);
                    let _ = child.kill().await;
                    return Err(SupervisorError::Failed { reason, stderr_tail: tail });
                }
            }
        }

        let port = match port_found {
            Some(p) => p,
            None => {
                let tail = self.stderr_tail();
                let reason = "No stdout available to read handshake".to_string();
                let status = BackendStatus::Failed {
                    reason: reason.clone(),
                    stderr_tail: tail.clone(),
                };
                let _ = self.status_tx.send(status);
                let _ = child.kill().await;
                return Err(SupervisorError::Failed { reason, stderr_tail: tail });
            }
        };

        let base_url = format!("http://127.0.0.1:{}", port);
        let info = BackendInfo {
            base_url: base_url.clone(),
            pid,
            port,
        };

        // Poll readiness probe (GET /api/settings)
        let probe_url = format!("{}/api/settings", base_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(1000))
            .build()
            .unwrap_or_default();

        let readiness_fut = async {
            let start = tokio::time::Instant::now();
            loop {
                // Check if child exited prematurely
                if let Ok(Some(exit_status)) = child.try_wait() {
                    return Err(format!("Backend child exited before readiness with status: {}", exit_status));
                }

                if let Ok(resp) = client.get(&probe_url).send().await {
                    if resp.status().is_success() || resp.status().as_u16() < 500 {
                        return Ok(());
                    }
                }

                if start.elapsed() > opts.readiness_timeout {
                    return Err(format!("Readiness probe timed out after {:?}", opts.readiness_timeout));
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        };

        if let Err(err_reason) = readiness_fut.await {
            let tail = self.stderr_tail();
            let status = BackendStatus::Failed {
                reason: err_reason.clone(),
                stderr_tail: tail.clone(),
            };
            let _ = self.status_tx.send(status);
            let _ = child.kill().await;
            return Err(SupervisorError::Failed { reason: err_reason, stderr_tail: tail });
        }

        // Ready!
        let _ = self.status_tx.send(BackendStatus::Ready);
        self.info = Some(info.clone());
        *self.child.lock() = Some(child);

        // Spawn child exit monitor
        let child_arc = Arc::clone(&self.child);
        let status_tx = self.status_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let mut lock = child_arc.lock();
                if let Some(child) = lock.as_mut() {
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            let code = exit_status.code();
                            let _ = status_tx.send(BackendStatus::Crashed { exit_code: code });
                            break;
                        }
                        Ok(None) => continue,
                        Err(_) => {
                            let _ = status_tx.send(BackendStatus::Crashed { exit_code: None });
                            break;
                        }
                    }
                } else {
                    // Stopped cleanly
                    break;
                }
            }
        });

        Ok(info)
    }

    /// Return child PID if currently spawned.
    pub fn child_pid(&self) -> Option<u32> {
        self.child.lock().as_ref().and_then(|c| c.id())
    }

    /// Stop the backend process: graceful terminate with grace duration, then hard kill.
    pub async fn stop(&mut self, grace: Duration) -> Result<(), SupervisorError> {
        let child_opt = self.child.lock().take();
        if let Some(mut child) = child_opt {
            // Attempt graceful stop if platform permits, else hard kill
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }
                }
            }

            let wait_fut = child.wait();
            match tokio::time::timeout(grace, wait_fut).await {
                Ok(Ok(_)) => {}
                _ => {
                    let _ = child.kill().await;
                }
            }
        }
        self.info = None;
        Ok(())
    }

    /// Check if a previously recorded backend is still serving, or clear it.
    pub async fn adopt_or_clear(base_url: Option<String>) -> Option<BackendInfo> {
        let base_url = base_url?;
        let probe_url = format!("{}/api/settings", base_url.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .ok()?;

        if let Ok(resp) = client.get(&probe_url).send().await {
            if resp.status().is_success() {
                let port = probe_url
                    .split(':')
                    .next_back()?
                    .split('/')
                    .next()?
                    .parse::<u16>()
                    .unwrap_or(0);
                return Some(BackendInfo {
                    base_url,
                    pid: 0,
                    port,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_handshake() {
        assert_eq!(parse_handshake_line("BACKEND_PORT:54321"), Some(54321));
        assert_eq!(parse_handshake_line("prefix BACKEND_PORT:8080 suffix"), Some(8080));
        assert_eq!(parse_handshake_line("BACKEND_PORT:0"), Some(0));
        assert_eq!(parse_handshake_line("BACKEND_PORT:notaport"), None);
        assert_eq!(parse_handshake_line("just a regular log line"), None);
    }

    #[test]
    fn test_spawn_options_validation() {
        let valid_key = "a1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
        let opts = SpawnOptions::new("bin", "db.sqlite", valid_key);
        assert!(opts.is_ok());

        let invalid_short = "shortkey";
        assert!(SpawnOptions::new("bin", "db.sqlite", invalid_short).is_err());

        let invalid_hex = "g1b2c3d4e5f67890123456789abcdef0123456789abcdef0123456789abcdef0";
        assert!(SpawnOptions::new("bin", "db.sqlite", invalid_hex).is_err());
    }

    #[test]
    fn test_env_only_secrets() {
        let key = "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff";
        let opts = SpawnOptions::new("backend", "data/webterm.db", key).unwrap();
        let env_vars = opts.env_vars();

        let key_env = env_vars.iter().find(|(k, _)| *k == "WEBTERM_ENCRYPTION_KEY");
        assert!(key_env.is_some());
        assert_eq!(key_env.unwrap().1, key);

        let cmd = opts.build_command();
        let std_cmd = cmd.as_std();
        // Assert argv contains no secret key material
        for arg in std_cmd.get_args() {
            assert!(!arg.to_string_lossy().contains(key));
        }
    }
}
