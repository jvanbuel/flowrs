// Clippy pedantic allows
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

pub mod auth;
pub mod paths;
pub mod server;
pub mod theme;

// Re-export all public types at crate root for ergonomic imports
pub use auth::{AirflowAuth, BasicAuth, TokenSource};
pub use paths::ConfigPaths;
pub use server::{AirflowConfig, AirflowVersion, GccConfig, ManagedService};
pub use theme::Theme;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

const TICK_RATE_MS: u64 = 200;
const MIN_POLL_INTERVAL_MS: u64 = 500;

/// Process-local counter that keeps atomic-write temp filenames unique.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const fn default_poll_interval_ms() -> u64 {
    2000
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FlowrsConfig {
    #[serde(default)]
    pub servers: Vec<AirflowConfig>,
    #[serde(default)]
    pub managed_services: Vec<ManagedService>,
    pub active_server: Option<String>,
    /// API poll interval in milliseconds. Controls how often the TUI refreshes
    /// data from the Airflow API. Minimum 500ms, default 2000ms.
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    /// Theme: "auto" (detect terminal background), "dark", "light", "catppuccin-latte",
    /// "catppuccin-frappe", "catppuccin-macchiato", or "catppuccin-mocha".
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub gcc: Option<GccConfig>,
    #[serde(skip_serializing)]
    pub path: Option<PathBuf>,
}

impl FlowrsConfig {
    /// Creates a new `FlowrsConfig` with default values.
    ///
    /// Returns a `FlowrsConfig` with:
    /// - No servers configured
    /// - No managed services
    /// - No active server
    /// - Default config file path derived from the provided `ConfigPaths`
    pub fn new(config_paths: &ConfigPaths) -> Self {
        Self {
            servers: Vec::new(),
            managed_services: Vec::new(),
            active_server: None,
            poll_interval_ms: default_poll_interval_ms(),
            theme: Theme::default(),
            gcc: None,
            path: Some(config_paths.write_path.clone()),
        }
    }

    /// Compute the tick multiplier for API polling based on `poll_interval_ms`.
    /// Clamps values below the minimum and logs a warning.
    pub fn poll_tick_multiplier(&self) -> u32 {
        let interval = if self.poll_interval_ms < MIN_POLL_INTERVAL_MS {
            log::warn!(
                "poll_interval_ms ({}) is below minimum ({}), clamping",
                self.poll_interval_ms,
                MIN_POLL_INTERVAL_MS
            );
            MIN_POLL_INTERVAL_MS
        } else {
            self.poll_interval_ms
        };
        #[allow(clippy::cast_possible_truncation)]
        let multiplier = (interval / TICK_RATE_MS) as u32;
        multiplier.max(1)
    }

    pub fn from_file(config_path: Option<&PathBuf>, config_paths: &ConfigPaths) -> Result<Self> {
        let path = config_path
            .filter(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| {
                // No valid path was provided by the user, use the default read path
                let default_path = config_paths.read_path.clone();
                info!("Using configuration path: {}", default_path.display());
                default_path
            });

        // Only a missing file becomes an empty config; any other read error must
        // surface, since silently returning empty would clobber the real file on
        // the next write.
        let toml_config = match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Err(anyhow::Error::new(e)
                    .context(format!("failed to read config file {}", path.display())))
            }
        };
        let mut config = Self::parse_toml(&toml_config)?;
        config.path = Some(path);
        Ok(config)
    }

    pub fn parse_toml(config: &str) -> Result<Self> {
        let config: Self = toml::from_str(config)?;
        info!(
            "Loaded config: servers={}, managed_services={}",
            config.servers.len(),
            config.managed_services.len()
        );
        Ok(config)
    }

    pub fn extend_servers<I>(&mut self, new_servers: I)
    where
        I: IntoIterator<Item = AirflowConfig>,
    {
        self.servers.extend(new_servers);
    }

    pub fn to_str(&self) -> Result<String> {
        toml::to_string(self).map_err(std::convert::Into::into)
    }

    pub fn write_to_file(&self, config_paths: &ConfigPaths) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or_else(|| config_paths.write_path.clone());

        // Create parent directory if it doesn't exist (for XDG path)
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut persisted = self.clone();
        persisted.servers.retain(|server| server.managed.is_none());
        let contents = persisted.to_str()?;

        // Write atomically via a temp file + rename, so a serialization error or
        // crash can't leave the config truncated: the previous file survives
        // until the final rename. The pid and a process-local counter keep the
        // temp name unique across concurrent writers.
        let unique = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = path.with_extension(format!("tmp.{}.{unique}", std::process::id()));

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        // The config can hold credentials, so restrict it to the owner; creating
        // the temp file with 0o600 carries that protection through the rename.
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&tmp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONFIG: &str = r#"[[servers]]
        name = "test"
        endpoint = "http://localhost:8080"

        [servers.auth.Basic]
        username = "airflow"
        password = "airflow"
        "#;

    #[test]
    fn test_get_config() {
        let result = FlowrsConfig::parse_toml(TEST_CONFIG).unwrap();
        assert_eq!(result.servers.len(), 1);
        assert_eq!(result.servers[0].name, "test");
    }

    const TEST_CONFIG_CONVEYOR: &str = r#"
managed_services = ["Conveyor"]
poll_interval_ms = 2000
theme = "auto"

[[servers]]
name = "bla"
endpoint = "http://localhost:8080"
version = "V2"
timeout_secs = 30
insecure = false

[servers.auth.Basic]
username = "airflow"
password = "airflow"
    "#;
    #[test]
    fn test_get_config_conveyor() {
        let result = FlowrsConfig::parse_toml(TEST_CONFIG_CONVEYOR.trim()).unwrap();
        assert_eq!(result.managed_services.len(), 1);
        assert_eq!(result.managed_services[0], ManagedService::Conveyor);
    }

    #[test]
    fn test_write_config_conveyor() {
        use server::default_timeout;

        let config = FlowrsConfig {
            servers: vec![AirflowConfig {
                name: "bla".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                auth: AirflowAuth::Basic(BasicAuth {
                    username: "airflow".to_string(),
                    password: "airflow".to_string(),
                }),
                managed: None,
                version: AirflowVersion::V2,
                timeout_secs: default_timeout(),
                insecure: false,
            }],
            managed_services: vec![ManagedService::Conveyor],
            active_server: None,
            poll_interval_ms: default_poll_interval_ms(),
            theme: Theme::default(),
            gcc: None,
            path: None,
        };

        let serialized_config = config.to_str().unwrap();
        assert_eq!(serialized_config.trim(), TEST_CONFIG_CONVEYOR.trim());
    }

    #[test]
    fn non_existing_path() {
        let config_paths = ConfigPaths::resolve();
        let path = PathBuf::from("non-existing.toml");
        let config = FlowrsConfig::from_file(Some(&path), &config_paths);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert!(config.path.is_some());
    }

    #[test]
    fn none_path() {
        let config_paths = ConfigPaths::resolve();
        let config = FlowrsConfig::from_file(None, &config_paths);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.path.unwrap(), config_paths.read_path);
    }

    #[test]
    fn test_poll_interval_ms_default() {
        let config = FlowrsConfig::parse_toml(TEST_CONFIG).unwrap();
        assert_eq!(config.poll_interval_ms, 2000);
        assert_eq!(config.poll_tick_multiplier(), 10);
    }

    #[test]
    fn test_poll_interval_ms_custom() {
        let toml = r#"
poll_interval_ms = 5000

[[servers]]
name = "test"
endpoint = "http://localhost:8080"

[servers.auth.Basic]
username = "airflow"
password = "airflow"
"#;
        let config = FlowrsConfig::parse_toml(toml).unwrap();
        assert_eq!(config.poll_interval_ms, 5000);
        assert_eq!(config.poll_tick_multiplier(), 25);
    }

    fn unique_temp_path(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("flowrs-cfg-test-{}-{tag}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.toml")
    }

    fn basic_server(name: &str, managed: Option<ManagedService>) -> AirflowConfig {
        AirflowConfig {
            name: name.to_string(),
            endpoint: "http://localhost:8080".to_string(),
            auth: AirflowAuth::Basic(BasicAuth {
                username: "airflow".to_string(),
                password: "airflow".to_string(),
            }),
            managed,
            version: AirflowVersion::V2,
            timeout_secs: server::default_timeout(),
            insecure: false,
        }
    }

    fn config_with(servers: Vec<AirflowConfig>, path: PathBuf) -> FlowrsConfig {
        FlowrsConfig {
            servers,
            managed_services: Vec::new(),
            active_server: None,
            poll_interval_ms: default_poll_interval_ms(),
            theme: Theme::default(),
            gcc: None,
            path: Some(path),
        }
    }

    #[test]
    fn write_to_file_excludes_managed_servers() {
        let path = unique_temp_path("managed");
        let paths = ConfigPaths::resolve();
        let config = config_with(
            vec![
                basic_server("managed", Some(ManagedService::Conveyor)),
                basic_server("local", None),
            ],
            path.clone(),
        );

        config.write_to_file(&paths).unwrap();

        let persisted = FlowrsConfig::from_file(Some(&path), &paths).unwrap();
        assert_eq!(persisted.servers.len(), 1);
        assert_eq!(persisted.servers[0].name, "local");
    }

    #[cfg(unix)]
    #[test]
    fn write_to_file_restricts_permissions_to_owner() {
        use std::os::unix::fs::PermissionsExt;
        let path = unique_temp_path("perms");
        let paths = ConfigPaths::resolve();
        let config = config_with(vec![basic_server("local", None)], path.clone());

        config.write_to_file(&paths).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn from_file_propagates_read_error_other_than_not_found() {
        let path = unique_temp_path("bad-utf8");
        // Invalid UTF-8 fails `read_to_string` with `InvalidData`, exercising a
        // read error that is not `NotFound`.
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        let paths = ConfigPaths::resolve();

        let result = FlowrsConfig::from_file(Some(&path), &paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_poll_interval_ms_clamped_to_minimum() {
        let toml = r#"
poll_interval_ms = 100

[[servers]]
name = "test"
endpoint = "http://localhost:8080"

[servers.auth.Basic]
username = "airflow"
password = "airflow"
"#;
        let config = FlowrsConfig::parse_toml(toml).unwrap();
        assert_eq!(config.poll_interval_ms, 100);
        // Should clamp to 500ms minimum → 500/200 = 2
        assert_eq!(config.poll_tick_multiplier(), 2);
    }
}
