use crate::app::model::popup::commands_help::{Command, CommandPopUp};

pub const DAGRUN_COMMAND_POP_UP: CommandPopUp<6> = CommandPopUp {
    title: "DAG Commands",
    commands: [
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
            name: "Mark",
            key_binding: "m",
            description: "Mark a DAG run",
        },
        Command {
            name: "Mark multiple",
            key_binding: "M",
            description: "Mark multiple DAG runs",
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
