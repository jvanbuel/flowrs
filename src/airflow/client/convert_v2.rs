use crate::airflow::model::common::dagrun::{DagRunState, RunType};
use crate::airflow::model::common::dagstats::{DagStatistic, DagStatistics};
use crate::airflow::model::common::taskinstance::TaskInstanceState;
use crate::airflow::model::common::{
    Dag, DagList, DagRun, DagRunList, DagStatsResponse, Log, Tag, Task, TaskInstance,
    TaskInstanceList, TaskList, TaskTryGantt,
};

pub(crate) fn v2_dag_to_dag(value: flowrs_airflow::client::v2::model::dag::Dag) -> Dag {
    Dag {
        dag_id: value.dag_id.into(),
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
        tags: value
            .tags
            .into_iter()
            .map(|t| Tag { name: t.name })
            .collect(),
        file_token: value.file_token,
        timetable_description: value.timetable_description,
    }
}

pub(crate) fn v2_dag_list_to_dag_list(
    value: flowrs_airflow::client::v2::model::dag::DagList,
) -> DagList {
    DagList {
        dags: value.dags.into_iter().map(v2_dag_to_dag).collect(),
        total_entries: value.total_entries,
    }
}

pub(crate) fn v2_dagrun_to_dagrun(
    value: flowrs_airflow::client::v2::model::dagrun::DagRun,
) -> DagRun {
    DagRun {
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.into(),
        logical_date: value.logical_date,
        data_interval_end: value.data_interval_end,
        data_interval_start: value.data_interval_start,
        end_date: value.end_date,
        start_date: value.start_date,
        last_scheduling_decision: value.last_scheduling_decision,
        run_type: RunType::from(value.run_type.as_str()),
        state: DagRunState::from(value.state.as_str()),
        note: value.note,
        external_trigger: None,
    }
}

pub(crate) fn v2_dagrun_list_to_list(
    value: flowrs_airflow::client::v2::model::dagrun::DagRunList,
) -> DagRunList {
    DagRunList {
        dag_runs: value
            .dag_runs
            .into_iter()
            .map(v2_dagrun_to_dagrun)
            .collect(),
        total_entries: value.total_entries,
    }
}

pub(crate) fn v2_task_instance_to_domain(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstance,
) -> TaskInstance {
    TaskInstance {
        task_id: value.task_id.into(),
        dag_id: value.dag_id.into(),
        dag_run_id: value.dag_run_id.into(),
        logical_date: value.logical_date,
        start_date: value.start_date,
        end_date: value.end_date,
        duration: value.duration,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
        try_number: value.try_number,
        max_tries: value.max_tries,
        map_index: value.map_index,
        hostname: value.hostname,
        unixname: value.unixname,
        pool: value.pool,
        pool_slots: value.pool_slots,
        queue: value.queue,
        priority_weight: value.priority_weight,
        operator: value.operator,
        queued_when: value.queued_when,
        pid: value.pid,
        note: value.note,
    }
}

pub(crate) fn v2_task_instance_list_to_list(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstanceList,
) -> TaskInstanceList {
    TaskInstanceList {
        task_instances: value
            .task_instances
            .into_iter()
            .map(v2_task_instance_to_domain)
            .collect(),
        total_entries: value.total_entries,
    }
}

pub(crate) fn v2_task_instance_try_to_gantt(
    value: flowrs_airflow::client::v2::model::taskinstance::TaskInstanceTryResponse,
) -> TaskTryGantt {
    TaskTryGantt {
        try_number: value.try_number,
        start_date: value.start_date,
        end_date: value.end_date,
        state: value.state.map(|s| TaskInstanceState::from(s.as_str())),
    }
}

pub(crate) fn v2_task_to_task(
    value: flowrs_airflow::client::v2::model::task::TaskResponse,
) -> Task {
    Task {
        task_id: value.task_id,
        downstream_task_ids: value.downstream_task_ids,
    }
}

pub(crate) fn v2_task_collection_to_list(
    value: flowrs_airflow::client::v2::model::task::TaskCollectionResponse,
) -> TaskList {
    TaskList {
        tasks: value.tasks.into_iter().map(v2_task_to_task).collect(),
    }
}

pub(crate) fn v2_log_to_log(value: flowrs_airflow::client::v2::model::log::Log) -> Log {
    Log {
        continuation_token: value.continuation_token,
        content: value.content.to_string(),
    }
}

pub(crate) fn v2_dagstats_to_response(
    value: flowrs_airflow::client::v2::model::dagstats::DagStatsResponse,
) -> DagStatsResponse {
    DagStatsResponse {
        dags: value
            .dags
            .into_iter()
            .map(|ds| DagStatistics {
                dag_id: ds.dag_id,
                stats: ds
                    .stats
                    .into_iter()
                    .map(|s| DagStatistic {
                        state: DagRunState::from(s.state.as_str()),
                        count: s.count,
                    })
                    .collect(),
            })
            .collect(),
        total_entries: value.total_entries,
    }
}
