use crate::airflow::client::auth::AuthProvider;
use crate::airflow::config::{AirflowAuth, AirflowConfig, ManagedService};
use anyhow::{Context, Result};
use async_trait::async_trait;
use dirs::home_dir;
use log::info;
use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Timeout in seconds for conveyor CLI commands.
/// If the CLI hangs (e.g., waiting for input), the operation will fail after this duration.
const CONVEYOR_TIMEOUT_SECS: u64 = 30;

/// Run a conveyor command with timeout, returning stdout on success.
fn run_conveyor_command(args: &[&str]) -> Result<String> {
    let mut child = Command::new("conveyor")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn conveyor {} command", args.join(" ")))?;

    let mut stdout = child.stdout.take().context("Failed to capture stdout")?;

    let (tx, rx) = mpsc::channel();

    // Spawn a thread to read stdout - this allows us to implement a timeout
    // since the read operation itself is blocking
    thread::spawn(move || {
        let mut output = String::new();
        let result = stdout.read_to_string(&mut output).map(|_| output);
        let _ = tx.send(result);
    });

    let timeout = Duration::from_secs(CONVEYOR_TIMEOUT_SECS);
    let cmd_desc = args.join(" ");

    let error = match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            let status = child.wait().context("Failed to wait for process")?;
            if !status.success() {
                let stderr = read_stderr(&mut child);
                anyhow::bail!("conveyor {cmd_desc} failed: {stderr}");
            }
            return Ok(output);
        }
        Ok(Err(e)) => Err(e).with_context(|| format!("Failed to read conveyor {cmd_desc} output")),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(anyhow::anyhow!(
            "Conveyor {cmd_desc} command timed out after {CONVEYOR_TIMEOUT_SECS} seconds. \
             The conveyor CLI may be hung or waiting for input."
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(anyhow::anyhow!("Reader thread disconnected unexpectedly"))
        }
    };

    // Kill and reap child process to prevent zombies
    let _ = child.kill();
    let _ = child.wait();
    error
}

fn read_stderr(child: &mut Child) -> String {
    child
        .stderr
        .take()
        .and_then(|mut s| {
            let mut buf = String::new();
            s.read_to_string(&mut buf).ok().map(|_| buf)
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct ConveyorClient {}

impl ConveyorClient {
    pub fn get_token() -> Result<String> {
        let output = run_conveyor_command(&["auth", "get", "--quiet"])?;
        let token = serde_json::from_str::<ConveyorTokenResponse>(&output)
            .context("Failed to parse JSON token from conveyor output")?
            .access_token;
        Ok(token)
    }
}

#[derive(Debug, Clone)]
pub struct ConveyorAuthProvider;

#[async_trait]
impl AuthProvider for ConveyorAuthProvider {
    async fn authenticate(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        info!("🔑 Conveyor Auth");
        let token = ConveyorClient::get_token()?;
        Ok(request.bearer_auth(token))
    }

    fn clone_box(&self) -> Box<dyn AuthProvider> {
        Box::new(self.clone())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConveyorEnvironment {
    pub name: String,
    #[serde(rename = "clusterName")]
    pub cluster_name: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: String,
    #[serde(rename = "airflowVersion")]
    pub airflow_version: String,
}

pub fn list_conveyor_environments() -> Result<Vec<ConveyorEnvironment>> {
    // Ensure authentication before listing environments
    ConveyorClient::get_token()?;

    let output = run_conveyor_command(&["environment", "list", "-o", "json"])?;
    let environments: Vec<ConveyorEnvironment> = serde_json::from_str(&output)
        .context("Failed to parse conveyor environment list output")?;

    info!("Found {} Conveyor environment(s)", environments.len());
    Ok(environments)
}

pub fn get_conveyor_environment_servers() -> Result<Vec<AirflowConfig>> {
    let environments = list_conveyor_environments()?;
    let api_endpoint = get_conveyor_api_endpoint()?;

    let servers = environments
        .iter()
        .map(|env| {
            let version = match env.airflow_version.as_str() {
                "AirflowVersion_V3" => crate::airflow::config::AirflowVersion::V3,
                _ => crate::airflow::config::AirflowVersion::V2,
            };
            AirflowConfig {
                name: env.name.clone(),
                endpoint: format!("{}/environments/{}/airflow/", api_endpoint, env.name),
                auth: AirflowAuth::Conveyor,
                managed: Some(ManagedService::Conveyor),
                version,
                timeout_secs: 30,
            }
        })
        .collect();
    Ok(servers)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConveyorTokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
struct ConveyorProfiles {
    activeprofile: String,
    #[serde(rename = "version")]
    _version: Option<i8>,
    #[serde(flatten)]
    profiles: std::collections::HashMap<String, ConveyorProfile>,
}

#[derive(Deserialize, Debug)]
struct ConveyorProfile {
    api: String,
}

fn get_conveyor_api_endpoint() -> Result<String> {
    let profiles_path = home_dir()
        .context("Could not determine home directory")?
        .join(".conveyor/profiles.toml");

    let profiles_content = std::fs::read_to_string(&profiles_path)
        .context("Failed to read ~/.conveyor/profiles.toml")?;

    let profiles_config: ConveyorProfiles =
        toml::from_str(&profiles_content).context("Failed to parse profiles.toml")?;

    if profiles_config.activeprofile.as_str() == "default" {
        return Ok("https://app.conveyordata.com".to_string());
    }

    let active_profile = profiles_config
        .profiles
        .get(&profiles_config.activeprofile)
        .context(format!(
            "Active profile '{}' not found in profiles.toml",
            profiles_config.activeprofile
        ))?;

    Ok(active_profile.api.clone())
}
