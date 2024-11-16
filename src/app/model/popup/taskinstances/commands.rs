use crate::app::model::popup::commands_help::{Command, CommandPopUp};

pub const TASK_COMMAND_POP_UP: CommandPopUp<4> = CommandPopUp {
    title: "DAG Commands",
    commands: [
        Command {
            name: "Clear",
            key_binding: "c",
            description: "Clear a DAG run",
        },
        Command {
            name: "Mark",
            key_binding: "m",
            description: "Mark a DAG run",
        },
        Command {
            name: "Trigger",
            key_binding: "t",
            description: "Trigger a DAG run",
        },
        Command {
            name: "Filter",
            key_binding: "/",
            description: "Filter DAG runs",
        },
    ],
};
