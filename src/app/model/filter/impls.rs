//! Filterable trait implementations for domain types.
//!
//! This module centralizes all `Filterable` implementations, keeping them
//! separate from the domain model definitions.

use crate::airflow::config::AirflowConfig;
use crate::airflow::model::common::dag::Dag;
use crate::airflow::model::common::dagrun::DagRun;
use crate::airflow::model::common::taskinstance::TaskInstance;
use crate::impl_filterable;

impl_filterable! {
    Dag,
    primary: dag_id => |s: &Dag| Some(s.dag_id.clone()),
    fields: [
        is_paused: enum["true", "false"] => |s: &Dag| Some(s.is_paused.to_string()),
        owners => |s: &Dag| Some(s.owners.join(", ")),
        tags => |s: &Dag| Some(s.tags.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", ")),
    ]
}

impl_filterable! {
    DagRun,
    primary: dag_run_id => |s: &DagRun| Some(s.dag_run_id.clone()),
    fields: [
        state: enum["running", "success", "failed", "queued", "up_for_retry"] => |s: &DagRun| Some(s.state.clone()),
        run_type: enum["scheduled", "manual", "backfill", "dataset_triggered"] => |s: &DagRun| Some(s.run_type.clone()),
    ]
}

impl_filterable! {
    TaskInstance,
    primary: task_id => |s: &TaskInstance| Some(s.task_id.clone()),
    fields: [
        state: enum[
            "running", "success", "failed", "queued",
            "up_for_retry", "up_for_reschedule", "skipped",
            "deferred", "removed", "restarting"
        ] => |s: &TaskInstance| s.state.clone(),
        operator => |s: &TaskInstance| s.operator.clone(),
    ]
}

impl_filterable! {
    AirflowConfig,
    primary: name => |s: &AirflowConfig| Some(s.name.clone()),
    fields: [
        endpoint => |s: &AirflowConfig| Some(s.endpoint.clone()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::filter::Filterable;

    #[test]
    fn test_dag_filterable() {
        assert_eq!(Dag::primary_field(), "dag_id");

        let fields = Dag::filterable_fields();
        assert_eq!(fields.len(), 4);
        assert!(fields[0].is_primary);
        assert_eq!(fields[0].name, "dag_id");
    }

    #[test]
    fn test_dagrun_filterable() {
        assert_eq!(DagRun::primary_field(), "dag_run_id");

        let fields = DagRun::filterable_fields();
        assert_eq!(fields.len(), 3);
        assert!(fields[0].is_primary);
    }

    #[test]
    fn test_taskinstance_filterable() {
        assert_eq!(TaskInstance::primary_field(), "task_id");

        let fields = TaskInstance::filterable_fields();
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn test_airflowconfig_filterable() {
        assert_eq!(AirflowConfig::primary_field(), "name");

        let fields = AirflowConfig::filterable_fields();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_dag_get_field_value() {
        let dag = Dag {
            dag_id: "test_dag".to_string(),
            is_paused: true,
            owners: vec!["alice".to_string(), "bob".to_string()],
            ..Default::default()
        };

        assert_eq!(dag.get_field_value("dag_id"), Some("test_dag".to_string()));
        assert_eq!(dag.get_field_value("is_paused"), Some("true".to_string()));
        assert_eq!(
            dag.get_field_value("owners"),
            Some("alice, bob".to_string())
        );
        assert_eq!(dag.get_field_value("unknown"), None);
    }
}
