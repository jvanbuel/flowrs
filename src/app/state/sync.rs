use std::sync::Arc;

use super::App;
use crate::app::navigation::Level;

/// Collect autocomplete candidates: sorted, de-duplicated, blanks dropped.
fn distinct(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut values: Vec<String> = values.filter(|v| !v.is_empty()).collect();
    values.sort_unstable();
    values.dedup();
    values
}

impl App {
    /// Sync a specific panel's data from `environment_state`.
    pub fn sync_panel(&mut self, level: Level) {
        let path = self.navigation.path();
        match level {
            Level::Dags => {
                self.dags.table.set_items(self.environment_state.dags());
                self.dags.dag_stats = self.environment_state.dag_stats();
                let dag_ids: Vec<String> = self
                    .dags
                    .table
                    .all()
                    .iter()
                    .map(|d| d.dag_id.to_string())
                    .collect();
                self.dags
                    .table
                    .filter_mut()
                    .set_primary_values("dag_id", dag_ids);
                // Free-text fields autocomplete from the values actually present.
                let owners = distinct(
                    self.dags
                        .table
                        .all()
                        .iter()
                        .flat_map(|d| d.owners.iter().cloned()),
                );
                self.dags
                    .table
                    .filter_mut()
                    .set_field_values("owners", owners);
                let tags = distinct(
                    self.dags
                        .table
                        .all()
                        .iter()
                        .flat_map(|d| d.tags.iter().map(|t| t.name.clone())),
                );
                self.dags.table.filter_mut().set_field_values("tags", tags);
            }
            Level::DagRuns => {
                if let Some(dag_id) = path.dag_id() {
                    self.dagruns
                        .table
                        .set_items(self.environment_state.dag_runs(dag_id));
                    let dag_run_ids: Vec<String> = self
                        .dagruns
                        .table
                        .all()
                        .iter()
                        .map(|dr| dr.dag_run_id.to_string())
                        .collect();
                    self.dagruns
                        .table
                        .filter_mut()
                        .set_primary_values("dag_run_id", dag_run_ids);
                } else {
                    self.dagruns.table.clear();
                }
            }
            Level::TaskInstances => {
                if let (Some(dag_id), Some(dag_run_id)) = (path.dag_id(), path.dag_run_id()) {
                    self.task_instances
                        .table
                        .set_items(self.environment_state.task_instances(dag_id, dag_run_id));
                    self.task_instances.task_graph = self.environment_state.task_graph(dag_id);
                    let task_ids: Vec<String> = self
                        .task_instances
                        .table
                        .all()
                        .iter()
                        .map(|ti| ti.task_id.to_string())
                        .collect();
                    self.task_instances
                        .table
                        .filter_mut()
                        .set_primary_values("task_id", task_ids);
                    let operators = distinct(
                        self.task_instances
                            .table
                            .all()
                            .iter()
                            .filter_map(|ti| ti.operator.clone()),
                    );
                    self.task_instances
                        .table
                        .filter_mut()
                        .set_field_values("operator", operators);
                } else {
                    self.task_instances.table.clear();
                }
            }
            Level::Logs => {
                if let (Some(dag_id), Some(dag_run_id), Some(task_id)) =
                    (path.dag_id(), path.dag_run_id(), path.task_id())
                {
                    self.logs.update_logs(
                        self.environment_state
                            .task_logs(dag_id, dag_run_id, task_id),
                    );
                } else {
                    // Go through update_logs so the selected try is clamped and
                    // the search matches are refreshed, like any other update.
                    self.logs.update_logs(Arc::default());
                }
            }
            Level::Environments => {
                let config_names: Vec<String> = self
                    .configs
                    .table
                    .all()
                    .iter()
                    .map(|c| c.name.clone())
                    .collect();
                self.configs
                    .table
                    .filter_mut()
                    .set_primary_values("name", config_names);
                let endpoints =
                    distinct(self.configs.table.all().iter().map(|c| c.endpoint.clone()));
                self.configs
                    .table
                    .filter_mut()
                    .set_field_values("endpoint", endpoints);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::distinct;

    #[test]
    fn distinct_sorts_dedups_and_drops_blanks() {
        let got = distinct(
            ["bob", "alice", "bob", "", "alice"]
                .into_iter()
                .map(str::to_string),
        );
        assert_eq!(got, vec!["alice", "bob"]);
    }
}
