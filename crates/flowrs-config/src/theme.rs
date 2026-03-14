use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Controls which theme is used. Serialized in the config file.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Display, ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
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
