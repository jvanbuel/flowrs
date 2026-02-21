use anyhow::Result;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;
use std::fmt;

use super::AuthProvider;

pub struct StaticTokenProvider {
    pub(super) token: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_token_provider() {
        let provider = StaticTokenProvider {
            token: "my-token".to_string(),
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
        assert_eq!(auth_header, "Bearer my-token");
    }
}
