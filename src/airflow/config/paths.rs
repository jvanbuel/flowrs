use std::path::PathBuf;

use dirs::home_dir;

/// Manages configuration file paths with XDG support and legacy fallback.
pub struct ConfigPaths {
    /// Path to read config from (XDG if exists, else legacy)
    pub read_path: PathBuf,
    /// Path to write config to (always XDG)
    pub write_path: PathBuf,
    /// True if both XDG and legacy config files exist
    pub has_legacy_conflict: bool,
}

impl ConfigPaths {
    /// Resolves configuration paths according to XDG spec with legacy fallback.
    ///
    /// Read precedence:
    /// 1. `$XDG_CONFIG_HOME/flowrs/config.toml` (or `~/.config/flowrs/config.toml`)
    /// 2. `~/.flowrs` (legacy)
    ///
    /// Write always goes to XDG path.
    pub fn resolve() -> Self {
        let xdg_path = Self::xdg_config_path();
        let legacy_path = Self::legacy_config_path();

        let xdg_exists = xdg_path.exists();
        let legacy_exists = legacy_path.exists();

        let read_path = if xdg_exists {
            xdg_path.clone()
        } else if legacy_exists {
            legacy_path.clone()
        } else {
            // Neither exists - default to XDG for new configs
            xdg_path.clone()
        };

        ConfigPaths {
            read_path,
            write_path: xdg_path,
            has_legacy_conflict: xdg_exists && legacy_exists,
        }
    }

    /// Returns the XDG config path: `$XDG_CONFIG_HOME/flowrs/config.toml`
    /// Falls back to `~/.config/flowrs/config.toml` if `XDG_CONFIG_HOME` is unset or empty.
    fn xdg_config_path() -> PathBuf {
        // Check XDG_CONFIG_HOME first, fall back to ~/.config (per XDG spec)
        let base_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .filter(|s| !s.is_empty())
            .map_or_else(|| home_dir().unwrap().join(".config"), PathBuf::from);

        base_dir.join("flowrs").join("config.toml")
    }

    /// Returns the legacy config path: `~/.flowrs`
    fn legacy_config_path() -> PathBuf {
        home_dir().unwrap().join(".flowrs")
    }

    /// Returns the XDG config directory (for creating if needed).
    pub fn xdg_config_dir(&self) -> PathBuf {
        self.write_path.parent().unwrap().to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xdg_config_path_structure() {
        let path = ConfigPaths::xdg_config_path();
        assert!(path.ends_with("flowrs/config.toml"));
    }

    #[test]
    fn test_legacy_config_path_structure() {
        let path = ConfigPaths::legacy_config_path();
        assert!(path.ends_with(".flowrs"));
    }
}
