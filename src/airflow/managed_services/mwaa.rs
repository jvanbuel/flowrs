use crate::airflow::client::auth::AuthProvider;
use crate::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, ManagedService};
use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_mwaa as mwaa;
use log::info;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

/// MWAA client for managing authentication and environment discovery
#[derive(Debug, Clone)]
pub struct MwaaClient {
    client: mwaa::Client,
}

impl MwaaClient {
    /// Creates a new MWAA client using default AWS configuration
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = mwaa::Client::new(&config);
        Ok(Self { client })
    }

    /// Lists all MWAA environments in the current AWS account/region
    pub async fn list_environments(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .list_environments()
            .send()
            .await
            .context("Failed to list MWAA environments")?;

        Ok(response.environments().to_vec())
    }

    /// Gets detailed information about a specific MWAA environment
    pub async fn get_environment(&self, name: &str) -> Result<MwaaEnvironment> {
        let response = self
            .client
            .get_environment()
            .name(name)
            .send()
            .await
            .context(format!("Failed to get environment: {name}"))?;

        let env = response
            .environment
            .context("No environment data in response")?;

        let airflow_version = env
            .airflow_version()
            .context("No Airflow version in response")?
            .to_string();

        let webserver_url = env
            .webserver_url()
            .context("No webserver URL in response")?
            .to_string();

        Ok(MwaaEnvironment {
            name: name.to_string(),
            airflow_version,
            webserver_url,
        })
    }

    /// Creates a web login token for a specific MWAA environment
    pub async fn create_web_login_token(&self, name: &str) -> Result<MwaaWebToken> {
        let response = self
            .client
            .create_web_login_token()
            .name(name)
            .send()
            .await
            .context(format!("Failed to create web login token for: {name}"))?;

        let web_token = response.web_token().context("No web token in response")?;
        let hostname = response
            .web_server_hostname()
            .context("No webserver hostname in response")?;

        Ok(MwaaWebToken {
            token: web_token.to_string(),
            hostname: hostname.to_string(),
        })
    }

    /// Exchanges a web login token for an authentication token.
    /// For Airflow 2.x: Uses `/aws_mwaa/login` and returns a session cookie.
    /// For Airflow 3.x: Uses `/pluginsv2/aws_mwaa/login` and returns a JWT token.
    pub async fn get_auth_token(
        &self,
        web_token: &MwaaWebToken,
        version: &AirflowVersion,
    ) -> Result<MwaaTokenType> {
        // Different endpoints and cookie names based on Airflow version
        let (login_path, cookie_name) = match version {
            AirflowVersion::V2 => ("aws_mwaa/login", "session"),
            AirflowVersion::V3 => ("pluginsv2/aws_mwaa/login", "_token"),
        };

        let login_url = format!("https://{}/{}", web_token.hostname, login_path);

        let form_data = LoginForm {
            token: &web_token.token,
        };

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none()) // Don't follow redirects
            .build()
            .context("Failed to build HTTP client")?;

        let response = client
            .post(&login_url)
            .form(&form_data)
            .send()
            .await
            .context("Failed to send login request")?;

        // MWAA login returns a redirect with Set-Cookie header
        if !response.status().is_redirection() && !response.status().is_success() {
            anyhow::bail!("Failed to log in: HTTP {}", response.status());
        }

        // Extract the token from Set-Cookie header
        let cookies = response.headers().get_all("set-cookie");

        for cookie_header in cookies {
            let cookie_str = cookie_header.to_str().context("Invalid cookie header")?;

            // Parse the cookie to extract token value
            if let Some(cookie_part) = cookie_str.split(';').next() {
                if let Some((name, value)) = cookie_part.split_once('=') {
                    if name == cookie_name {
                        return Ok(match version {
                            AirflowVersion::V2 => MwaaTokenType::SessionCookie(value.to_string()),
                            AirflowVersion::V3 => MwaaTokenType::JwtToken(value.to_string()),
                        });
                    }
                }
            }
        }

        anyhow::bail!("No {cookie_name} cookie found in response")
    }
}

/// MWAA environment metadata
#[derive(Debug, Clone)]
pub struct MwaaEnvironment {
    pub name: String,
    pub airflow_version: String,
    pub webserver_url: String,
}

/// MWAA web login token and hostname
#[derive(Debug, Clone)]
pub struct MwaaWebToken {
    pub token: String,
    pub hostname: String,
}

/// MWAA authentication token type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MwaaTokenType {
    /// Session cookie for Airflow 2.x (uses Cookie header)
    SessionCookie(String),
    /// JWT token for Airflow 3.x (uses Bearer auth)
    JwtToken(String),
}

/// MWAA authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwaaAuth {
    pub token: MwaaTokenType,
    pub environment_name: String,
}

#[derive(Debug, Clone)]
pub struct MwaaAuthProvider {
    token: MwaaTokenType,
    environment_name: String,
}

impl From<&MwaaAuth> for MwaaAuthProvider {
    fn from(auth: &MwaaAuth) -> Self {
        Self {
            token: auth.token.clone(),
            environment_name: auth.environment_name.clone(),
        }
    }
}

#[async_trait]
impl AuthProvider for MwaaAuthProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 MWAA Auth: {}", self.environment_name);
        match &self.token {
            MwaaTokenType::SessionCookie(cookie) => {
                Ok(request.header("Cookie", format!("session={cookie}")))
            }
            MwaaTokenType::JwtToken(token) => Ok(request.bearer_auth(token)),
        }
    }

    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(self.clone())
    }
}

#[derive(Serialize)]
struct LoginForm<'a> {
    token: &'a str,
}

/// Lists all MWAA environments and returns them as `AirflowConfig` instances
pub async fn get_mwaa_environment_servers() -> Result<Vec<AirflowConfig>> {
    let client = MwaaClient::new().await?;
    let env_names = client.list_environments().await?;

    let mut servers = Vec::new();

    for env_name in env_names {
        let env = client.get_environment(&env_name).await?;

        // Determine Airflow version from the version string
        let version = if env.airflow_version.starts_with("2.") {
            AirflowVersion::V2
        } else if env.airflow_version.starts_with("3.") {
            AirflowVersion::V3
        } else {
            anyhow::bail!(
                "Unsupported Airflow version '{}' for environment '{}'",
                env.airflow_version,
                env_name
            );
        };

        // Create web token and authenticate for this environment
        let web_token = client.create_web_login_token(&env_name).await?;
        let auth_token = client.get_auth_token(&web_token, &version).await?;

        // Ensure the endpoint has a proper scheme (MWAA webserver URLs may not include https://)
        let endpoint = if env.webserver_url.starts_with("http://")
            || env.webserver_url.starts_with("https://")
        {
            env.webserver_url.clone()
        } else {
            format!("https://{}", env.webserver_url)
        };

        servers.push(AirflowConfig {
            name: env.name.clone(),
            endpoint,
            auth: AirflowAuth::Mwaa(MwaaAuth {
                token: auth_token,
                environment_name: env.name.clone(),
            }),
            managed: Some(ManagedService::Mwaa),
            version,
            timeout_secs: 30,
        });
    }

    info!("Found {} MWAA environment(s)", servers.len());
    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request() -> RequestBuilder {
        reqwest::Client::new().get("http://localhost:8080/api/v1/dags")
    }

    #[tokio::test]
    async fn test_mwaa_session_cookie_provider() {
        let provider = MwaaAuthProvider {
            token: MwaaTokenType::SessionCookie("my-session".to_string()),
            environment_name: "test-env".to_string(),
        };
        let request = provider.authenticate(test_request()).await.unwrap();
        let built = request.build().unwrap();
        let cookie = built.headers().get("Cookie").unwrap().to_str().unwrap();
        assert_eq!(cookie, "session=my-session");
    }

    #[tokio::test]
    async fn test_mwaa_jwt_token_provider() {
        let provider = MwaaAuthProvider {
            token: MwaaTokenType::JwtToken("jwt-token".to_string()),
            environment_name: "test-env".to_string(),
        };
        let request = provider.authenticate(test_request()).await.unwrap();
        let built = request.build().unwrap();
        let auth_header = built
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(auth_header, "Bearer jwt-token");
    }
}
