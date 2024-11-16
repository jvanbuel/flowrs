use crossterm::event::{KeyCode, KeyEvent};

pub struct Filter {
    pub enabled: bool,
    pub prefix: Option<String>,
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            enabled: false,
            prefix: None,
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn prefix(&self) -> &Option<String> {
        &self.prefix
    }

    pub fn reset(&mut self) {
        self.enabled = false;
        self.prefix = None;
    }

    pub fn update(&mut self, key_event: &KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.toggle();
            }
            KeyCode::Backspace => {
                if let Some(ref mut prefix) = self.prefix {
                    prefix.pop();
                }
            }
            KeyCode::Char(c) => match self.prefix {
                Some(ref mut prefix) => {
                    prefix.push(c);
                }
                None => {
                    self.prefix = Some(c.to_string());
                }
            },
            _ => {}
        }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
