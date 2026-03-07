use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        Block, BorderType, Borders, Clear, Paragraph, Widget,
    },
};
use time::{Date, Duration, Month, OffsetDateTime};

use crate::{
    airflow::traits::DagRunDateFilter,
    app::{
        events::custom::FlowrsEvent,
        model::{popup::popup_area, Model},
        worker::WorkerMessage,
    },
    ui::theme::{ACCENT, BORDER_STYLE, DEFAULT_STYLE, PURPLE, PURPLE_DIM, SURFACE_STYLE},
};

/// Which date field the user is currently editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateField {
    Start,
    End,
}

/// Popup for selecting a date range filter using two calendar widgets.
pub struct DateFilterPopup {
    /// Currently selected date on the calendar cursor
    cursor_date: Date,
    /// The month/year being displayed
    display_date: Date,
    /// Which field is being edited
    active_field: DateField,
    /// Selected start date
    pub start_date: Option<Date>,
    /// Selected end date
    pub end_date: Option<Date>,
    /// Whether the popup was confirmed (Enter pressed)
    pub confirmed: bool,
}

impl DateFilterPopup {
    pub fn new(existing_filter: &DagRunDateFilter) -> Self {
        let today = OffsetDateTime::now_utc().date();
        Self {
            cursor_date: existing_filter.start_date.unwrap_or(today),
            display_date: existing_filter.start_date.unwrap_or(today),
            active_field: DateField::Start,
            start_date: existing_filter.start_date,
            end_date: existing_filter.end_date,
            confirmed: false,
        }
    }

    /// Build the date filter from the current selections
    pub fn to_date_filter(&self) -> DagRunDateFilter {
        DagRunDateFilter {
            start_date: self.start_date,
            end_date: self.end_date,
        }
    }

    fn move_cursor(&mut self, days: i64) {
        if let Some(new_date) = self.cursor_date.checked_add(Duration::days(days)) {
            self.cursor_date = new_date;
            // Update display month if cursor moved to a different month
            if self.cursor_date.month() != self.display_date.month()
                || self.cursor_date.year() != self.display_date.year()
            {
                self.display_date = self.cursor_date;
            }
        }
    }

    fn next_month(&mut self) {
        let month = self.display_date.month().next();
        let year = if month == Month::January {
            self.display_date.year() + 1
        } else {
            self.display_date.year()
        };
        if let Ok(d) = Date::from_calendar_date(year, month, 1) {
            self.display_date = d;
            // Move cursor to first of new month
            self.cursor_date = d;
        }
    }

    fn prev_month(&mut self) {
        let month = self.display_date.month().previous();
        let year = if month == Month::December {
            self.display_date.year() - 1
        } else {
            self.display_date.year()
        };
        if let Ok(d) = Date::from_calendar_date(year, month, 1) {
            self.display_date = d;
            self.cursor_date = d;
        }
    }

    fn select_current_date(&mut self) {
        match self.active_field {
            DateField::Start => {
                self.start_date = Some(self.cursor_date);
                // Auto-advance to end date selection
                self.active_field = DateField::End;
            }
            DateField::End => {
                self.end_date = Some(self.cursor_date);
            }
        }
    }
}

impl Model for DateFilterPopup {
    fn update(
        &mut self,
        event: &FlowrsEvent,
        _ctx: &crate::app::state::NavigationContext,
    ) -> (Option<FlowrsEvent>, Vec<WorkerMessage>) {
        if let FlowrsEvent::Key(key_event) = event {
            match key_event.code {
                // Navigation within the calendar
                KeyCode::Left | KeyCode::Char('h') => {
                    self.move_cursor(-1);
                    return (None, vec![]);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.move_cursor(1);
                    return (None, vec![]);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_cursor(-7);
                    return (None, vec![]);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_cursor(7);
                    return (None, vec![]);
                }
                // Month navigation
                KeyCode::Char('H') => {
                    self.prev_month();
                    return (None, vec![]);
                }
                KeyCode::Char('L') => {
                    self.next_month();
                    return (None, vec![]);
                }
                // Select date
                KeyCode::Char(' ') => {
                    self.select_current_date();
                    return (None, vec![]);
                }
                // Toggle between start/end field
                KeyCode::Tab => {
                    self.active_field = match self.active_field {
                        DateField::Start => DateField::End,
                        DateField::End => DateField::Start,
                    };
                    // Move cursor to the selected date of the new field, if any
                    match self.active_field {
                        DateField::Start => {
                            if let Some(d) = self.start_date {
                                self.cursor_date = d;
                                self.display_date = d;
                            }
                        }
                        DateField::End => {
                            if let Some(d) = self.end_date {
                                self.cursor_date = d;
                                self.display_date = d;
                            }
                        }
                    }
                    return (None, vec![]);
                }
                // Clear the active field
                KeyCode::Char('x') => {
                    match self.active_field {
                        DateField::Start => self.start_date = None,
                        DateField::End => self.end_date = None,
                    }
                    return (None, vec![]);
                }
                // Confirm
                KeyCode::Enter => {
                    self.confirmed = true;
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                // Cancel
                KeyCode::Esc | KeyCode::Char('q') => {
                    return (Some(FlowrsEvent::Key(*key_event)), vec![]);
                }
                _ => {}
            }
        }
        (None, vec![])
    }
}

impl Widget for &mut DateFilterPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = popup_area(area, 50, 70);

        let popup_block = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(BORDER_STYLE)
            .style(SURFACE_STYLE)
            .title(" Date Filter ");

        let inner = popup_block.inner(area);

        Clear.render(area, buf);
        popup_block.render(area, buf);

        // Layout: header, start label, calendar, end label, help text
        let [header_area, dates_area, calendar_area, help_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .areas(inner);

        // Header
        let header = Paragraph::new("Select date range for DAG runs")
            .style(DEFAULT_STYLE)
            .centered();
        header.render(header_area, buf);

        // Date fields display
        let [start_area, end_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(dates_area);

        let start_style = if self.active_field == DateField::Start {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            DEFAULT_STYLE
        };
        let end_style = if self.active_field == DateField::End {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            DEFAULT_STYLE
        };

        let start_text = self
            .start_date
            .map_or_else(|| "Start: <none>".to_string(), |d| format!("Start: {d}"));
        let end_text = self
            .end_date
            .map_or_else(|| "End: <none>".to_string(), |d| format!("End: {d}"));

        Paragraph::new(start_text)
            .style(start_style)
            .centered()
            .render(start_area, buf);
        Paragraph::new(end_text)
            .style(end_style)
            .centered()
            .render(end_area, buf);

        // Calendar rendering
        let mut events = CalendarEventStore::default();

        // Highlight cursor date
        events.add(
            self.cursor_date,
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        );

        // Highlight selected start date
        if let Some(start) = self.start_date {
            if start != self.cursor_date {
                events.add(
                    start,
                    Style::default()
                        .fg(Color::Black)
                        .bg(PURPLE)
                        .add_modifier(Modifier::BOLD),
                );
            }
        }

        // Highlight selected end date
        if let Some(end) = self.end_date {
            if end != self.cursor_date {
                events.add(
                    end,
                    Style::default()
                        .fg(Color::Black)
                        .bg(PURPLE)
                        .add_modifier(Modifier::BOLD),
                );
            }
        }

        // Highlight range between start and end
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            let (range_start, range_end) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            let mut d = range_start;
            while d <= range_end {
                if d != self.cursor_date && Some(d) != self.start_date && Some(d) != self.end_date {
                    events.add(d, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD));
                }
                if let Some(next) = d.next_day() {
                    d = next;
                } else {
                    break;
                }
            }
        }

        let calendar = Monthly::new(self.display_date, events)
            .show_month_header(Style::default().fg(PURPLE).add_modifier(Modifier::BOLD))
            .show_weekdays_header(Style::default().fg(PURPLE_DIM))
            .show_surrounding(Style::default().fg(Color::DarkGray))
            .default_style(DEFAULT_STYLE);

        // Center the calendar within its area
        let [_, cal_centered, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(calendar.width()),
            Constraint::Fill(1),
        ])
        .areas(calendar_area);

        let [_, cal_v_centered, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(calendar.height()),
            Constraint::Fill(1),
        ])
        .areas(cal_centered);

        calendar.render(cal_v_centered, buf);

        // Help text
        let help = Paragraph::new(
            "h/l: ±day  j/k: ±week  H/L: ±month  Space: select  Tab: toggle field  x: clear  Enter: apply  Esc: cancel",
        )
        .style(Style::default().fg(PURPLE_DIM))
        .centered();
        help.render(help_area, buf);
    }
}
