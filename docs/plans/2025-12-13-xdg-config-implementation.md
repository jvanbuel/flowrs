# XDG Configuration Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add XDG Base Directory Specification support for configuration with backwards compatibility for `~/.flowrs`.

**Architecture:** Create a `ConfigPaths` struct that resolves read/write paths at startup. XDG path takes precedence for reads; all writes go to XDG. Show a warning popup when both config files exist.

**Tech Stack:** Rust, ratatui, dirs crate (already in use)

---

### Task 1: Create ConfigPaths Module

**Files:**
- Create: `src/airflow/config/paths.rs`
- Modify: `src/airflow/config/mod.rs` (add module export)

**Step 1: Create the paths module with ConfigPaths struct**

Create `src/airflow/config/paths.rs`:

```rust
use std::path::PathBuf;

use dirs::{config_dir, home_dir};

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
    /// Falls back to `~/.config/flowrs/config.toml` if XDG_CONFIG_HOME is unset or empty.
    fn xdg_config_path() -> PathBuf {
        config_dir()
            .unwrap_or_else(|| home_dir().unwrap().join(".config"))
            .join("flowrs")
            .join("config.toml")
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
```

**Step 2: Export the module from config/mod.rs**

Add to `src/airflow/config/mod.rs` after line 1:

```rust
pub mod paths;
```

**Step 3: Run tests to verify**

Run: `cargo test paths --lib`
Expected: PASS

**Step 4: Commit**

```bash
git add src/airflow/config/paths.rs src/airflow/config/mod.rs
git commit -m "feat: add ConfigPaths module for XDG config support"
```

---

### Task 2: Update main.rs to Use ConfigPaths

**Files:**
- Modify: `src/main.rs:15-17`

**Step 1: Replace CONFIG_FILE with CONFIG_PATHS**

In `src/main.rs`, replace lines 15-17:

```rust
use dirs::home_dir;

static CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| home_dir().unwrap().join(".flowrs"));
```

With:

```rust
use airflow::config::paths::ConfigPaths;

pub static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compile errors in `src/airflow/config/mod.rs` (expected - we'll fix those next)

---

### Task 3: Update Config Module to Use ConfigPaths

**Files:**
- Modify: `src/airflow/config/mod.rs:14,118,128,209`

**Step 1: Update the import**

In `src/airflow/config/mod.rs`, replace line 14:

```rust
use crate::CONFIG_FILE;
```

With:

```rust
use crate::CONFIG_PATHS;
```

**Step 2: Update FlowrsConfig::new() to use write_path**

Replace line 118:

```rust
            path: Some(CONFIG_FILE.as_path().to_path_buf()),
```

With:

```rust
            path: Some(CONFIG_PATHS.write_path.clone()),
```

**Step 3: Update FlowrsConfig::from_file() to use read_path**

Replace lines 127-130:

```rust
            .unwrap_or_else(|| {
                // No valid path was provided by the user, use the default path
                let default_path = CONFIG_FILE.as_path().to_path_buf();
                info!("Using configuration path: {}", default_path.display());
                default_path
            });
```

With:

```rust
            .unwrap_or_else(|| {
                // No valid path was provided by the user, use the default read path
                let default_path = CONFIG_PATHS.read_path.clone();
                info!("Using configuration path: {}", default_path.display());
                default_path
            });
```

**Step 4: Update write_to_file() to use write_path and create directory**

Replace lines 205-215:

```rust
    pub fn write_to_file(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or(CONFIG_FILE.as_path().to_path_buf());
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
```

With:

```rust
    pub fn write_to_file(&mut self) -> Result<()> {
        let path = self
            .path
            .clone()
            .unwrap_or_else(|| CONFIG_PATHS.write_path.clone());

        // Create parent directory if it doesn't exist (for XDG path)
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)?;
```

**Step 5: Update test to use CONFIG_PATHS**

Replace the test at line 311:

```rust
        assert_eq!(config.path.unwrap(), CONFIG_FILE.as_path().to_path_buf());
```

With:

```rust
        assert_eq!(config.path.unwrap(), CONFIG_PATHS.write_path);
```

**Step 6: Verify it compiles and tests pass**

Run: `cargo test --lib`
Expected: PASS

**Step 7: Commit**

```bash
git add src/main.rs src/airflow/config/mod.rs
git commit -m "feat: migrate from CONFIG_FILE to CONFIG_PATHS"
```

---

### Task 4: Create WarningPopup Component

**Files:**
- Create: `src/app/model/popup/warning.rs`
- Modify: `src/app/model/popup/mod.rs`

**Step 1: Create the warning popup**

Create `src/app/model/popup/warning.rs`:

```rust
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};

use super::popup_area;

pub struct WarningPopup {
    pub warnings: Vec<String>,
}

impl WarningPopup {
    pub fn new(warnings: Vec<String>) -> Self {
        Self { warnings }
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Widget for &WarningPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.warnings.is_empty() {
            return;
        }

        let popup_area = popup_area(area, 80, 50);
        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .title("Warning - Press <Esc> or <q> to close")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        Clear.render(popup_area, buf);

        let mut text = Text::default();
        for (idx, warning) in self.warnings.iter().enumerate() {
            text.push_line(Line::from(vec![
                Span::styled(
                    format!("Warning {}: ", idx + 1),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(warning.as_str(), Style::default().fg(Color::White)),
            ]));
            if idx < self.warnings.len() - 1 {
                text.push_line(Line::from(""));
            }
        }

        let warning_paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(popup);
        warning_paragraph.render(popup_area, buf);
    }
}
```

**Step 2: Export the module from popup/mod.rs**

Add to `src/app/model/popup/mod.rs` after line 5:

```rust
pub mod warning;
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: PASS

**Step 4: Commit**

```bash
git add src/app/model/popup/warning.rs src/app/model/popup/mod.rs
git commit -m "feat: add WarningPopup component with yellow styling"
```

---

### Task 5: Add Warning Popup to ConfigModel

**Files:**
- Modify: `src/app/model/config.rs`

**Step 1: Add import for WarningPopup**

Add after line 16 in `src/app/model/config.rs`:

```rust
use super::popup::warning::WarningPopup;
```

**Step 2: Add warning_popup field to ConfigModel**

After line 25 (`pub error_popup: Option<ErrorPopup>,`), add:

```rust
    pub warning_popup: Option<WarningPopup>,
```

**Step 3: Update ConfigModel::new() to initialize warning_popup**

In the `new` function (lines 29-37), add after `error_popup: None,`:

```rust
            warning_popup: None,
```

**Step 4: Add new constructor for warnings**

After the `new_with_errors` function (after line 53), add:

```rust
    pub fn new_with_errors_and_warnings(
        configs: Vec<AirflowConfig>,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) -> Self {
        let error_popup = if errors.is_empty() {
            None
        } else {
            Some(ErrorPopup::from_strings(errors))
        };

        let warning_popup = if warnings.is_empty() {
            None
        } else {
            Some(WarningPopup::new(warnings))
        };

        ConfigModel {
            all: configs.clone(),
            filtered: StatefulTable::new(configs),
            filter: Filter::new(),
            commands: None,
            error_popup,
            warning_popup,
        }
    }
```

**Step 5: Update new_with_errors to initialize warning_popup**

In `new_with_errors` (around line 52), add `warning_popup: None,` to the struct initialization.

**Step 6: Update update() to handle warning popup dismissal**

In the `update` method, after the error_popup handling block (around line 80-86), add:

```rust
                } else if self.warning_popup.is_some() {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.warning_popup = None;
                        }
                        _ => (),
                    }
```

**Step 7: Update render to display warning popup**

At the end of the `render` implementation (after the error_popup render, around line 213), add:

```rust
        if let Some(warning_popup) = &self.warning_popup {
            warning_popup.render(area, buf);
        }
```

**Step 8: Verify it compiles**

Run: `cargo build`
Expected: PASS

**Step 9: Commit**

```bash
git add src/app/model/config.rs
git commit -m "feat: add warning_popup support to ConfigModel"
```

---

### Task 6: Wire Up Legacy Config Warning

**Files:**
- Modify: `src/app/state.rs`
- Modify: `src/commands/run.rs`

**Step 1: Update App::new_with_errors to accept warnings**

In `src/app/state.rs`, rename and update the function signature at line 39:

```rust
    pub fn new_with_errors_and_warnings(
        config: FlowrsConfig,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) -> Self {
```

**Step 2: Update the ConfigModel initialization**

Replace line 50:

```rust
            configs: ConfigModel::new_with_errors(servers.clone(), errors),
```

With:

```rust
            configs: ConfigModel::new_with_errors_and_warnings(servers.clone(), errors, warnings),
```

**Step 3: Update App::new to use new signature**

Replace lines 34-37:

```rust
    #[allow(dead_code)]
    pub fn new(config: FlowrsConfig) -> Self {
        Self::new_with_errors(config, vec![])
    }
```

With:

```rust
    #[allow(dead_code)]
    pub fn new(config: FlowrsConfig) -> Self {
        Self::new_with_errors_and_warnings(config, vec![], vec![])
    }
```

**Step 4: Update run.rs to generate and pass warnings**

In `src/commands/run.rs`, add import at the top:

```rust
use crate::CONFIG_PATHS;
```

**Step 5: Generate warnings and pass to App**

Replace lines 29-35:

```rust
        let (config, errors) = FlowrsConfig::from_file(path.as_ref())?
            .expand_managed_services()
            .await?;

        // setup terminal (includes panic hooks) and run app
        let mut terminal = ratatui::init();
        let app = App::new_with_errors(config, errors);
```

With:

```rust
        let (config, errors) = FlowrsConfig::from_file(path.as_ref())?
            .expand_managed_services()
            .await?;

        // Generate warnings for legacy config conflict
        let mut warnings = Vec::new();
        if CONFIG_PATHS.has_legacy_conflict {
            warnings.push(format!(
                "Configuration file found in both locations:\n  \
                 - {} (active)\n  \
                 - {} (ignored)\n\n\
                 Consider removing the legacy file.",
                CONFIG_PATHS.write_path.display(),
                dirs::home_dir().unwrap().join(".flowrs").display()
            ));
        }

        // setup terminal (includes panic hooks) and run app
        let mut terminal = ratatui::init();
        let app = App::new_with_errors_and_warnings(config, errors, warnings);
```

**Step 6: Add dirs import to run.rs**

Add at the top of `src/commands/run.rs`:

```rust
use dirs;
```

**Step 7: Verify it compiles**

Run: `cargo build`
Expected: PASS

**Step 8: Commit**

```bash
git add src/app/state.rs src/commands/run.rs
git commit -m "feat: wire up legacy config warning through app initialization"
```

---

### Task 7: Update Documentation

**Files:**
- Modify: `README.md`
- Modify: `CLAUDE.md`

**Step 1: Update README.md**

Find the section mentioning `~/.flowrs` and update to explain both locations.

**Step 2: Update CLAUDE.md**

Update the Configuration section (around line 31) to reflect the new XDG path behavior.

**Step 3: Commit**

```bash
git add README.md CLAUDE.md
git commit -m "docs: update configuration path documentation for XDG support"
```

---

### Task 8: Manual Testing

**Step 1: Test with no config files**

```bash
rm -f ~/.flowrs
rm -rf ~/.config/flowrs
cargo run
```
Expected: App starts normally, no warning popup

**Step 2: Test with only legacy config**

```bash
echo '[[servers]]
name = "test"
endpoint = "http://localhost:8080"

[servers.auth.Basic]
username = "airflow"
password = "airflow"' > ~/.flowrs
rm -rf ~/.config/flowrs
cargo run
```
Expected: App starts, reads from `~/.flowrs`, no warning

**Step 3: Test config add creates XDG path**

```bash
rm -f ~/.flowrs
rm -rf ~/.config/flowrs
# Run flowrs config add and add a server
# Then verify ~/.config/flowrs/config.toml was created
ls -la ~/.config/flowrs/
```
Expected: `~/.config/flowrs/config.toml` exists

**Step 4: Test both configs show warning**

```bash
# Keep the XDG config from step 3
# Also create legacy config
echo '[[servers]]
name = "legacy"
endpoint = "http://localhost:8081"

[servers.auth.Basic]
username = "airflow"
password = "airflow"' > ~/.flowrs
cargo run
```
Expected: App starts with yellow warning popup about both configs existing

**Step 5: Commit final changes if any fixes needed**

```bash
git add -A
git commit -m "fix: address issues found during manual testing"
```
