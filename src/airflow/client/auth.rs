use anyhow::{Context, Result};
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;
use std::fmt;

use crate::airflow::config::{AirflowAuth, BasicAuth, TokenSource};
use crate::airflow::managed_services::astronomer::AstronomerAuthProvider;
use crate::airflow::managed_services::composer::ComposerAuthProvider;
use crate::airflow::managed_services::conveyor::ConveyorAuthProvider;
use crate::airflow::managed_services::mwaa::MwaaAuthProvider;

/// Authentication provider trait for Airflow API requests.
///
/// Each implementation decorates a `RequestBuilder` with the appropriate
/// authentication headers/cookies for a specific auth method.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder>;
}

// --- Core (non-managed-service) providers ---

pub struct BasicAuthProvider {
    username: String,
    password: String,
}

impl fmt::Debug for BasicAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicAuthProvider")
            .field("username", &self.username)
            .field("password", &"***redacted***")
            .finish()
    }
}

#[async_trait]
impl AuthProvider for BasicAuthProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Basic Auth: {}", self.username);
        Ok(request.basic_auth(&self.username, Some(&self.password)))
    }
}

pub struct StaticTokenProvider {
    token: String,
}

impl fmt::Debug for StaticTokenProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StaticTokenProvider")
            .field("token", &"***redacted***")
            .finish()
    }
}

#[async_trait]
impl AuthProvider for StaticTokenProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Token Auth (static)");
        Ok(request.bearer_auth(self.token.trim()))
    }
}

#[derive(Debug)]
pub struct CommandTokenProvider {
    cmd: String,
}

#[async_trait]
impl AuthProvider for CommandTokenProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Token Auth (command): {}", self.cmd);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .output()
            .context("Failed to run token helper command")?;

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

        let token = String::from_utf8(output.stdout)
            .context("Token helper returned invalid UTF-8")?
            .trim()
            .replace('"', "");
        Ok(request.bearer_auth(token))
    }
}

/// Create an auth provider from an `AirflowAuth` config enum variant.
pub fn create_auth_provider(auth: &AirflowAuth) -> Result<Box<dyn AuthProvider>> {
    match auth {
        AirflowAuth::Basic(BasicAuth { username, password }) => Ok(Box::new(BasicAuthProvider {
            username: username.clone(),
            password: password.clone(),
        })),
        AirflowAuth::Token(TokenSource::Static { token }) => Ok(Box::new(StaticTokenProvider {
            token: token.clone(),
        })),
        AirflowAuth::Token(TokenSource::Command { cmd }) => {
            Ok(Box::new(CommandTokenProvider { cmd: cmd.clone() }))
        }
        AirflowAuth::Conveyor => Ok(Box::new(ConveyorAuthProvider)),
        AirflowAuth::Mwaa(mwaa_auth) => Ok(Box::new(MwaaAuthProvider::from(mwaa_auth))),
        AirflowAuth::Astronomer(astro_auth) => {
            Ok(Box::new(AstronomerAuthProvider::from(astro_auth)))
        }
        AirflowAuth::Composer(composer_auth) => {
            Ok(Box::new(ComposerAuthProvider::new(composer_auth)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request() -> RequestBuilder {
        reqwest::Client::new().get("http://localhost:8080/api/v1/dags")
    }

    #[tokio::test]
    async fn test_basic_auth_provider() {
        let provider = BasicAuthProvider {
            username: "airflow".to_string(),
            password: "airflow".to_string(),
        };
        let request = provider.authenticate(test_request()).await.unwrap();
        let built = request.build().unwrap();
        let auth_header = built
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(auth_header.starts_with("Basic "));
    }

    #[tokio::test]
    async fn test_static_token_provider() {
        let provider = StaticTokenProvider {
            token: "my-token".to_string(),
        };
        let request = provider.authenticate(test_request()).await.unwrap();
        let built = request.build().unwrap();
        let auth_header = built
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth_header, "Bearer my-token");
    }

    #[tokio::test]
    async fn test_command_token_provider() {
        let provider = CommandTokenProvider {
            cmd: "echo test-token".to_string(),
        };
        let request = provider.authenticate(test_request()).await.unwrap();
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
        let result = provider.authenticate(test_request()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token helper command failed"));
    }

    #[test]
    fn test_create_auth_provider_basic() {
        let auth = AirflowAuth::Basic(BasicAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }

    #[test]
    fn test_create_auth_provider_static_token() {
        let auth = AirflowAuth::Token(TokenSource::Static {
            token: "tok".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }

    #[test]
    fn test_create_auth_provider_command_token() {
        let auth = AirflowAuth::Token(TokenSource::Command {
            cmd: "echo hi".to_string(),
        });
        assert!(create_auth_provider(&auth).is_ok());
    }
}
