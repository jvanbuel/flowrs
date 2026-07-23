use std::fmt;
use std::time::{Duration, Instant};

use anyhow::Context;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;

use super::AuthProvider;
use crate::error::{AirflowError, Result};

/// How long a fetched token is reused before the helper command is run again.
///
/// The command's token lifetime is unknown (it is user-defined), so this uses a
/// short, conservative window: long enough to eliminate the per-request process
/// spawn, short enough to stay safe for typical (minutes-to-hours) token
/// lifetimes.
const TOKEN_TTL: Duration = Duration::from_secs(60);

pub struct CommandTokenProvider {
    cmd: String,
    /// Cached `(token, fetched_at)`, refreshed once `TOKEN_TTL` elapses. The
    /// async mutex also single-flights concurrent refreshes.
    cached: tokio::sync::Mutex<Option<(String, Instant)>>,
}

impl CommandTokenProvider {
    pub fn new(cmd: String) -> Self {
        Self {
            cmd,
            cached: tokio::sync::Mutex::new(None),
        }
    }

    /// Run the helper command and return its trimmed token output.
    ///
    /// Uses `anyhow` internally for the layered context messages; the chain is
    /// flattened into `AirflowError::Auth` at the trait boundary.
    async fn fetch_token(&self) -> anyhow::Result<String> {
        let cmd = self.cmd.clone();
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .context("Failed to run token helper command")
        })
        .await
        .context("Token helper task panicked")??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Token helper command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            ));
        }

        let token =
            String::from_utf8(output.stdout).context("Token helper returned invalid UTF-8")?;
        Ok(token.trim().trim_matches('"').to_string())
    }
}

impl fmt::Debug for CommandTokenProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show the configured command, never the cached token.
        f.debug_struct("CommandTokenProvider")
            .field("cmd", &self.cmd)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AuthProvider for CommandTokenProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let mut cached = self.cached.lock().await;

        let fresh = cached
            .as_ref()
            .is_some_and(|(_, fetched)| fetched.elapsed() < TOKEN_TTL);

        if !fresh {
            info!("🔑 Token Auth (command): refreshing via {}", self.cmd);
            let token = self
                .fetch_token()
                .await
                .map_err(|e| AirflowError::auth("token command", &e))?;
            *cached = Some((token, Instant::now()));
        }

        let (token, _) = cached.as_ref().expect("token cached above");
        Ok(request.bearer_auth(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bearer(request: reqwest::RequestBuilder) -> String {
        request
            .build()
            .unwrap()
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn get() -> reqwest::RequestBuilder {
        reqwest::Client::new().get("http://localhost:8080/api/v1/dags")
    }

    #[tokio::test]
    async fn test_command_token_provider() {
        let provider = CommandTokenProvider::new("echo test-token".to_string());
        let request = provider.authenticate(get()).await.unwrap();
        assert_eq!(bearer(request), "Bearer test-token");
    }

    #[tokio::test]
    async fn test_command_token_provider_failure() {
        let provider = CommandTokenProvider::new("false".to_string());
        let result = provider.authenticate(get()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token helper command failed"));
    }

    #[tokio::test]
    async fn caches_token_and_runs_command_only_once() {
        // The command appends a line each time it runs, so we can count runs.
        let marker = std::env::temp_dir().join(format!(
            "flowrs-cmd-cache-{}-{}",
            std::process::id(),
            "once"
        ));
        let _ = std::fs::remove_file(&marker);
        let cmd = format!("echo run >> {}; echo tok", marker.display());
        let provider = CommandTokenProvider::new(cmd);

        // Two authentications within the TTL should reuse the first token.
        assert_eq!(
            bearer(provider.authenticate(get()).await.unwrap()),
            "Bearer tok"
        );
        assert_eq!(
            bearer(provider.authenticate(get()).await.unwrap()),
            "Bearer tok"
        );

        let runs = std::fs::read_to_string(&marker).unwrap().lines().count();
        let _ = std::fs::remove_file(&marker);
        assert_eq!(runs, 1, "helper command should run once, not per request");
    }
}
