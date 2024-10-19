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

    pub fn matches(&self, matchee: &str) -> bool {
        match &self.prefix {
            Some(prefix) => matchee.contains(prefix),
            None => true,
        }
    }
}
