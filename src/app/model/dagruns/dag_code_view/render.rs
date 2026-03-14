use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, StatefulWidget,
    Widget,
};

use crate::app::model::popup::popup_area;
use crate::ui::theme::theme;

use super::DagCodeView;

impl DagCodeView {
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let area = popup_area(area, 60, 90);
        let t = theme();

        let popup = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(" DAG Code ")
            .border_style(t.border_style)
            .style(t.surface_style)
            .title_style(t.title_style);

        #[allow(clippy::cast_possible_truncation)]
        let code_text = Paragraph::new(self.lines.clone())
            .block(popup)
            .style(t.default_style)
            .scroll((self.vertical_scroll as u16, 0));

        Clear.render(area, buf);
        code_text.render(area, buf);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        scrollbar.render(area, buf, &mut self.vertical_scroll_state);
    }
}
