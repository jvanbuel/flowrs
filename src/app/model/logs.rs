use crate::{
    airflow::model::log::Log,
    app::{error::FlowrsError, events::custom::FlowrsEvent, worker::WorkerMessage},
};

use super::{filter::Filter, Model, StatefulTable};

pub struct LogModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub task_id: Option<String>,
    pub all: Vec<Log>,
    pub filtered: StatefulTable<Log>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
    ticks: u32,
}

impl Model for LogModel {
    fn update(
        &mut self,
        event: &crate::app::events::custom::FlowrsEvent,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        unimplemented!()
    }
    fn view(&mut self, f: &mut ratatui::Frame) {
        unimplemented!()
    }
}
