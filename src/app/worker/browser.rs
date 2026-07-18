use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};

use crate::airflow::model::common::OpenItem;
use crate::airflow::traits::AirflowClient;
use crate::app::state::App;

/// Open an item (DAG, DAG run, task instance, etc.) in the browser.
///
/// Any failure (no active server, an unbuildable URL, or the browser refusing
/// to launch) is surfaced to the user via the active panel's error popup
/// instead of being silently logged.
pub fn handle_open_item(app: &Arc<Mutex<App>>, client: &Arc<dyn AirflowClient>, item: OpenItem) {
    if let Err(e) = try_open_item(app, client, item) {
        app.lock().unwrap().show_error(vec![e.to_string()]);
    }
}

fn try_open_item(
    app: &Arc<Mutex<App>>,
    client: &Arc<dyn AirflowClient>,
    item: OpenItem,
) -> Result<()> {
    // For Config items, look up the endpoint from active_server instead of using the passed string
    let final_item = if let OpenItem::Config(_) = &item {
        let app_lock = app.lock().unwrap();

        let active_server_name = app_lock
            .config
            .active_server
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No active server configured"))?;

        let server = app_lock
            .config
            .servers
            .iter()
            .find(|s| &s.name == active_server_name)
            .ok_or_else(|| {
                anyhow::anyhow!("Active server '{active_server_name}' not found in configuration")
            })?;

        OpenItem::Config(server.endpoint.clone())
    } else {
        item
    };

    let url = client.build_open_url(&final_item)?;
    webbrowser::open(&url).with_context(|| format!("failed to open browser for {url}"))?;
    Ok(())
}
