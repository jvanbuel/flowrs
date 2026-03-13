use anyhow::Result;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;
use std::fmt;

use super::AuthProvider;

pub struct BasicAuthProvider {
    pub(super) username: String,
    pub(super) password: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_auth_provider() {
        let provider = BasicAuthProvider {
            username: "airflow".to_string(),
            password: "airflow".to_string(),
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
        assert!(auth_header.starts_with("Basic "));
    }
}
