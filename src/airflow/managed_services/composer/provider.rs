use crate::airflow::client::auth::AuthProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use google_cloud_auth::credentials::{AccessTokenCredentials, Builder};
use log::info;
use reqwest::RequestBuilder;

use super::ComposerAuth;

#[derive(Clone)]
pub struct ComposerAuthProvider {
    project_id: String,
    environment_name: String,
    credentials: AccessTokenCredentials,
}

impl ComposerAuthProvider {
    pub fn new(auth: &ComposerAuth) -> Result<Self> {
        let credentials = Builder::default()
            .build_access_token_credentials()
            .context("Failed to build GCP credentials")?;
        Ok(Self {
            project_id: auth.project_id.clone(),
            environment_name: auth.environment_name.clone(),
            credentials,
        })
    }
}

impl std::fmt::Debug for ComposerAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposerAuthProvider")
            .field("project_id", &self.project_id)
            .field("environment_name", &self.environment_name)
            .field("credentials", &"***")
            .finish()
    }
}

#[async_trait]
impl AuthProvider for ComposerAuthProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!(
            "🔑 Composer Auth: {}/{}",
            self.project_id, self.environment_name
        );
        let token = self
            .credentials
            .access_token()
            .await
            .context("Failed to get GCP access token")?;
        Ok(request.bearer_auth(token.token))
    }

    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(self.clone())
    }
}
