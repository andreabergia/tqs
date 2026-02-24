use crate::app::app_error::AppError;

use super::args::{Cli, Command};
use super::commands::{complete, create, delete, edit, info, list, move_cmd, reopen};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Create(create_cmd)) => {
            create::handle_create(create_cmd, cli.root, cli.global)
        }
        Some(Command::List(list_cmd)) => list::handle_list(list_cmd, cli.root, cli.global),
        Some(Command::Complete(complete_cmd)) => {
            complete::handle_complete(complete_cmd, cli.root, cli.global)
        }
        Some(Command::Reopen(reopen_cmd)) => {
            reopen::handle_reopen(reopen_cmd, cli.root, cli.global)
        }
        Some(Command::Info(info_cmd)) => info::handle_info(info_cmd, cli.root, cli.global),
        Some(Command::Delete(delete_cmd)) => {
            delete::handle_delete(delete_cmd, cli.root, cli.global)
        }
        Some(Command::Move(move_cmd)) => move_cmd::handle_move(move_cmd, cli.root, cli.global),
        Some(Command::Edit(edit_cmd)) => edit::handle_edit(edit_cmd, cli.root, cli.global),
        None => Err(AppError::usage("no command specified")),
    }
}
