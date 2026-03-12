use crate::app::app_error::AppError;

use super::args::{Cli, Command};
use super::commands::{add, config, doctor, done, edit, find, inbox, list, move_cmd, now, show};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Add(command)) => add::handle_add(command, cli.root),
        Some(Command::List(command)) => list::handle_list(command, cli.root),
        Some(Command::Now(command)) => now::handle_now(command, cli.root),
        Some(Command::Inbox(command)) => inbox::handle_inbox(command, cli.root),
        Some(Command::Move(command)) => move_cmd::handle_move(command, cli.root),
        Some(Command::Done(command)) => done::handle_done(command, cli.root),
        Some(Command::Edit(command)) => edit::handle_edit(command, cli.root),
        Some(Command::Show(command)) => show::handle_show(command, cli.root),
        Some(Command::Find(command)) => find::handle_find(command, cli.root),
        Some(Command::Config(command)) => config::handle_config(command, cli.root),
        Some(Command::Doctor(command)) => doctor::handle_doctor(command, cli.root),
        None => Err(AppError::usage("no command specified")),
    }
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::app::app_error::AppError;
    use crate::cli::args::Cli;

    #[test]
    fn handle_returns_usage_error_when_no_command_is_specified() {
        let err = handle(Cli {
            root: None,
            command: None,
        })
        .expect_err("missing command should fail");

        assert!(matches!(err, AppError::Usage(_)));
        assert_eq!(err.to_string(), "no command specified");
    }
}
