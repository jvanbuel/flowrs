use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp, DefaultCommands};

pub static TASK_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![
        Command {
            name: "Clear",
            key_binding: "c",
            description: "Clear a task instance",
        },
        Command {
            name: "Mark",
            key_binding: "m",
            description: "Mark a task instance",
        },
        Command {
            name: "Filter",
            key_binding: "/",
            description: "Filter task instances",
        },
    ];

    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "Task Commands".into(),
        commands,
    }
});
