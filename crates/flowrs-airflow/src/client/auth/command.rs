use std::fmt;
use std::time::{Duration, Instant};

use anyhow::Context;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;

use super::AuthProvider;
use crate::error::{AirflowError, Result};

/// How long a fetched credential is reused before the helper command runs again.
///
/// The command's credential lifetime is unknown (it is user-defined), so this
/// uses a short, conservative window: long enough to eliminate the per-request
/// process spawn, short enough to stay safe for typical (minutes-to-hours)
/// lifetimes.
const CREDENTIAL_TTL: Duration = Duration::from_secs(60);

/// Runs a helper command and caches its trimmed stdout for [`CREDENTIAL_TTL`],
/// single-flighting concurrent refreshes. Shared by the token and cookie
/// command-based auth providers.
pub(super) struct CachedCommand {
    cmd: String,
    /// Human-readable credential name for log and error messages ("Token", "Cookie").
    label: &'static str,
    /// Static provider name passed to [`AirflowError::auth`].
    provider: &'static str,
    /// Cached `(value, fetched_at)`, refreshed once `CREDENTIAL_TTL` elapses. The
    /// async mutex also single-flights concurrent refreshes.
    cached: tokio::sync::Mutex<Option<(String, Instant)>>,
}

impl CachedCommand {
    pub fn new(cmd: String, label: &'static str, provider: &'static str) -> Self {
        Self {
            cmd,
            label,
            provider,
            cached: tokio::sync::Mutex::new(None),
        }
    }

    pub fn cmd(&self) -> &str {
        &self.cmd
    }

    /// Return the cached credential, refreshing via the helper command once the
    /// TTL elapses.
    pub async fn value(&self) -> Result<String> {
        let mut cached = self.cached.lock().await;

        let fresh = cached
            .as_ref()
            .is_some_and(|(_, fetched)| fetched.elapsed() < CREDENTIAL_TTL);

        if !fresh {
            info!(
                "🔑 {} Auth (command): refreshing via {}",
                self.label, self.cmd
            );
            let value = self
                .fetch()
                .await
                .map_err(|e| AirflowError::auth(self.provider, &e))?;
            *cached = Some((value, Instant::now()));
        }

        let (value, _) = cached.as_ref().expect("value cached above");
        Ok(value.clone())
    }

    /// Run the helper command and return its trimmed stdout.
    ///
    /// Uses `anyhow` internally for the layered context messages; the chain is
    /// flattened into `AirflowError::Auth` at the trait boundary.
    async fn fetch(&self) -> anyhow::Result<String> {
        let cmd = self.cmd.clone();
        let label = self.label;
        let output = tokio::task::spawn_blocking(move || {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .output()
                .with_context(|| format!("Failed to run {label} helper command"))
        })
        .await
        .with_context(|| format!("{label} helper task panicked"))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "{label} helper command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                output.status.code(),
                stdout,
                stderr
            ));
        }

        let value = String::from_utf8(output.stdout)
            .with_context(|| format!("{label} helper returned invalid UTF-8"))?;
        Ok(value.trim().trim_matches('"').to_string())
    }
}

pub struct CommandTokenProvider {
    cached: CachedCommand,
}

impl CommandTokenProvider {
    pub fn new(cmd: String) -> Self {
        Self {
            cached: CachedCommand::new(cmd, "Token", "token command"),
        }
    }
}

impl fmt::Debug for CommandTokenProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show the configured command, never the cached token.
        f.debug_struct("CommandTokenProvider")
            .field("cmd", &self.cached.cmd())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AuthProvider for CommandTokenProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let token = self.cached.value().await?;
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
