use std::hash::{DefaultHasher, Hash, Hasher};

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

// `trim: false` keeps leading whitespace on wrapped rows, so indentation in
// stack traces, JSON and nested log output survives wrapping.
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
            .select(self.current)
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

        if let Some(log) = self.all.get(self.current) {
            // Wrapping is the expensive part of rendering a log, so only the
            // source lines covering the visible rows are handed to the
            // paragraph; the cache maps wrapped rows back to source lines.
            let inner_width = chunks[1].width.saturating_sub(2); // block borders
            let inner_height = usize::from(chunks[1].height.saturating_sub(2));
            self.wrap_cache
                .update(&log.content, (self.generation, self.current, inner_width));
            let cache = &self.wrap_cache;

            let line_count = cache.line_count();
            let scroll_pos = match self.scroll_mode {
                // Search jumps target a source line; resolve it to a wrapped
                // position now that the render width is known
                ScrollMode::SourceLine { line } => {
                    let position = cache.row_of_line(line);
                    self.scroll_mode = ScrollMode::Manual { position };
                    position
                }
                _ => self.scroll_mode.position(line_count),
            };

            let visible = cache.lines_covering(scroll_pos, inner_height);
            let content = match self.search.data() {
                Some(data) if !data.matches.is_empty() => {
                    let mut text = highlighted_content(&log.content, data);
                    text.lines.truncate(visible.end);
                    text.lines.drain(..visible.start);
                    text
                }
                _ => cache
                    .slice(&log.content, visible.start)
                    .lines()
                    .take(visible.len())
                    .map(Line::raw)
                    .collect(),
            };
            // Rows of the first visible line that sit above the viewport.
            let row_offset =
                u16::try_from(scroll_pos - cache.row_of_line(visible.start)).unwrap_or(u16::MAX);

            let paragraph = Paragraph::new(content)
                .block(
                    Block::default()
                        .border_type(BorderType::Rounded)
                        .borders(Borders::ALL)
                        .title(" Content ")
                        .title_bottom(self.bottom_title())
                        .border_style(t.border_style)
                        .title_style(t.title_style),
                )
                .wrap(WRAP)
                .style(t.default_style)
                .scroll((row_offset, 0));

            // Render the selected log's content
            paragraph.render(chunks[1], buffer);

            // The log borrow (held by `content`) has ended, so the scroll state
            // can now be updated before rendering the scrollbar.
            self.vertical_scroll_state = self.vertical_scroll_state.position(scroll_pos);

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

            #[expect(
                clippy::cast_possible_truncation,
                reason = "value is bounded by terminal/layout dimensions and stays well within the target integer range"
            )]
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

/// Identifies what a [`WrapCache`] was built for: the log model's content
/// generation, the selected try, and the wrap width.
type WrapKey = (u64, usize, u16);

/// Wrapped-row layout of one log, so a frame only wraps the lines it shows.
///
/// Logs can run to tens of thousands of lines, and word-wrapping all of them
/// on every frame stalls the UI. Each line wraps independently, so knowing
/// where every source line starts (in bytes and in wrapped rows) is enough to
/// render any window of the log exactly as a full-text paragraph would.
#[derive(Debug, Default)]
pub(crate) struct WrapCache {
    key: Option<WrapKey>,
    content_len: usize,
    content_hash: u64,
    /// Byte offset where each source line starts.
    line_starts: Vec<usize>,
    /// Wrapped row where each source line starts, plus the total row count.
    row_starts: Vec<usize>,
}

impl WrapCache {
    /// Bring the layout up to date with `content` under `key`. Unchanged
    /// content is reused, and content that only grew (a running task's log)
    /// is re-wrapped from its last line on.
    fn update(&mut self, content: &str, key: WrapKey) {
        if self.key == Some(key) && self.content_len == content.len() {
            return;
        }
        let (_, tab, width) = key;
        let same_layout = self.key.is_some_and(|(_, t, w)| t == tab && w == width);
        let prefix_unchanged = same_layout
            && content
                .get(..self.content_len)
                .is_some_and(|prefix| hash_str(prefix) == self.content_hash);

        if prefix_unchanged && self.content_len == content.len() {
            self.key = Some(key);
            return;
        }
        let (mut byte, mut row) = (0, 0);
        if prefix_unchanged && !self.line_starts.is_empty() {
            // The last cached line may have been extended, so redo it too.
            self.row_starts.pop(); // the total row count
            byte = self.line_starts.pop().unwrap_or(0);
            row = self.row_starts.pop().unwrap_or(0);
        } else {
            self.line_starts.clear();
            self.row_starts.clear();
        }
        for line in content[byte..].split_inclusive('\n') {
            self.line_starts.push(byte);
            self.row_starts.push(row);
            byte += line.len();
            row += wrapped_height(line, width);
        }
        self.row_starts.push(row);

        self.key = Some(key);
        self.content_len = content.len();
        self.content_hash = hash_str(content);
    }

    /// Number of source lines in the log.
    fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Wrapped row where source `line` starts (the total row count past the end).
    fn row_of_line(&self, line: usize) -> usize {
        self.row_starts[line.min(self.line_count())]
    }

    /// The source lines that cover `height` rows starting at wrapped row `row`.
    fn lines_covering(&self, row: usize, height: usize) -> std::ops::Range<usize> {
        let lines = &self.row_starts[..self.line_count()];
        let first = lines
            .partition_point(|&start| start <= row)
            .saturating_sub(1);
        let end = lines.partition_point(|&start| start < row.saturating_add(height));
        first..end.max(first)
    }

    /// `content` from the start of source `line` on.
    fn slice<'a>(&self, content: &'a str, line: usize) -> &'a str {
        self.line_starts
            .get(line)
            .map_or("", |&start| &content[start..])
    }
}

/// Rows `line` takes up when word-wrapped to `width`, the way the log
/// paragraph wraps it.
fn wrapped_height(line: &str, width: u16) -> usize {
    let line = line.strip_suffix('\n').unwrap_or(line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    if width == 0 {
        return 1;
    }
    Paragraph::new(Line::raw(line)).wrap(WRAP).line_count(width)
}

fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
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
    fn highlights_every_occurrence() {
        let content = "error and error";
        let text = highlighted_content(content, &search_data(content, "error", 0));
        let spans = &text.lines[0].spans;
        assert_eq!(
            spans.iter().map(|s| s.content.as_ref()).collect::<Vec<_>>(),
            vec!["error", " and ", "error"]
        );
        assert_eq!(spans[0].style, theme().search_current_match_style);
        assert_eq!(spans[2].style, theme().search_match_style);
    }

    #[test]
    fn current_match_style_follows_current_index() {
        let content = "error and error";
        let text = highlighted_content(content, &search_data(content, "error", 1));
        let spans = &text.lines[0].spans;
        assert_eq!(spans[0].style, theme().search_match_style);
        assert_eq!(spans[2].style, theme().search_current_match_style);
    }

    #[test]
    fn lines_without_matches_are_left_intact() {
        let content = "foo\nbar error\nbaz";
        let text = highlighted_content(content, &search_data(content, "error", 0));
        assert_eq!(text.lines.len(), 3);
        assert_eq!(text.lines[0].spans[0].content, "foo");
        assert_eq!(text.lines[2].spans[0].content, "baz");
        assert_eq!(
            text.lines[1].spans[1].style,
            theme().search_current_match_style
        );
    }

    use crate::airflow::model::common::Log;

    /// The log's content rows as rendered into a `width`x`height` area.
    fn content_rows(model: &mut LogModel, width: u16, height: u16) -> Vec<String> {
        let area = Rect::new(0, 0, width, height);
        let mut buffer = Buffer::empty(area);
        model.render(area, &mut buffer);
        // Skip the tabs (3 rows), the content borders and the scrollbar column.
        (4..height - 1)
            .map(|y| (1..width - 1).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    /// The same rows from a paragraph holding the whole log, as the panel
    /// rendered it before only visible lines were wrapped.
    fn full_paragraph_rows(content: &str, scroll: u16, width: u16, height: u16) -> Vec<String> {
        let area = Rect::new(0, 0, width - 2, height - 5);
        let mut buffer = Buffer::empty(area);
        let text: Text = content.lines().map(Line::raw).collect();
        Paragraph::new(text)
            .wrap(WRAP)
            .scroll((scroll, 0))
            .render(area, &mut buffer);
        (0..area.height)
            .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    fn sample_log(lines: usize) -> String {
        (0..lines)
            .map(|i| match i % 4 {
                0 => format!("line {i}"),
                1 => format!("line {i} {}", "word ".repeat(i % 13 + 3)),
                2 => String::new(),
                _ => format!("  indented {i} {}", "x".repeat(i % 50)),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn windowed_render_matches_full_paragraph() {
        let (width, height) = (32, 14);
        let content = sample_log(60);
        let mut model = LogModel::default();
        model.update_logs(vec![Log {
            continuation_token: None,
            content: content.clone(),
        }]);
        for position in 0..200 {
            model.scroll_mode = ScrollMode::Manual { position };
            assert_eq!(
                content_rows(&mut model, width, height),
                full_paragraph_rows(&content, u16::try_from(position).unwrap(), width, height),
                "scroll position {position}"
            );
        }
    }

    #[test]
    fn wrap_cache_follows_appended_and_replaced_content() {
        let (width, height) = (32, 14);
        let mut model = LogModel::default();
        for content in [
            sample_log(30),
            format!("{} and more", sample_log(30)),
            format!("{} and more\n{}", sample_log(30), sample_log(25)),
            sample_log(10),
        ] {
            model.update_logs(vec![Log {
                continuation_token: None,
                content: content.clone(),
            }]);
            for position in [0, 7, 40, 90] {
                model.scroll_mode = ScrollMode::Manual { position };
                assert_eq!(
                    content_rows(&mut model, width, height),
                    full_paragraph_rows(&content, u16::try_from(position).unwrap(), width, height),
                    "scroll position {position}"
                );
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

        // The matched text is styled with the current-match style
        let hit_x = (0..area.height)
            .flat_map(|y| (0..area.width).map(move |x| (x, y)))
            .find(|&(x, y)| {
                buffer[(x, y)].symbol() == "e"
                    && buffer[(x, y)].style().add_modifier
                        == theme().search_current_match_style.add_modifier
            });
        assert!(hit_x.is_some(), "no styled match cell found in buffer");
    }

    #[test]
    fn row_of_line_counts_wrapped_rows() {
        let long = "x".repeat(25); // wraps to 3 rows at width 10
        let content = format!("{long}\nshort\ntarget");
        let mut cache = WrapCache::default();
        cache.update(&content, (0, 0, 10));
        assert_eq!(cache.row_of_line(0), 0);
        assert_eq!(cache.row_of_line(1), 3);
        assert_eq!(cache.row_of_line(2), 4);
        assert_eq!(cache.row_of_line(3), 5); // past the end: total rows
    }

    #[test]
    fn wrapping_keeps_indentation() {
        let mut model = LogModel::default();
        model.update_logs(vec![crate::airflow::model::common::Log {
            continuation_token: None,
            content: format!("    {}", "x".repeat(40)),
        }]);

        let area = Rect::new(0, 0, 20, 12);
        let mut buffer = Buffer::empty(area);
        (&mut model).render(area, &mut buffer);

        // The indent must survive on the first content row
        let row: String = (0..area.width)
            .map(|x| buffer[(x, 4)].symbol())
            .collect::<String>();
        assert!(row.contains("    xxx"), "indent was trimmed: {row:?}");
    }
}
