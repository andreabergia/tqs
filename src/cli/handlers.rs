use crate::app::app_error::AppError;

use super::args::{Cli, Command};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Create { .. }) => {
            println!("create is not implemented yet");
            Ok(())
        }
        Some(Command::List { .. }) => {
            println!("list is not implemented yet");
            Ok(())
        }
        Some(Command::Complete { .. }) => {
            println!("complete is not implemented yet");
            Ok(())
        }
        Some(Command::Reopen { .. }) => {
            println!("reopen is not implemented yet");
            Ok(())
        }
        Some(Command::Info { .. }) => {
            println!("info is not implemented yet");
            Ok(())
        }
        Some(Command::Delete { .. }) => {
            println!("delete is not implemented yet");
            Ok(())
        }
        None => Err(AppError::usage("no command specified")),
    }
}
