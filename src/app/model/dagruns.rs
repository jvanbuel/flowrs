use crossterm::event::KeyCode;
use log::debug;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    StatefulWidget, Table, Widget, Wrap,
};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect_tui::into_span;
use time::format_description;

use crate::airflow::model::dagrun::DagRun;
use crate::app::events::custom::FlowrsEvent;
use crate::ui::constants::DEFAULT_STYLE;
use crate::ui::TIME_FORMAT;

use super::popup::ClearDagRunPopup;
use super::{filter::Filter, Model, StatefulTable};
use crate::app::error::FlowrsError;
use crate::app::worker::WorkerMessage;

pub struct DagRunModel {
    pub dag_id: Option<String>,
    pub dag_code: DagCodeWidget,
    pub all: Vec<DagRun>,
    pub filtered: StatefulTable<DagRun>,
    pub filter: Filter,
    pub marked: Vec<usize>,
    #[allow(dead_code)]
    pub errors: Vec<FlowrsError>,
    pub popup: Option<ClearDagRunPopup>,
    ticks: u32,
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
            ticks: 0,
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
                } else if let Some(popup) = &mut self.popup {
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
                        KeyCode::Char('t') => {
                            unimplemented!();
                        }
                        KeyCode::Char('m') => {
                            if let Some(index) = self.filtered.state.selected() {
                                self.marked.push(index);
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
                            self.popup = Some(ClearDagRunPopup::new(
                                self.current().unwrap().dag_run_id.clone(),
                                self.dag_id.clone().unwrap(),
                            ));
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
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(0)
            .split(area);

        let normal_style = DEFAULT_STYLE;

        let headers = ["State", "DAG Run ID", "Logical Date", "Type"];
        let header_cells = headers.iter().map(|h| Cell::from(*h).style(normal_style));
        let header =
            Row::new(header_cells).style(DEFAULT_STYLE.reversed().add_modifier(Modifier::BOLD));

        let rows = self.filtered.items.iter().enumerate().map(|(idx, item)| {
            Row::new(vec![
                Line::from(match item.state.as_str() {
                    "success" => Span::styled("■", Style::default().fg(Color::Rgb(0, 128, 0))),
                    "running" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightGreen)),
                    "failed" => Span::styled("■", DEFAULT_STYLE.fg(Color::Red)),
                    "queued" => Span::styled("■", DEFAULT_STYLE.fg(Color::LightBlue)),
                    _ => Span::styled("■", DEFAULT_STYLE.fg(Color::White)),
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
                Constraint::Length(7),
                Constraint::Percentage(20),
                Constraint::Max(22),
                Constraint::Length(10),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if let Some(dag_id) = &self.dag_id {
                    format!("DAGRuns ({}) - press <v> to view DAG code", dag_id)
                } else {
                    "DAGRuns press <v> to view DAG code".to_string()
                })
                .style(DEFAULT_STYLE),
        )
        .row_highlight_style(DEFAULT_STYLE.reversed());
        StatefulWidget::render(t, rects[0], buf, &mut self.filtered.state);

        if let Some(dag_code) = &self.dag_code.code {
            let area = popup_area(area, 60, 90);

            let popup = Block::default()
                .borders(Borders::ALL)
                .title("DAG Code - <j> down, <k> up, <q>|<Enter>|<Esc>|<v> close")
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

        if let Some(popup) = &self.popup {
            let area = popup_area(area, 50, 50);

            let [_, header, options, _] = Layout::vertical([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .flex(Flex::Center)
            .areas(area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .title("Clear DAG Run - press <Enter> to confirm, <q>|<Esc> to close")
                .border_style(DEFAULT_STYLE)
                .style(DEFAULT_STYLE)
                .title_style(DEFAULT_STYLE.add_modifier(Modifier::BOLD));

            let text = Paragraph::new("Are you sure you want to clear this DAG Run?")
                .style(DEFAULT_STYLE)
                .block(Block::default())
                .centered()
                .wrap(Wrap { trim: true });

            let [_, yes, _, no, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(7),
                Constraint::Percentage(5),
                Constraint::Length(7),
                Constraint::Fill(1),
            ])
            .areas(options);

            let yes_text = Paragraph::new("Yes")
                .style(if popup.confirm {
                    DEFAULT_STYLE.reversed()
                } else {
                    DEFAULT_STYLE
                })
                .centered()
                .block(Block::default().borders(Borders::ALL));

            let no_text = Paragraph::new("No")
                .style(if !popup.confirm {
                    DEFAULT_STYLE.reversed()
                } else {
                    DEFAULT_STYLE
                })
                .centered()
                .block(Block::default().borders(Borders::ALL));

            Clear.render(area, buf); //this clears out the background
            popup_block.render(area, buf);
            text.render(header, buf);
            yes_text.render(yes, buf);
            no_text.render(no, buf);
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
