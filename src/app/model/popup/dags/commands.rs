use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp, DefaultCommands};

pub static DAG_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| {
    let mut commands = vec![Command {
        name: "Toggle pauze",
        key_binding: "p",
        description: "Toggle pauze/unpauze a DAG",
    }];
    commands.append(&mut DefaultCommands::new().0);
    CommandPopUp {
        title: "DAG Commands".into(),
        commands,
    }
});
