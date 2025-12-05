use crate::airflow::config::{AirflowAuth, AirflowConfig, AirflowVersion, ManagedService};
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_sdk_mwaa as mwaa;
use log::info;
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

    /// Exchanges a web login token for a session cookie
    pub async fn get_session_cookie(&self, web_token: &MwaaWebToken) -> Result<String> {
        let login_url = format!("https://{}/aws_mwaa/login", web_token.hostname);

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

        // Extract the session cookie from Set-Cookie header
        let cookies = response.headers().get_all("set-cookie");

        for cookie_header in cookies {
            let cookie_str = cookie_header.to_str().context("Invalid cookie header")?;

            // Parse the cookie to extract session value
            if let Some(session_part) = cookie_str.split(';').next() {
                if let Some(("session", value)) = session_part.split_once('=') {
                    return Ok(value.to_string());
                }
            }
        }

        anyhow::bail!("No session cookie found in response")
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

/// MWAA authentication data including session cookie
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MwaaAuth {
    pub session_cookie: String,
    pub environment_name: String,
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

        // Create web token and session for this environment
        let web_token = client.create_web_login_token(&env_name).await?;
        let session_cookie = client.get_session_cookie(&web_token).await?;

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
                session_cookie,
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

    #[tokio::test]
    async fn test_list_mwaa_environments() {
        let result = get_mwaa_environment_servers().await;
        // This test will only work if you have MWAA environments configured
        // and AWS credentials available
        if let Ok(environments) = result {
            println!("Found {} MWAA environments", environments.len());
            for env in environments {
                println!("  - {} ({})", env.name, env.endpoint);
            }
        }
    }

    #[tokio::test]
    async fn test_mwaa_client_new() {
        let client = MwaaClient::new().await;
        assert!(client.is_ok());
    }
}
