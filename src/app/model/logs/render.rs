use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        StatefulWidget, Tabs, Widget, Wrap,
    },
};
use unicode_width::UnicodeWidthStr;

use crate::ui::theme::theme;

use super::search::{Search, SearchData};
use super::{LogModel, ScrollMode};

// trim: false preserves the leading whitespace of wrapped continuation
// lines, which matters for indented log content (stack traces, JSON)
const WRAP: Wrap = Wrap { trim: false };

impl Widget for &mut LogModel {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let t = theme();
        self.search_cursor_position = None;

        if self.all.is_empty() {
            Paragraph::new("No logs available")
                .style(t.default_style)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                        .border_style(t.border_style),
                )
                .render(area, buffer);
            return;
        }

        let tab_titles = (0..self.all.len())
            .map(|i| format!("Task {}", i + 1))
            .collect::<Vec<String>>();

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(t.border_style),
            )
            .select(self.current % self.all.len())
            .highlight_style(Style::default().fg(t.accent).add_modifier(Modifier::BOLD))
            .style(t.default_style);

        // Render the tabs
        tabs.render(area, buffer);

        // Define the layout for content under the tabs, with an extra
        // search input box at the bottom while the search bar is open,
        // styled like the filter boxes of the table panels
        let chunks = if self.search.is_editing() {
            Layout::default()
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area)
        } else {
            Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area)
        };

        if let Some(log) = self.all.get(self.current % self.all.len()) {
            let content = match self.search.data() {
                Some(data) if !data.matches.is_empty() => highlighted_content(&log.content, data),
                _ => log.content.lines().map(Line::raw).collect(),
            };

            let line_count = self.current_line_count();
            let scroll_pos = match self.scroll_mode {
                // Search jumps target a source line; resolve it to a wrapped
                // position now that the render width is known
                ScrollMode::SourceLine { line } => {
                    let position = wrapped_offset(&log.content, line, chunks[1]);
                    self.scroll_mode = ScrollMode::Manual { position };
                    position
                }
                _ => self.scroll_mode.position(line_count),
            };
            self.vertical_scroll_state = self.vertical_scroll_state.position(scroll_pos);

            #[allow(clippy::cast_possible_truncation)]
            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Plain)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .title_bottom(self.bottom_title())
                        .border_style(t.border_style)
                        .title_style(t.title_style),
                )
                .wrap(WRAP)
                .style(t.default_style)
                .scroll((scroll_pos as u16, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            scrollbar.render(chunks[1], buffer, &mut self.vertical_scroll_state);
        }

        if let Search::Editing(data) = &self.search {
            let bar = chunks[2];
            Clear.render(bar, buffer);
            Paragraph::new(Line::raw(data.query.as_str()))
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::ALL)
                        .title("search")
                        .style(t.default_style),
                )
                .style(t.default_style)
                .render(bar, buffer);

            #[allow(clippy::cast_possible_truncation)]
            {
                self.search_cursor_position = Some(Position {
                    x: bar.x + 1 + data.query.width() as u16,
                    y: bar.y + 1,
                });
            }
        }

        if let Some(error_popup) = &self.error_popup {
            error_popup.render(area, buffer);
        }
    }
}

impl LogModel {
    fn bottom_title(&self) -> String {
        match &self.search {
            Search::Applied(data) if data.matches.is_empty() => {
                format!(" search: {} - no matches | Esc: clear ", data.query)
            }
            Search::Applied(data) => format!(
                " search: {} [{}/{}] | n/N: next/prev | Esc: clear ",
                data.query,
                data.current + 1,
                data.matches.len()
            ),
            Search::Editing(data) if !data.query.is_empty() => {
                format!(" {} matches ", data.matches.len())
            }
            _ if self.scroll_mode.is_following() => {
                " [F]ollow: ON - auto-scrolling | /: search ".to_string()
            }
            _ => " [F]ollow: OFF - press G to resume | /: search ".to_string(),
        }
    }
}

/// Build the log text with every search match highlighted and the current
/// match emphasized. Matches carry byte ranges, so lines are simply sliced.
fn highlighted_content<'a>(content: &'a str, data: &SearchData) -> Text<'a> {
    let t = theme();
    let current = data.current_match();
    let mut matches = data.matches.iter().peekable();
    let mut text = Text::default();
    for (line_idx, line) in content.lines().enumerate() {
        let mut spans = Vec::new();
        let mut pos = 0;
        while let Some(m) = matches.next_if(|m| m.line <= line_idx) {
            // Skip matches that no longer fit the content (stale between refreshes)
            let (Some(before), Some(hit)) = (line.get(pos..m.start), line.get(m.start..m.end))
            else {
                continue;
            };
            if !before.is_empty() {
                spans.push(Span::raw(before));
            }
            let style = if current == Some(*m) {
                t.search_current_match_style
            } else {
                t.search_match_style
            };
            spans.push(Span::styled(hit, style));
            pos = m.end;
        }
        if !line[pos..].is_empty() {
            spans.push(Span::raw(&line[pos..]));
        }
        text.push_line(Line::from(spans));
    }
    text
}

/// Wrapped-row offset of `line` when the content is rendered into `area`,
/// computed with the same word-wrapping the paragraph itself uses.
fn wrapped_offset(content: &str, line: usize, area: Rect) -> usize {
    let inner_width = area.width.saturating_sub(2); // block borders
    if inner_width == 0 || line == 0 {
        return line;
    }
    let preceding: Text = content.lines().take(line).map(Line::raw).collect();
    Paragraph::new(preceding).wrap(WRAP).line_count(inner_width)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::model::logs::search::find_matches;

    fn search_data(content: &str, query: &str, current: usize) -> SearchData {
        SearchData {
            query: query.to_string(),
            matches: find_matches(content, query),
            current,
        }
    }

    #[test]
    fn highlights_every_occurrence_and_emphasizes_the_current_one() {
        let content = "error and error";
        for (current, current_span) in [(0, 0), (1, 2)] {
            let text = highlighted_content(content, &search_data(content, "error", current));
            let spans = &text.lines[0].spans;
            assert_eq!(
                spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>(),
                vec!["error", " and ", "error"]
            );
            for span_idx in [0, 2] {
                let expected = if span_idx == current_span {
                    theme().search_current_match_style
                } else {
                    theme().search_match_style
                };
                assert_eq!(spans[span_idx].style, expected);
            }
        }
    }

    #[test]
    fn stale_out_of_range_matches_are_skipped() {
        let data = search_data("a long enough line with error", "error", 0);
        // Content shrank since matches were computed
        let text = highlighted_content("short", &data);
        assert_eq!(text.lines[0].spans[0].content, "short");
    }

    #[test]
    fn render_with_open_search_bar_shows_query_and_cursor() {
        let mut model = LogModel::default();
        model.update_logs(vec![crate::airflow::model::common::Log {
            continuation_token: None,
            content: "some error line".to_string(),
        }]);
        model.search = Search::Editing(search_data("some error line", "error", 0));

        let area = Rect::new(0, 0, 40, 12);
        let mut buffer = Buffer::empty(area);
        (&mut model).render(area, &mut buffer);

        // The query sits inside the bordered search box at the bottom
        let bar_row: String = (0..area.width)
            .map(|x| buffer[(x, area.height - 2)].symbol())
            .collect();
        assert!(bar_row.contains("error"));
        assert_eq!(model.search_cursor_position, Some(Position { x: 6, y: 10 }));

        // Locate the rendered log line and assert exactly the matched cells
        // carry the current-match style
        let row = |y: u16| -> String { (0..area.width).map(|x| buffer[(x, y)].symbol()).collect() };
        let y = (0..area.height)
            .find(|&y| row(y).contains("some error line"))
            .expect("log line not rendered");
        // Convert the byte offset to a cell index (border glyphs are multi-byte)
        let line = row(y);
        let byte_offset = line.find("error").unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let start = line[..byte_offset].chars().count() as u16;
        let current_modifier = theme().search_current_match_style.add_modifier;
        for x in 0..area.width {
            let styled = buffer[(x, y)].style().add_modifier == current_modifier;
            assert_eq!(styled, (start..start + 5).contains(&x), "cell x={x}");
        }
    }

    #[test]
    fn wrapped_offset_counts_wrapped_rows() {
        let area = Rect::new(0, 0, 12, 10); // inner width 10
        let long = "x".repeat(25); // wraps to 3 rows at width 10
        let content = format!("{long}\nshort\ntarget");
        assert_eq!(wrapped_offset(&content, 0, area), 0);
        assert_eq!(wrapped_offset(&content, 1, area), 3);
        assert_eq!(wrapped_offset(&content, 2, area), 4);
    }
}
