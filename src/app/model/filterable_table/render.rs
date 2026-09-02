use std::ops::RangeInclusive;

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::app::model::filter::Filterable;
use crate::ui::theme::theme;

use super::FilterableTable;

/// The filtered rows of a [`FilterableTable`], detached from its selection
/// state by [`FilterableTable::rows_and_state`].
///
/// Holds only borrows of the canonical items and the view's index list plus
/// the visual-selection range captured at split time.
#[derive(Debug)]
pub struct TableRows<'a, T> {
    pub(super) all: &'a [T],
    pub(super) indices: &'a [usize],
    pub(super) visual: Option<RangeInclusive<usize>>,
}

impl<'a, T> TableRows<'a, T> {
    /// Iterate the filtered items in display order. The items outlive the
    /// view, so text borrowed from them can be placed straight into a `Row`.
    pub fn iter(&self) -> impl Iterator<Item = &'a T> + '_ {
        self.indices.iter().filter_map(|&i| self.all.get(i))
    }

    /// Number of rows in the filtered view.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Whether the filtered view is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// The inclusive range of visually selected positions, if in visual mode.
    pub fn visual_selection(&self) -> Option<RangeInclusive<usize>> {
        self.visual.clone()
    }

    /// Style for the row at `idx`; see [`FilterableTable::row_style`].
    pub fn row_style(&self, idx: usize) -> Style {
        row_style(self.visual.as_ref(), idx)
    }
}

/// `MARKED_STYLE` for visually selected rows, alternating
/// `DEFAULT_STYLE`/`ALT_ROW_STYLE` otherwise.
fn row_style(visual: Option<&RangeInclusive<usize>>, idx: usize) -> Style {
    let t = theme();
    if visual.is_some_and(|r| r.contains(&idx)) {
        t.marked_style
    } else if idx.is_multiple_of(2) {
        t.default_style
    } else {
        t.alt_row_style
    }
}

impl<T: Filterable> FilterableTable<T> {
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
        row_style(self.visual_selection().as_ref(), idx)
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
                    Style::default()
                        .fg(theme().accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" selected) -- | "),
                Span::styled(
                    format!("Filter: {filter} "),
                    Style::default()
                        .fg(theme().accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            (true, None) => Some(Line::from(vec![
                Span::raw(" -- VISUAL ("),
                Span::styled(
                    format!("{}", self.visual_selection_count()),
                    Style::default()
                        .fg(theme().accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" selected) -- "),
            ])),
            (false, Some(filter)) => Some(Line::from(Span::styled(
                format!(" Filter: {filter} "),
                Style::default()
                    .fg(theme().accent)
                    .add_modifier(Modifier::BOLD),
            ))),
            (false, None) => None,
        }
    }
}
