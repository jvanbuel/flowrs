use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum CustomEvent<I, J> {
    Tick,
    Key(I),
    Mouse(J),
}

impl From<crossterm::event::Event> for CustomEvent<KeyEvent, MouseEvent> {
    fn from(ev: crossterm::event::Event) -> Self {
        match ev {
            crossterm::event::Event::Key(key) => CustomEvent::Key(key),
            crossterm::event::Event::Mouse(mouse) => CustomEvent::Mouse(mouse),
            _ => CustomEvent::Tick,
        }
    }
}
