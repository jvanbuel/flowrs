use super::{App, Panel};

impl App {
    /// Sync a specific panel's data from `environment_state`.
    pub fn sync_panel(&mut self, panel: &Panel) {
        match panel {
            Panel::Dag => {
                self.dags.table.all = self.environment_state.get_active_dags();
                self.dags.dag_stats = self.environment_state.get_active_dag_stats();
                let dag_ids: Vec<String> = self
                    .dags
                    .table
                    .all
                    .iter()
                    .map(|d| d.dag_id.to_string())
                    .collect();
                self.dags.table.filter.set_primary_values("dag_id", dag_ids);
                self.dags.table.apply_filter();
            }
            Panel::DAGRun => {
                if let Some(dag_id) = self.nav_context.dag_id() {
                    self.dagruns.table.all = self.environment_state.get_active_dag_runs(dag_id);
                    let dag_run_ids: Vec<String> = self
                        .dagruns
                        .table
                        .all
                        .iter()
                        .map(|dr| dr.dag_run_id.to_string())
                        .collect();
                    self.dagruns
                        .table
                        .filter
                        .set_primary_values("dag_run_id", dag_run_ids);
                    self.dagruns.table.apply_filter();
                    self.dagruns.sort_dag_runs();
                } else {
                    self.dagruns.table.all.clear();
                }
            }
            Panel::TaskInstance => {
                if let (Some(dag_id), Some(dag_run_id)) =
                    (self.nav_context.dag_id(), self.nav_context.dag_run_id())
                {
                    self.task_instances.set_gantt_context(dag_id, dag_run_id);
                    self.task_instances.table.all = self
                        .environment_state
                        .get_active_task_instances(dag_id, dag_run_id);
                    self.task_instances.sort_task_instances();
                    let task_ids: Vec<String> = self
                        .task_instances
                        .table
                        .all
                        .iter()
                        .map(|ti| ti.task_id.to_string())
                        .collect();
                    self.task_instances
                        .table
                        .filter
                        .set_primary_values("task_id", task_ids);
                    self.task_instances.table.apply_filter();
                } else {
                    self.task_instances.table.all.clear();
                }
            }
            Panel::Logs => {
                if let (Some(dag_id), Some(dag_run_id), Some(task_id)) = (
                    self.nav_context.dag_id(),
                    self.nav_context.dag_run_id(),
                    self.nav_context.task_id(),
                ) {
                    self.logs.update_logs(
                        self.environment_state
                            .get_active_task_logs(dag_id, dag_run_id, task_id),
                    );
                } else {
                    self.logs.all.clear();
                }
            }
            Panel::Config => {
                let config_names: Vec<String> = self
                    .configs
                    .table
                    .all
                    .iter()
                    .map(|c| c.name.clone())
                    .collect();
                self.configs
                    .table
                    .filter
                    .set_primary_values("name", config_names);
            }
        }
    }
}
