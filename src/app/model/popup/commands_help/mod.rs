mod render;

pub struct Command<'a> {
    pub name: &'a str,
    pub key_binding: &'a str,
    pub description: &'a str,
}

pub struct CommandPopUp<'a> {
    pub title: String,
    pub commands: Vec<Command<'a>>,
}

pub struct DefaultCommands(pub Vec<Command<'static>>);

impl Default for DefaultCommands {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultCommands {
    pub fn new() -> Self {
        Self(vec![
            Command {
                name: "Enter",
                key_binding: "Enter",
                description: "Open the selected item",
            },
            Command {
                name: "Filter",
                key_binding: "/",
                description: "Filter items",
            },
            Command {
                name: "Open",
                key_binding: "o",
                description: "Open the selected item in the browser",
            },
            Command {
                name: "Previous",
                key_binding: "k / Up",
                description: "Move to the previous item",
            },
            Command {
                name: "Next",
                key_binding: "j / Down",
                description: "Move to the next item",
            },
            Command {
                name: "Previous tab",
                key_binding: "h / Left / Esc",
                description: "Move to the previous tab",
            },
            Command {
                name: "Next tab",
                key_binding: "l / Right",
                description: "Move to the next tab",
            },
            Command {
                name: "Help",
                key_binding: "?",
                description: "Show help",
            },
            Command {
                name: "Quit",
                key_binding: "q / Ctrl-c",
                description: "Quit",
            },
        ])
    }
}
