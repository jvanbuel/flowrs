use std::sync::LazyLock;

use time::format_description::BorrowedFormatItem;

use super::App;
use crate::app::navigation::Level;
use crate::ui::common::truncate_cols;

/// Cached date format for breadcrumb display (YYYY-MM-DD)
static BREADCRUMB_DATE_FORMAT: LazyLock<Vec<BorrowedFormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        .expect("Invalid date format")
});

impl App {
    /// Generate breadcrumb string showing navigation context.
    pub fn breadcrumb(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        let path = self.navigation.path();
        let level = self.navigation.level();
        let env_name = path.environment()?;
        parts.push(env_name.clone());

        if level >= Level::DagRuns {
            if let Some(dag_id) = path.dag_id() {
                let truncated = truncate_cols(dag_id, 25);
                parts.push(truncated);
            }
        }

        if level >= Level::TaskInstances {
            if let Some(task_instance) = self.task_instances.table.items().next() {
                if let Some(logical_date) = task_instance.logical_date {
                    let formatted = logical_date
                        .format(&BREADCRUMB_DATE_FORMAT)
                        .unwrap_or_else(|_| "unknown".to_string());
                    parts.push(formatted);
                } else if let Some(dag_run_id) = path.dag_run_id() {
                    let truncated = truncate_cols(dag_run_id, 20);
                    parts.push(truncated);
                }
            } else if let Some(dag_run_id) = path.dag_run_id() {
                let truncated = truncate_cols(dag_run_id, 20);
                parts.push(truncated);
            }
        }

        if level == Level::Logs {
            if let Some(task_id) = path.task_id() {
                let truncated = truncate_cols(task_id, 20);
                parts.push(truncated);
            }
        }

        if parts.len() > 1 || level != Level::Environments {
            Some(parts.join(" > "))
        } else if parts.len() == 1 {
            Some(parts[0].clone())
        } else {
            None
        }
    }
}
