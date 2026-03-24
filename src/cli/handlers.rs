use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::config;

use super::args::{Cli, Command};
use super::commands::{
    add, config as config_cmd, delete, doctor, done, edit, find, helpers, inbox, list, move_cmd,
    now, show, start, triage,
};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Add(command)) => add::handle_add(command, cli.root),
        Some(Command::List(command)) => list::handle_list(command, cli.root),
        Some(Command::Now(command)) => now::handle_now(command, cli.root),
        Some(Command::Inbox(command)) => inbox::handle_inbox(command, cli.root),
        Some(Command::Move(command)) => move_cmd::handle_move(command, cli.root),
        Some(Command::Start(command)) => start::handle_start(command, cli.root),
        Some(Command::Delete(command)) => delete::handle_delete(command, cli.root),
        Some(Command::Done(command)) => done::handle_done(command, cli.root),
        Some(Command::Edit(command)) => edit::handle_edit(command, cli.root),
        Some(Command::Show(command)) => show::handle_show(command, cli.root),
        Some(Command::Find(command)) => find::handle_find(command, cli.root),
        Some(Command::Triage(command)) => triage::handle_triage(command, cli.root),
        Some(Command::Config(command)) => config_cmd::handle_config(command, cli.root),
        Some(Command::Doctor(command)) => doctor::handle_doctor(command, cli.root),
        None => handle_default(cli.root),
    }
}

fn handle_default(root: Option<std::path::PathBuf>) -> Result<(), AppError> {
    let resolved = match config::resolve(root) {
        Ok(resolved) => resolved,
        Err(_) => {
            let inspection = config::inspect(None)?;
            output::print_getting_started(inspection.config_path.as_deref());
            return Ok(());
        }
    };

    let repo = helpers::repo_from_config(&resolved);
    let tasks = repo.list()?;

    if tasks.is_empty() {
        let inspection = config::inspect(None)?;
        output::print_getting_started(inspection.config_path.as_deref());
    } else {
        output::print_dashboard(&tasks);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::cli::args::Cli;
    use crate::test_support::LockedEnv;
    use tempfile::TempDir;

    #[test]
    fn handle_shows_getting_started_when_no_command_and_no_config() {
        let _env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        handle(Cli {
            root: None,
            command: None,
        })
        .expect("bare invocation should succeed with getting-started guide");
    }

    #[test]
    fn handle_shows_getting_started_when_no_command_and_no_tasks() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        env.set("TQS_ROOT", temp.path().as_os_str());

        handle(Cli {
            root: None,
            command: None,
        })
        .expect("bare invocation with empty repo should succeed");
    }
}
