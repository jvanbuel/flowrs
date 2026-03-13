use std::sync::LazyLock;

use time::format_description::BorrowedFormatItem;

use super::{App, Panel};

/// Cached date format for breadcrumb display (YYYY-MM-DD)
static BREADCRUMB_DATE_FORMAT: LazyLock<Vec<BorrowedFormatItem<'static>>> = LazyLock::new(|| {
    time::format_description::parse("[year]-[month]-[day]").expect("Invalid date format")
});

impl App {
    /// Generate breadcrumb string showing navigation context.
    pub fn breadcrumb(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        let env_name = self.nav_context.environment()?;
        parts.push(env_name.clone());

        if self.active_panel != Panel::Config && self.active_panel != Panel::Dag {
            if let Some(dag_id) = self.nav_context.dag_id() {
                let truncated = Self::truncate_breadcrumb_part(dag_id, 25);
                parts.push(truncated);
            }
        }

        if self.active_panel == Panel::TaskInstance || self.active_panel == Panel::Logs {
            if let Some(task_instance) = self.task_instances.table.filtered.items.first() {
                if let Some(logical_date) = task_instance.logical_date {
                    let formatted = logical_date
                        .format(&BREADCRUMB_DATE_FORMAT)
                        .unwrap_or_else(|_| "unknown".to_string());
                    parts.push(formatted);
                } else if let Some(dag_run_id) = self.nav_context.dag_run_id() {
                    let truncated = Self::truncate_breadcrumb_part(dag_run_id, 20);
                    parts.push(truncated);
                }
            } else if let Some(dag_run_id) = self.nav_context.dag_run_id() {
                let truncated = Self::truncate_breadcrumb_part(dag_run_id, 20);
                parts.push(truncated);
            }
        }

        if self.active_panel == Panel::Logs {
            if let Some(task_id) = self.nav_context.task_id() {
                let truncated = Self::truncate_breadcrumb_part(task_id, 20);
                parts.push(truncated);
            }
        }

        if parts.len() > 1 || self.active_panel != Panel::Config {
            Some(parts.join(" > "))
        } else if parts.len() == 1 {
            Some(parts[0].clone())
        } else {
            None
        }
    }

    fn truncate_breadcrumb_part(s: &str, max_chars: usize) -> String {
        let char_count = s.chars().count();
        if char_count <= max_chars {
            s.to_string()
        } else {
            let truncate_at = max_chars.saturating_sub(3);
            let byte_index = s
                .char_indices()
                .nth(truncate_at)
                .map_or(s.len(), |(idx, _)| idx);
            format!("{}...", &s[..byte_index])
        }
    }
}
