use crate::app::app_error::AppError;

use super::args::{Cli, Command};
use super::commands::{complete, create, delete, info, list, reopen};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Create(create_cmd)) => create::handle_create(create_cmd, cli.root),
        Some(Command::List(list_cmd)) => list::handle_list(list_cmd, cli.root),
        Some(Command::Complete(complete_cmd)) => complete::handle_complete(complete_cmd, cli.root),
        Some(Command::Reopen(reopen_cmd)) => reopen::handle_reopen(reopen_cmd, cli.root),
        Some(Command::Info(info_cmd)) => info::handle_info(info_cmd, cli.root),
        Some(Command::Delete(delete_cmd)) => delete::handle_delete(delete_cmd, cli.root),
        None => Err(AppError::usage("no command specified")),
    }
}
