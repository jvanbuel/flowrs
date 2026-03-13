use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp, DefaultCommands};

pub static DAGRUN_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![
        Command {
            name: "Clear",
            key_binding: "c",
            description: "Clear a DAG run",
        },
        Command {
            name: "Show",
            key_binding: "v",
            description: "Show DAG code",
        },
        Command {
            name: "Visual",
            key_binding: "V",
            description: "Enter visual selection mode",
        },
        Command {
            name: "Mark",
            key_binding: "m",
            description: "Mark selected DAG run(s)",
        },
        Command {
            name: "Trigger",
            key_binding: "t",
            description: "Trigger a DAG run",
        },
    ];
    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "DAG Run Commands".into(),
        commands,
    }
});
