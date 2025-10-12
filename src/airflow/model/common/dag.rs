use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Common DAG model used by the application
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dag {
    pub dag_id: String,
    pub dag_display_name: Option<String>,
    pub description: Option<String>,
    pub fileloc: String,
    pub is_paused: bool,
    pub is_active: Option<bool>,
    pub has_import_errors: bool,
    pub has_task_concurrency_limits: bool,
    pub last_parsed_time: Option<OffsetDateTime>,
    pub last_expired: Option<OffsetDateTime>,
    pub max_active_tasks: i64,
    pub max_active_runs: Option<i64>,
    pub next_dagrun_logical_date: Option<OffsetDateTime>,
    pub next_dagrun_data_interval_start: Option<OffsetDateTime>,
    pub next_dagrun_data_interval_end: Option<OffsetDateTime>,
    pub next_dagrun_create_after: Option<OffsetDateTime>,
    pub owners: Vec<String>,
    pub tags: Vec<Tag>,
    pub file_token: String,
    pub timetable_description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DagList {
    pub dags: Vec<Dag>,
    pub total_entries: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

// From trait implementations for v1 models
impl From<crate::airflow::model::v1::dag::DagResponse> for Dag {
    /// Convert a v1 `DagResponse` into the common `Dag` model.
    ///
    /// Maps fields from the v1 API response into the shared `Dag` representation, supplying sensible defaults for optional booleans and numeric limits and converting nested tags.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::airflow::model::common::dag::Dag;
    /// # use crate::airflow::model::v1::dag::DagResponse;
    /// // given a `DagResponse` named `response`:
    /// let dag: Dag = response.into();
    /// ```
    fn from(value: crate::airflow::model::v1::dag::DagResponse) -> Self {
        Dag {
            dag_id: value.dag_id,
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused.unwrap_or(false),
            is_active: value.is_active,
            has_import_errors: value.has_import_errors.unwrap_or(false),
            has_task_concurrency_limits: value.has_task_concurrency_limits.unwrap_or(false),
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks.unwrap_or(0),
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            next_dagrun_create_after: value.next_dagrun_create_after,
            owners: value.owners.clone(),
            tags: value
                .tags
                .unwrap_or_default()
                .into_iter()
                .map(|t| t.into())
                .collect(),
            file_token: value.file_token.clone(),
            timetable_description: value.timetable_description.clone(),
        }
    }
}

impl From<crate::airflow::model::v1::dag::DagCollectionResponse> for DagList {
    /// Converts a v1 `DagCollectionResponse` into the common `DagList`.
    ///
    /// # Examples
    ///
    /// ```
    /// let v1 = crate::airflow::model::v1::dag::DagCollectionResponse {
    ///     dags: vec![],
    ///     total_entries: 0,
    /// };
    /// let list: crate::airflow::model::common::dag::DagList = v1.into();
    /// assert_eq!(list.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v1::dag::DagCollectionResponse) -> Self {
        DagList {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v1::dag::DagTagResponse> for Tag {
    /// Create a common `Tag` from a v1 `DagTagResponse` by copying the `name` field.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v1::dag::DagTagResponse;
    /// use crate::airflow::model::common::dag::Tag;
    ///
    /// let src = DagTagResponse { name: "etl".to_string() };
    /// let tag: Tag = src.into();
    /// assert_eq!(tag.name, "etl");
    /// ```
    fn from(value: crate::airflow::model::v1::dag::DagTagResponse) -> Self {
        Tag { name: value.name }
    }
}

// From trait implementations for v2 models
impl From<crate::airflow::model::v2::dag::Dag> for Dag {
    /// Converts a v2 `Dag` into the common `Dag` model.
    ///
    /// Copies identifiers, display name, description, file location, status flags, timing fields, concurrency limits, owners, tags, file token, and timetable description from the source `crate::airflow::model::v2::dag::Dag`. The resulting `Dag` sets `is_active` to `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::airflow::model::v2::dag as v2;
    /// // Construct a minimal v2 DAG (fields omitted for brevity; real code should populate required fields)
    /// let v2_dag: v2::Dag = Default::default();
    /// let common_dag: crate::airflow::model::common::dag::Dag = v2_dag.into();
    /// // `is_active` is unset in the common model
    /// assert_eq!(common_dag.is_active, None);
    /// ```
    fn from(value: crate::airflow::model::v2::dag::Dag) -> Self {
        Dag {
            dag_id: value.dag_id,
            dag_display_name: Some(value.dag_display_name),
            description: value.description,
            fileloc: value.fileloc,
            is_paused: value.is_paused,
            is_active: None,
            has_import_errors: value.has_import_errors,
            has_task_concurrency_limits: value.has_task_concurrency_limits,
            last_parsed_time: value.last_parsed_time,
            last_expired: value.last_expired,
            max_active_tasks: value.max_active_tasks,
            max_active_runs: value.max_active_runs,
            next_dagrun_logical_date: value.next_dagrun_logical_date,
            next_dagrun_create_after: value.next_dagrun_run_after,
            next_dagrun_data_interval_start: value.next_dagrun_data_interval_start,
            next_dagrun_data_interval_end: value.next_dagrun_data_interval_end,
            owners: value.owners,
            tags: value.tags.into_iter().map(|t| t.into()).collect(),
            file_token: value.file_token,
            timetable_description: value.timetable_description,
        }
    }
}

impl From<crate::airflow::model::v2::dag::DagList> for DagList {
    /// Convert a v2 `DagList` into the common `DagList`, converting each contained DAG and preserving the entry count.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2 = crate::airflow::model::v2::dag::DagList { dags: vec![], total_entries: 0 };
    /// let list: crate::airflow::model::common::dag::DagList = v2.into();
    /// assert_eq!(list.total_entries, 0);
    /// ```
    fn from(value: crate::airflow::model::v2::dag::DagList) -> Self {
        DagList {
            dags: value.dags.into_iter().map(|d| d.into()).collect(),
            total_entries: value.total_entries,
        }
    }
}

impl From<crate::airflow::model::v2::dag::Tag> for Tag {
    /// Converts a v2 `Tag` into the common `Tag` model.
    ///
    /// # Examples
    ///
    /// ```
    /// let v2_tag = crate::airflow::model::v2::dag::Tag { name: "analytics".into() };
    /// let common_tag: crate::airflow::model::common::dag::Tag = v2_tag.into();
    /// assert_eq!(common_tag.name, "analytics");
    /// ```
    fn from(value: crate::airflow::model::v2::dag::Tag) -> Self {
        Tag { name: value.name }
    }
}