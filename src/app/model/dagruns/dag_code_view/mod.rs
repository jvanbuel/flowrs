mod render;

use crossterm::event::KeyCode;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ScrollbarState;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct DagCodeView {
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) vertical_scroll: usize,
    pub(crate) vertical_scroll_state: ScrollbarState,
    event_buffer: Vec<KeyCode>,
}

impl DagCodeView {
    pub fn new(code: &str) -> Self {
        let lines = code_to_lines(code);
        let content_length = lines.len();
        Self {
            lines,
            vertical_scroll: 0,
            vertical_scroll_state: ScrollbarState::default().content_length(content_length),
            event_buffer: Vec::new(),
        }
    }

    /// Handle a key event. Returns `true` if the view should be closed.
    pub fn update(&mut self, key_code: KeyCode) -> bool {
        match key_code {
            KeyCode::Esc | KeyCode::Char('q' | 'v') | KeyCode::Enter => return true,
            KeyCode::Down | KeyCode::Char('j') => {
                self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Char('G') => {
                self.vertical_scroll = self.lines.len().saturating_sub(1);
                self.vertical_scroll_state =
                    self.vertical_scroll_state.position(self.vertical_scroll);
            }
            KeyCode::Char('g') => {
                if let Some(KeyCode::Char('g')) = self.event_buffer.pop() {
                    self.vertical_scroll = 0;
                    self.vertical_scroll_state = self.vertical_scroll_state.position(0);
                } else {
                    self.event_buffer.push(key_code);
                }
            }
            _ => {}
        }
        false
    }
}

fn code_to_lines(dag_code: &str) -> Vec<Line<'static>> {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps
        .find_syntax_by_extension("py")
        .expect("Python syntax definition should be available in default syntax set");
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    let mut lines: Vec<Line<'static>> = vec![];
    for line in LinesWithEndings::from(dag_code) {
        let line_spans: Vec<Span<'static>> = h
            .highlight_line(line, &ps)
            .expect("Syntax highlighting should succeed for valid Python code")
            .into_iter()
            .map(|(style, text)| {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                Span::styled(text.to_string(), Style::default().fg(fg))
            })
            .collect();
        lines.push(Line::from(line_spans));
    }
    lines
}
