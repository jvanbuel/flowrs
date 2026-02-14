use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::airflow::model::common::EnvironmentKey;
use crate::app::environment_state::EnvironmentData;
use crate::app::state::App;

/// Handle configuration selection.
/// Creates a new client for the selected configuration if needed and sets it as active.
pub fn handle_config_selected(app: &Arc<Mutex<App>>, idx: usize) -> Result<()> {
    let mut app = app
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire app lock"))?;

    let Some(selected_config) = app.configs.table.filtered.items.get(idx).cloned() else {
        log::error!(
            "Config index {idx} out of bounds (total: {})",
            app.configs.table.filtered.items.len()
        );
        app.configs
            .popup
            .show_error(vec![format!("Configuration index {idx} not found")]);
        app.loading = false;
        return Ok(());
    };
    let env_name = EnvironmentKey::from(selected_config.name.clone());

    // Check if environment already exists, if not create it
    if !app.environment_state.environments.contains_key(&env_name) {
        match crate::airflow::client::create_client(&selected_config) {
            Ok(client) => {
                let env_data = EnvironmentData::new(client);
                app.environment_state
                    .environments
                    .insert(env_name.clone(), env_data);
            }
            Err(e) => {
                log::error!("Failed to create client for '{env_name}': {e}");
                app.configs
                    .popup
                    .show_error(vec![format!("Failed to connect to '{env_name}': {e}")]);
                app.loading = false;
                return Ok(());
            }
        }
    }

    // Set this as the active environment
    app.environment_state
        .set_active_environment(env_name.clone());
    app.config.active_server = Some(env_name.to_string());
    app.nav_context = crate::app::state::NavigationContext::Environment {
        environment: env_name.to_string(),
    };

    // Clear the view state but NOT the environment data
    app.clear_state();

    // Sync panel data from the new environment
    let panel = app.active_panel.clone();
    app.sync_panel(&panel);
    app.loading = false;
    Ok(())
}
