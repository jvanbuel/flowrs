use crossterm::event::KeyEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum FlowrsEvent {
    Tick,
    Key(KeyEvent),
    Mouse,
}

impl From<crossterm::event::Event> for FlowrsEvent {
    fn from(ev: crossterm::event::Event) -> Self {
        match ev {
            crossterm::event::Event::Key(key) => FlowrsEvent::Key(key),
            crossterm::event::Event::Mouse(_) => FlowrsEvent::Mouse,
            _ => FlowrsEvent::Tick,
        }
    }
}
