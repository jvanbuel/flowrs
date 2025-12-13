# XDG Configuration Support

## Overview

Add XDG Base Directory Specification support for flowrs configuration, with backwards compatibility for the legacy `~/.flowrs` config file.

## Config Paths

**XDG Path:** `$XDG_CONFIG_HOME/flowrs/config.toml`
- Falls back to `~/.config/flowrs/config.toml` if `$XDG_CONFIG_HOME` is unset or empty

**Legacy Path:** `~/.flowrs`

## Read Behavior

Resolution order:
1. If XDG path exists → use it
2. Else if legacy path exists → use it
3. Else → no config found

If **both** paths exist, use XDG but display a warning popup.

## Write Behavior

All writes go to the XDG path. Create the directory `$XDG_CONFIG_HOME/flowrs/` if it doesn't exist.

## Warning Popup

When both config files exist, display a warning popup on the Config panel:

- Yellow/amber border (distinct from red error popups)
- Title: "Warning - Press <Esc> or <q> to close"
- Message:
  ```
  Configuration file found in both locations:
    - ~/.config/flowrs/config.toml (active)
    - ~/.flowrs (ignored)

  Consider removing the legacy file at ~/.flowrs
  ```
- Dismissed once per session (won't reappear until next launch)

## Code Changes

### New Module: `src/airflow/config/paths.rs`

```rust
pub struct ConfigPaths {
    pub read_path: PathBuf,        // Where to read config from
    pub write_path: PathBuf,       // Where to write config to (always XDG)
    pub has_legacy_conflict: bool, // True if both paths exist
}

impl ConfigPaths {
    pub fn resolve() -> Self { ... }
}
```

### Changes to `src/main.rs`

Replace:
```rust
static CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| home_dir().unwrap().join(".flowrs"));
```

With:
```rust
static CONFIG_PATHS: LazyLock<ConfigPaths> = LazyLock::new(ConfigPaths::resolve);
```

### Changes to `src/airflow/config/mod.rs`

Update all references from `CONFIG_FILE` to use `CONFIG_PATHS.read_path` or `CONFIG_PATHS.write_path` as appropriate.

### New File: `src/app/model/popup/warning.rs`

Similar structure to `error.rs`, but with yellow styling.

### Changes to Config Panel (`src/app/model/config.rs`)

Add `warning_popup: Option<WarningPopup>` field, initialized based on `CONFIG_PATHS.has_legacy_conflict`.

## Edge Cases

1. **`$XDG_CONFIG_HOME` is set but empty string** → Treat as unset, fall back to `~/.config`

2. **XDG directory exists but `config.toml` doesn't** → Fall through to legacy path

3. **Neither config exists** → No warning, app starts normally

4. **Write fails due to permissions** → Existing error handling surfaces this

## Dependencies

No new dependencies. The `dirs` crate already provides `dirs::config_dir()` which returns the appropriate XDG path.

## Documentation Updates

- Update `README.md` to mention both config locations
- Update `CLAUDE.md` to reflect the new config path behavior
