use crate::app::model::popup::commands_help::{Command, CommandPopUp};

pub const DAG_COMMAND_POP_UP: CommandPopUp<2> = CommandPopUp {
    title: "DAG Commands",
    commands: [
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
};
