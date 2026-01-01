use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::app::environment_state::EnvironmentData;
use crate::app::model::popup::error::ErrorPopup;
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
        app.configs.error_popup = Some(ErrorPopup::from_strings(vec![format!(
            "Configuration index {idx} not found"
        )]));
        app.loading = false;
        return Ok(());
    };
    let env_name = selected_config.name.clone();

    // Check if environment already exists, if not create it
    if !app.environment_state.environments.contains_key(&env_name) {
        match crate::airflow::client::create_client(&selected_config) {
            Ok(client) => {
                let env_data = EnvironmentData::new(client);
                app.environment_state
                    .add_environment(env_name.clone(), env_data);
            }
            Err(e) => {
                log::error!("Failed to create client for '{env_name}': {e}");
                app.configs.error_popup = Some(ErrorPopup::from_strings(vec![format!(
                    "Failed to connect to '{env_name}': {e}"
                )]));
                app.loading = false;
                return Ok(());
            }
        }
    }

    // Set this as the active environment
    app.environment_state
        .set_active_environment(env_name.clone());
    app.config.active_server = Some(env_name);

    // Clear the view state but NOT the environment data
    app.clear_state();

    // Sync panel data from the new environment
    app.sync_panel_data();
    app.loading = false;
    Ok(())
}
