use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp, DefaultCommands};

pub static DAG_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![
        Command {
            name: "Toggle pause",
            key_binding: "p",
            description: "Toggle pause/unpause a DAG",
        },
        Command {
            name: "Trigger",
            key_binding: "t",
            description: "Trigger a DAG run",
        },
    ];
    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "DAG Commands".into(),
        commands,
    }
});
