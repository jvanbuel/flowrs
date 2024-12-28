use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget, Table, Widget, Wrap,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_tui::into_span;
use time::format_description;

use crate::airflow::model::dagrun::DagRun;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::common::create_headers;
use crate::ui::constants::{AirflowStateColor, ALTERNATING_ROW_COLOR, DEFAULT_STYLE, MARKED_COLOR};
use crate::ui::TIME_FORMAT;

use super::popup::commands_help::CommandPopUp;
use super::popup::dagruns::commands::DAGRUN_COMMAND_POP_UP;
use super::popup::dagruns::trigger::TriggerDagRunPopUp;
use super::popup::dagruns::DagRunPopUp;
use super::popup::popup_area;
use super::popup::{dagruns::clear::ClearDagRunPopup, dagruns::mark::MarkDagRunPopup};
use super::{filter::Filter, Model, StatefulTable};
use crate::app::worker::{OpenItem, WorkerMessage};
use anyhow::Error;

pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: DagCodeWidget,
    pub all: Vec<DagRun>,
    pub filtered: StatefulTable<DagRun>,
    pub filter: Filter,
    pub marked: Vec<usize>,
    #[allow(dead_code)]
    pub errors: Vec<Error>,
    pub popup: Option<DagRunPopUp>,
    pub commands: Option<&'static CommandPopUp<'static>>,
    ticks: u32,
    event_buffer: Vec<FlowrsEvent>,
}

#[derive(Default)]
pub struct DagCodeWidget {
    pub code: Option<String>,
    pub vertical_scroll: usize,
    pub vertical_scroll_state: ScrollbarState,
}

impl DagRunModel {
    pub fn new() -> Self {
        DagRunModel {
            dag_id: None,
            dag_code: DagCodeWidget::default(),
            all: vec![],
            filtered: StatefulTable::new(vec![]),
            filter: Filter::new(),
            marked: vec![],
            errors: vec![],
            popup: None,
            commands: None,
            ticks: 0,
            event_buffer: vec![],
        }
    }

    pub fn filter_dag_runs(&mut self) {
        let prefix = &self.filter.prefix;
        let filtered_dag_runs = match prefix {
            Some(prefix) => &self
                .all
                .iter()
                .filter(|dagrun| dagrun.dag_run_id.contains(prefix))
                .cloned()
                .collect::<Vec<DagRun>>(),
            None => &self.all,
        };
        self.filtered.items = filtered_dag_runs.to_vec();
    }

    pub fn current(&self) -> Option<&DagRun> {
        self.filtered
            .state
            .selected()
            .map(|i| &self.filtered.items[i])
    }

    pub fn mark_dag_run(&mut self, dag_run_id: &str, status: &str) {
        self.filtered.items.iter_mut().for_each(|dag_run| {
            if dag_run.dag_run_id == dag_run_id {
                dag_run.state = status.to_string();
            }
        });
    }
}

impl Default for DagRunModel {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for DagRunModel {
    fn update(&mut self, event: &FlowrsEvent) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        match event {
            FlowrsEvent::Tick => {
                self.ticks += 1;
                if self.ticks % 10 != 0 {
                    return (Some(FlowrsEvent::Tick), vec![]);
                }
                let worker_messages = if let Some(dag_id) = &self.dag_id {
                    vec![WorkerMessage::UpdateDagRuns {
                        dag_id: dag_id.clone(),
                        clear: false,
                    }]
                } else {
                    Vec::default()
                };
                return (Some(FlowrsEvent::Tick), worker_messages);
            }
            FlowrsEvent::Key(key_event) => {
                if self.filter.is_enabled() {
                    self.filter.update(key_event);
                    self.filter_dag_runs();
                    return (None, vec![]);
                } else if let Some(_commands) = &mut self.commands {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                            self.commands = None;
                            return (None, vec![]);
                        }
                        _ => (),
                    }
                } else if let Some(popup) = &mut self.popup {
                    // TODO: refactor this, should be all the same
                    match popup {
                        DagRunPopUp::Clear(popup) => {
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
                        DagRunPopUp::Mark(popup) => {
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
                        DagRunPopUp::Trigger(popup) => {
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
                    }
                } else if self.dag_code.code.is_some() {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') | KeyCode::Enter => {
                            self.dag_code.code = None;
                            return (None, vec![]);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            self.dag_code.vertical_scroll =
                                self.dag_code.vertical_scroll.saturating_add(1);
                            self.dag_code.vertical_scroll_state = self
                                .dag_code
                                .vertical_scroll_state
                                .position(self.dag_code.vertical_scroll)
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            self.dag_code.vertical_scroll =
                                self.dag_code.vertical_scroll.saturating_sub(1);
                            self.dag_code.vertical_scroll_state = self
                                .dag_code
                                .vertical_scroll_state
                                .position(self.dag_code.vertical_scroll)
                        }
                        _ => {}
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
                        KeyCode::Char('g') => {
                            if let Some(FlowrsEvent::Key(key_event)) = self.event_buffer.pop() {
                                if key_event.code == KeyCode::Char('g') {
                                    self.filtered.state.select_first();
                                } else {
                                    self.event_buffer.push(FlowrsEvent::Key(key_event));
                                }
                            } else {
                                self.event_buffer.push(FlowrsEvent::Key(*key_event));
                            }
                        }
                        KeyCode::Char('t') => {
                            self.popup = Some(DagRunPopUp::Trigger(TriggerDagRunPopUp::new(
                                self.dag_id.clone().unwrap(),
                            )));
                        }
                        KeyCode::Char('m') => {
                            if let Some(index) = self.filtered.state.selected() {
                                self.marked.push(index);

                                self.popup = Some(DagRunPopUp::Mark(MarkDagRunPopup::new(
                                    self.marked
                                        .iter()
                                        .map(|i| self.filtered.items[*i].dag_run_id.clone())
                                        .collect(),
                                    self.current().unwrap().dag_id.clone(),
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
                        KeyCode::Char('?') => {
                            self.commands = Some(&*DAGRUN_COMMAND_POP_UP);
                        }
                        KeyCode::Char('/') => {
                            self.filter.toggle();
                            self.filter_dag_runs();
                        }
                        KeyCode::Char('v') => {
                            if let Some(dag_id) = &self.dag_id {
                                return (
                                    None,
                                    vec![WorkerMessage::GetDagCode {
                                        dag_id: dag_id.clone(),
                                    }],
                                );
                            }
                        }
                        KeyCode::Char('c') => {
                            if let (Some(dag_run), Some(dag_id)) = (self.current(), &self.dag_id) {
                                self.popup = Some(DagRunPopUp::Clear(ClearDagRunPopup::new(
                                    dag_run.dag_run_id.clone(),
                                    dag_id.clone(),
                                )));
                            }
                        }
                        KeyCode::Enter => {
                            if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::UpdateTaskInstances {
                                        dag_id: dag_id.clone(),
                                        dag_run_id: dag_run.dag_run_id.clone(),
                                        clear: true,
                                    }],
                                );
                            }
                        }
                        KeyCode::Char('o') => {
                            if let (Some(dag_id), Some(dag_run)) = (&self.dag_id, &self.current()) {
                                return (
                                    Some(FlowrsEvent::Key(*key_event)),
                                    vec![WorkerMessage::OpenItem(OpenItem::DagRun {
                                        dag_id: dag_id.clone(),
                                        dag_run_id: dag_run.dag_run_id.clone(),
                                    })],
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        (Some(event.clone()), vec![])
    }
}

impl Widget for &mut DagRunModel {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer) {
        let rects = if self.filter.is_enabled() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)].as_ref())
                .margin(0)
                .split(area);

            self.filter.render(rects[1], buf);
            rects
        } else {
            Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
                .margin(0)
                .split(area)
        };

        let headers = ["State", "DAG Run ID", "Logical Date", "Type"];
        let header_row = create_headers(headers);
        let header =
            Row::new(header_row).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(match item.state.as_str() {
                    "success" => {
                        Span::styled("■", Style::default().fg(AirflowStateColor::Success.into()))
                    }
                    "running" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Running.into()))
                    }
                    "failed" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Failed.into()))
                    }
                    "queued" => {
                        Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::Queued.into()))
                    }
                    _ => Span::styled("■", DEFAULT_STYLE.fg(AirflowStateColor::None.into())),
                }),
                Line::from(Span::styled(
                    item.dag_run_id.as_str(),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(if let Some(date) = item.logical_date {
                    date.format(&format_description::parse(TIME_FORMAT).unwrap())
                        .unwrap()
                        .to_string()
                } else {
                    "None".to_string()
                }),
                Line::from(item.run_type.as_str()),
            ])
            .style(if self.marked.contains(&idx) {
                DEFAULT_STYLE.bg(MARKED_COLOR)
            } else if (idx % 2) == 0 {
                DEFAULT_STYLE
            } else {
                DEFAULT_STYLE.bg(ALTERNATING_ROW_COLOR)
            })
        });
        let t = Table::new(
            rows,
            &[
                Constraint::Length(6),
                Constraint::Fill(1),
                Constraint::Length(20),
                Constraint::Length(11),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({}) - press <?> to see available commands", dag_id)
                } else {
                    "DAGRuns".to_string()
                })
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(DEFAULT_STYLE.reversed());
        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(dag_code) = &self.dag_code.code {
            let area = popup_area(area, 60, 90);

            let popup = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("DAG Code")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE)
                .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

            let code_text = Paragraph::new(code_to_lines(dag_code))
                .block(popup)
                .style(DEFAULT_STYLE)
                .wrap(Wrap { trim: true })
                .scroll((self.dag_code.vertical_scroll as u16, 0));

            Clear.render(area, buf); //this clears out the background
            code_text.render(area, buf);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(area, buf, &mut self.dag_code.vertical_scroll_state);
        }

        match &mut self.popup {
            Some(DagRunPopUp::Clear(popup)) => {
                popup.render(area, buf);
            }
            Some(DagRunPopUp::Mark(popup)) => {
                popup.render(area, buf);
            }
            Some(DagRunPopUp::Trigger(popup)) => {
                popup.render(area, buf);
            }
            _ => (),
        }

        if let Some(commands) = &self.commands {
            commands.render(area, buf);
        }
    }
}

fn code_to_lines(dag_code: &str) -> Vec<Line> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension("py").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line> = vec![];
    for line in LinesWithEndings::from(dag_code) {
        // LinesWithEndings enables use of newlines mode
        let line_spans: Vec<ratatui::prelude::Span> = h
            .highlight_line(line, &ps)
            .unwrap()
            .into_iter()
            .filter_map(|segment| into_span(segment).ok())
            .collect::<Vec<ratatui::prelude::Span>>();
        lines.push(Line::from(line_spans));
    }
    lines
}
