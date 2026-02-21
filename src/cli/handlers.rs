use crate::app::app_error::AppError;
use crate::domain::id::IdGenerator;
use crate::domain::task::Task;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;
use chrono::Utc;

use super::args::{Cli, Command};

pub fn handle(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Command::Create {
            summary,
            description,
        }) => handle_create(summary, description, cli.root),
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

fn handle_create(
    summary: Option<String>,
    description: Option<String>,
    root: Option<std::path::PathBuf>,
) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);
    let generator = IdGenerator::new(|id| repo.id_exists(id));

    let summary = match summary {
        Some(s) => s,
        None => {
            eprintln!("Error: summary is required");
            return Err(AppError::usage("missing summary"));
        }
    };

    let task = Task::new(generator.generate(), Utc::now(), summary, description);
    repo.create(&task)?;

    output::print_info(&format!("Created task: {}", task.id));
    Ok(())
}
