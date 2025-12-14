use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::airflow::traits::AirflowClient;
use crate::app::state::App;
use crate::app::worker::OpenItem;

/// Handle opening an item (DAG, DAG run, task instance, etc.) in the browser.
pub fn handle_open_item(
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

        let servers = app_lock
            .config
            .servers
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No servers configured"))?;

        let server = servers
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
    if let Err(e) = webbrowser::open(&url) {
        log::error!("Failed to open browser with URL {url}: {e}");
    }
    Ok(())
}
