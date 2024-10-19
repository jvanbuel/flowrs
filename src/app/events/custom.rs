use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum FlowrsEvent {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    ConfigSelected(usize),
}

impl From<crossterm::event::Event> for FlowrsEvent {
    fn from(ev: crossterm::event::Event) -> Self {
        match ev {
            crossterm::event::Event::Key(key) => FlowrsEvent::Key(key),
            crossterm::event::Event::Mouse(mouse) => FlowrsEvent::Mouse(mouse),
            _ => FlowrsEvent::Tick,
        }
    }
}
