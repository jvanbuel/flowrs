use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::model::filter::Filterable;
use crate::ui::theme::{ACCENT, ALT_ROW_STYLE, DEFAULT_STYLE, MARKED_STYLE};

use super::FilterableTable;

impl<T: Filterable + Clone> FilterableTable<T> {
    /// Renders the filter widget if active and returns the content area.
    ///
    /// This handles the common pattern of splitting the area for filter input.
    pub fn render_with_filter(&mut self, area: Rect, buf: &mut Buffer) -> Rect {
        if self.filter.is_active() {
            let rects = Layout::default()
                .constraints([Constraint::Fill(90), Constraint::Max(3)])
                .split(area);
            self.filter.render_widget(rects[1], buf);
            rects[0]
        } else {
            area
        }
    }

    /// Returns the style for a row based on its index and visual selection state.
    ///
    /// Uses `MARKED_STYLE` for visually selected rows, alternating `DEFAULT_STYLE`/`ALT_ROW_STYLE` otherwise.
    pub fn row_style(&self, idx: usize) -> Style {
        if self
            .visual_selection()
            .as_ref()
            .is_some_and(|r| r.contains(&idx))
        {
            MARKED_STYLE
        } else if idx.is_multiple_of(2) {
            DEFAULT_STYLE
        } else {
            ALT_ROW_STYLE
        }
    }

    /// Returns the bottom title line for table blocks, showing visual mode and/or filter status.
    ///
    /// Returns None if neither visual mode nor filter is active.
    pub fn status_title(&self) -> Option<Line<'static>> {
        let filter_text = self.filter.filter_display();
        match (self.visual_anchor.is_some(), filter_text) {
            (true, Some(filter)) => Some(Line::from(vec![
                Span::raw(" -- VISUAL ("),
                Span::styled(
                    format!("{}", self.visual_selection_count()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" selected) -- | "),
                Span::styled(
                    format!("Filter: {filter} "),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ),
            ])),
            (true, None) => Some(Line::from(vec![
                Span::raw(" -- VISUAL ("),
                Span::styled(
                    format!("{}", self.visual_selection_count()),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" selected) -- "),
            ])),
            (false, Some(filter)) => Some(Line::from(Span::styled(
                format!(" Filter: {filter} "),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ))),
            (false, None) => None,
        }
    }
}
