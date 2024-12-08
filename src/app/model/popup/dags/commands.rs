use std::sync::LazyLock;

use crate::app::model::popup::commands_help::{Command, CommandPopUp};

pub static DAG_COMMAND_POP_UP: LazyLock<CommandPopUp> = LazyLock::new(|| CommandPopUp {
    title: "DAG Commands".into(),
    commands: vec![
        Command {
            name: "Pauze",
            key_binding: "p",
            description: "Pauze/unpauze a DAG",
        },
        Command {
            name: "Filter",
            key_binding: "/",
            description: "Filter DAgs",
        },
    ],
});
