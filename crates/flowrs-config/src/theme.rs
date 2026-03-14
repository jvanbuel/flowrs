use serde::{Deserialize, Serialize};

/// Controls which theme is used. Serialized in the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    /// Automatically detect terminal background (dark or light)
    #[default]
    Auto,
    /// Force dark theme
    Dark,
    /// Force light theme
    Light,
    /// Catppuccin Latte (light)
    CatppuccinLatte,
    /// Catppuccin Frappé (medium-dark)
    CatppuccinFrappe,
    /// Catppuccin Macchiato (dark)
    CatppuccinMacchiato,
    /// Catppuccin Mocha (darkest)
    CatppuccinMocha,
}

impl ThemeMode {
    /// Returns true if this is the `Auto` variant (used for `skip_serializing_if`).
    pub fn is_auto(&self) -> bool {
        *self == Self::Auto
    }
}
