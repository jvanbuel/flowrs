use crossterm::event::KeyEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowrsEvent {
    Tick,
    Key(KeyEvent),
    Mouse,
    FocusGained,
    FocusLost,
}

impl From<crossterm::event::Event> for FlowrsEvent {
    fn from(ev: crossterm::event::Event) -> Self {
        match ev {
            crossterm::event::Event::Key(key) => Self::Key(key),
            crossterm::event::Event::Mouse(_) => Self::Mouse,
            crossterm::event::Event::FocusGained => Self::FocusGained,
            crossterm::event::Event::FocusLost => Self::FocusLost,
            _ => Self::Tick,
        }
    }
}
