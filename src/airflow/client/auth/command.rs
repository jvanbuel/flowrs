use anyhow::{Context, Result};
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;

use super::AuthProvider;

#[derive(Debug)]
pub struct CommandTokenProvider {
    pub(super) cmd: String,
}

#[async_trait]
impl AuthProvider for CommandTokenProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Token Auth (command): {}", self.cmd);
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
        let token = token.trim().trim_matches('"');
        Ok(request.bearer_auth(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_token_provider() {
        let provider = CommandTokenProvider {
            cmd: "echo test-token".to_string(),
        };
        let request = provider
            .authenticate(reqwest::Client::new().get("http://localhost:8080/api/v1/dags"))
            .await
            .unwrap();
        let built = request.build().unwrap();
        let auth_header = built
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth_header, "Bearer test-token");
    }

    #[tokio::test]
    async fn test_command_token_provider_failure() {
        let provider = CommandTokenProvider {
            cmd: "false".to_string(),
        };
        let result = provider
            .authenticate(reqwest::Client::new().get("http://localhost:8080/api/v1/dags"))
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token helper command failed"));
    }
}
