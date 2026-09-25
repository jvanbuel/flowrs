use crate::error::Result;
use async_trait::async_trait;
use log::info;
use reqwest::RequestBuilder;
use std::fmt;

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
        info!("🔑 Cookie Auth");
        Ok(request.header("Cookie", self.cookie.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cookie_auth_provider() {
        let provider = CookieAuthProvider {
            cookie: "session=my-session".to_string(),
        };
        let request = provider
            .authenticate(reqwest::Client::new().get("http://localhost:8080/api/v1/dags"))
            .await
            .unwrap();
        let built = request.build().unwrap();
        let cookie_header = built.headers().get("Cookie").unwrap().to_str().unwrap();
        assert_eq!(cookie_header, "session=my-session");
    }
}
