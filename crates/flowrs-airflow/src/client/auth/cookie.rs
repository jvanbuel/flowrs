use crate::error::Result;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;
use std::fmt;

use super::command::CachedCommand;
use super::AuthProvider;

pub struct CookieAuthProvider {
    pub(super) cookie: String,
}

impl fmt::Debug for CookieAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CookieAuthProvider")
            .field("cookie", &"***redacted***")
            .finish()
    }
}

#[async_trait]
impl AuthProvider for CookieAuthProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Cookie Auth (static)");
        Ok(request.header("Cookie", self.cookie.trim()))
    }
}

/// Cookie auth whose value comes from a helper command, re-run on a short TTL so
/// a rotating session cookie is refreshed automatically.
pub struct CommandCookieProvider {
    cached: CachedCommand,
}

impl CommandCookieProvider {
    pub fn new(cmd: String) -> Self {
        Self {
            cached: CachedCommand::new(cmd, "Cookie", "cookie command"),
        }
    }
}

impl fmt::Debug for CommandCookieProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show the configured command, never the cached cookie.
        f.debug_struct("CommandCookieProvider")
            .field("cmd", &self.cached.cmd())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl AuthProvider for CommandCookieProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        let cookie = self.cached.value().await?;
        Ok(request.header("Cookie", cookie.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookie_header(request: reqwest::RequestBuilder) -> String {
        request
            .build()
            .unwrap()
            .headers()
            .get("Cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn get() -> reqwest::RequestBuilder {
        reqwest::Client::new().get("http://localhost:8080/api/v1/dags")
    }

    #[tokio::test]
    async fn test_cookie_auth_provider() {
        let provider = CookieAuthProvider {
            cookie: "session=my-session".to_string(),
        };
        let request = provider.authenticate(get()).await.unwrap();
        assert_eq!(cookie_header(request), "session=my-session");
    }

    #[tokio::test]
    async fn test_command_cookie_provider() {
        let provider = CommandCookieProvider::new("echo session=from-cmd".to_string());
        let request = provider.authenticate(get()).await.unwrap();
        assert_eq!(cookie_header(request), "session=from-cmd");
    }

    #[tokio::test]
    async fn test_command_cookie_provider_failure() {
        let provider = CommandCookieProvider::new("false".to_string());
        let result = provider.authenticate(get()).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cookie helper command failed"));
    }
}
