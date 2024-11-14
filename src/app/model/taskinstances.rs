use std::vec;

use crossterm::event::KeyCode;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, StatefulWidget, Table, Widget};
use time::format_description;

use crate::airflow::model::taskinstance::TaskInstance;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::popup::taskinstances::clear::ClearTaskInstancePopup;
use super::popup::taskinstances::mark::MarkTaskInstancePopup;
use super::popup::taskinstances::TaskInstancePopUp;
use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::worker::WorkerMessage;

pub struct TaskInstanceModel {
    pub dag_id: Option<String>,
    pub dag_run_id: Option<String>,
    pub all: Vec<TaskInstance>,
    pub filtered: StatefulTable<TaskInstance>,
    pub filter: Filter,
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
    pub popup: Option<TaskInstancePopUp>,
    pub marked: Vec<usize>,
    ticks: u32,
}

impl TaskInstanceModel {
    pub fn new() -> Self {
        TaskInstanceModel {
            dag_id: None,
            dag_run_id: None,
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            errors: vec![],
            popup: None,
            marked: vec![],
            ticks: 0,
        }
    }

    pub fn filter_task_instances(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_task_instances = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|task_instance| task_instance.task_id.contains(prefix))
                .cloned()
                .collect::<Vec<TaskInstance>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_task_instances.to_vec();
    }

    #[allow(dead_code)]
    pub fn current(&mut self) -> Option<&mut TaskInstance> {
        self.filtered
            .state
            .selected()
            .map(|i| &mut self.filtered.items[i])
    }
    pub fn mark_task_instance(&mut self, task_id: &str, status: &str) {
        self.filtered.items.iter_mut().for_each(|task_instance| {
            if task_instance.task_id == task_id {
                task_instance.state = status.to_string();
            }
        });
    }
}

impl Default for TaskInstanceModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for TaskInstanceModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                debug!("Updating task instances");
                debug!("Dag ID: {:?}", self.dag_id);
                debug!("Dag Run ID: {:?}", self.dag_run_id);
                if let (Some(dag_run_id), Some(dag_id)) = (&self.dag_run_id, &self.dag_id) {
                    log::debug!("Updating task instances for dag_run_id: {}", dag_run_id);
                    return (
                        Some(FlowrsEvent::Tick),
                        vec![WorkerMessage::UpdateTaskInstances {
                            dag_id: dag_id.clone(),
                            dag_run_id: dag_run_id.clone(),
                            clear: false,
                        }],
                    );
                }
                (Some(FlowrsEvent::Tick), vec![])
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_task_instances();
                } else if let Some(popup) = &mut self.popup {
                    match popup {
                        TaskInstancePopUp::Clear(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {:?}", messages);
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                        TaskInstancePopUp::Mark(popup) => {
                            let (key_event, messages) = popup.update(event);
                            debug!("Popup messages: {:?}", messages);
                            if let Some(FlowrsEvent::Key(key_event)) = &key_event {
                                match key_event.code {
                                    KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                                        self.popup = None;
                                        self.marked = vec![];
                                    }
                                    _ => {}
                                }
                            }
                            return (None, messages);
                        }
                    }
                } else {
                    match key_event.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.filtered.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.filtered.previous();
                        }
                        KeyCode::Char('G') => {
                            self.filtered.state.select_last();
                        }
                        KeyCode::Char('m') => {
                            if let Some(index) = self.filtered.state.selected() {
                                self.marked.push(index);

                                let dag_id = self.current().unwrap().dag_id.clone();
                                let dag_run_id = self.current().unwrap().dag_run_id.clone();

                                self.popup =
                                    Some(TaskInstancePopUp::Mark(MarkTaskInstancePopup::new(
                                        self.marked
                                            .iter()
                                            .map(|i| self.filtered.items[*i].task_id.clone())
                                            .collect(),
                                        &dag_id,
                                        &dag_run_id,
                                    )));
                            }
                        }
                        KeyCode::Char('M') => {
                            if let Some(index) = self.filtered.state.selected() {
                                if self.marked.contains(&index) {
                                    self.marked.retain(|&i| i != index);
                                } else {
                                    self.marked.push(index);
                                }
                            }
                        }
                        KeyCode::Char('c') => {
                            if let Some(task_instance) = self.current() {
                                self.popup =
                                    Some(TaskInstancePopUp::Clear(ClearTaskInstancePopup::new(
                                        &task_instance.dag_run_id,
                                        &task_instance.dag_id,
                                        &task_instance.task_id,
                                    )));
                            }
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_task_instances();
                        }
                        KeyCode::Enter => {
                            if let Some(task_instance) = self.current() {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::GetTaskLogs {
                                        dag_id: task_instance.dag_id.clone(),
                                        dag_run_id: task_instance.dag_run_id.clone(),
                                        task_id: task_instance.task_id.clone(),
                                        task_try: task_instance.try_number as u16,
                                    }],
                                );
                            }
                        }
                        _ => return (Some(FlowrsEvent::Key(*key_event)), vec![]), // if no match, return the event
                    }
                }
                (None, vec![])
            }
            _ => (Some(event.clone()), vec![]),
        }
    }
}
impl Widget for &mut TaskInstanceModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            let filter = self.filter.prefix().clone();

            let paragraph = Paragraph::new(filter.unwrap_or("".to_string()))
                .block(Block::default().borders(Borders::ALL).title("filter"))
                .set_style(DEFAULT_STYLE);

            Widget::render(paragraph, rects[1], buffer);

            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };

        let selected_style = Style::default().add_modifier(Modifier::REVERSED);

        let headers = ["Task ID", "Execution Date", "Duration", "State", "Tries"];
        let header_cells = headers.iter().map(|h| Cell::from(*h));
        let header =
            Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(item.task_id.as_str()),
                Line::from(if let Some(date) = item.execution_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .to_string()
                } else {
                    "None".to_string()
                }),
                Line::from(if let Some(i) = item.duration {
                    format!("{i}")
                } else {
                    "None".to_string()
                }),
                Line::from(match item.state.as_str() {
                    "success" => state_to_color(item, Color::Green),
                    "running" => state_to_color(item, Color::LightGreen),
                    "failed" => state_to_color(item, Color::Red),
                    "queued" => state_to_color(item, Color::LightBlue),
                    "up_for_retry" => state_to_color(item, Color::LightYellow),
                    "upstream_failed" => state_to_color(item, Color::Rgb(255, 165, 0)),
                    _ => state_to_color(item, Color::White),
                }),
                Line::from(format!("{:?}", item.try_number)),
            ])
            .style(if self.marked.contains(&idx) {
                DEFAULT_STYLE.bg(Color::Rgb(255, 255, 224))
            } else if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(Color::Rgb(33, 34, 35))
            })
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Length(20),
                Constraint::Length(15),
                Constraint::Length(5),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("TaskInstances"),
        )
        .style(DEFAULT_STYLE)
        .row_highlight_style(selected_style);

        StatefulWidget::render(t, rects[0], buffer, &mut self.filtered.state);

        match &mut self.popup {
            Some(TaskInstancePopUp::Clear(popup)) => {
                popup.render(area, buffer);
            }
            Some(TaskInstancePopUp::Mark(popup)) => {
                popup.render(area, buffer);
            }
            _ => (),
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
#[allow(dead_code)]
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn state_to_color(item: &TaskInstance, color: Color) -> Span {
    Span::styled(item.state.as_str(), Style::default().fg(color))
}
