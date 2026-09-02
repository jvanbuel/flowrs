//! Filterable trait implementations for domain types.
//!
//! This module centralizes all `Filterable` implementations, keeping them
//! separate from the domain model definitions.
//!
//! Accessors borrow whenever the field is already a string (IDs, state names,
//! operator) and only build a `String` for derived values such as joined
//! lists, so a filter pass over the table stays allocation-free for the
//! common fields.

use std::borrow::Cow;

use crate::airflow::model::common::dag::Dag;
use crate::airflow::model::common::dagrun::DagRun;
use crate::airflow::model::common::taskinstance::TaskInstance;
use crate::impl_filterable;
use flowrs_config::AirflowConfig;

/// Borrow an existing string field.
#[expect(
    clippy::unnecessary_wraps,
    reason = "accessors must evaluate to Option so absent fields can return None"
)]
fn borrowed(value: &str) -> Option<Cow<'_, str>> {
    Some(Cow::Borrowed(value))
}

/// Hand over a value that had to be built for this lookup.
#[expect(
    clippy::unnecessary_wraps,
    reason = "accessors must evaluate to Option so absent fields can return None"
)]
fn owned(value: String) -> Option<Cow<'static, str>> {
    Some(Cow::Owned(value))
}

const fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

impl_filterable! {
    Dag as dag,
    primary: dag_id => borrowed(&dag.dag_id),
    fields: [
        is_paused: enum["true", "false"] => borrowed(bool_str(dag.is_paused)),
        owners => owned(dag.owners.join(", ")),
        tags => owned(dag.tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ")),
    ]
}

impl_filterable! {
    DagRun as run,
    primary: dag_run_id => borrowed(&run.dag_run_id),
    fields: [
        state: enum["running", "success", "failed", "queued", "up_for_retry"] => borrowed(run.state.as_str()),
        run_type: enum["scheduled", "manual", "backfill", "dataset_triggered", "asset_triggered"] => borrowed(run.run_type.as_str()),
    ]
}

impl_filterable! {
    TaskInstance as ti,
    primary: task_id => borrowed(&ti.task_id),
    fields: [
        state: enum[
            "running", "success", "failed", "queued",
            "up_for_retry", "up_for_reschedule", "skipped",
            "deferred", "removed", "restarting"
        ] => ti.state.as_ref().map(|s| Cow::Borrowed(s.as_str())),
        operator => ti.operator.as_deref().map(Cow::Borrowed),
    ]
}

impl_filterable! {
    AirflowConfig as config,
    primary: name => borrowed(&config.name),
    fields: [
        endpoint => borrowed(&config.endpoint),
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
            dag_id: "test_dag".into(),
            is_paused: true,
            owners: vec!["alice".to_string(), "bob".to_string()],
            ..Default::default()
        };

        assert_eq!(dag.get_field_value("dag_id").as_deref(), Some("test_dag"));
        assert_eq!(dag.get_field_value("is_paused").as_deref(), Some("true"));
        assert_eq!(dag.get_field_value("owners").as_deref(), Some("alice, bob"));
        assert_eq!(dag.get_field_value("unknown"), None);
    }

    #[test]
    fn string_fields_are_borrowed_not_copied() {
        let dag = Dag {
            dag_id: "test_dag".into(),
            ..Default::default()
        };
        assert!(matches!(
            dag.get_field_value("dag_id"),
            Some(Cow::Borrowed(_))
        ));
        assert!(matches!(
            dag.get_field_value("is_paused"),
            Some(Cow::Borrowed(_))
        ));

        let ti = TaskInstance {
            state: Some(crate::airflow::model::common::TaskInstanceState::Running),
            operator: Some("BashOperator".to_string()),
            ..Default::default()
        };
        assert_eq!(ti.get_field_value("state").as_deref(), Some("running"));
        assert!(matches!(
            ti.get_field_value("state"),
            Some(Cow::Borrowed(_))
        ));
        assert!(matches!(
            ti.get_field_value("operator"),
            Some(Cow::Borrowed(_))
        ));
    }
}
