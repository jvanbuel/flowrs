use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp, DefaultCommands};

pub static CONFIG_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![Command {
        name: "Open",
        key_binding: "o",
        description: "Open Airflow Web UI",
    }];
    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "Config Commands".into(),
        commands,
    }
});
