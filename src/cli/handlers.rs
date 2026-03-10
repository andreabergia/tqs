use crate::app::app_error::AppError;

use super::args::{Cli, Command};
use super::commands::{add, config, done, edit, find, inbox, list, move_cmd, now, show};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Add(command)) => add::handle_add(command, cli.root, cli.global),
        Some(Command::List(command)) => list::handle_list(command, cli.root, cli.global),
        Some(Command::Now(command)) => now::handle_now(command, cli.root, cli.global),
        Some(Command::Inbox(command)) => inbox::handle_inbox(command, cli.root, cli.global),
        Some(Command::Move(command)) => move_cmd::handle_move(command, cli.root, cli.global),
        Some(Command::Done(command)) => done::handle_done(command, cli.root, cli.global),
        Some(Command::Edit(command)) => edit::handle_edit(command, cli.root, cli.global),
        Some(Command::Show(command)) => show::handle_show(command, cli.root, cli.global),
        Some(Command::Find(command)) => find::handle_find(command, cli.root, cli.global),
        Some(Command::Config(command)) => config::handle_config(command, cli.root, cli.global),
        None => Err(AppError::usage("no command specified")),
    }
}
