use clap::Parser;

use crate::app::app_error::AppError;
use crate::domain::filter::{ListMode, matches_list_mode};
use crate::io::output;
use crate::io::picker;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Reopen {
    pub id: Option<String>,
}

pub fn handle_reopen(
    Reopen { id }: Reopen,
    root: Option<std::path::PathBuf>,
) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);

    let id = match id {
        Some(id) => id,
        None => {
            let tasks = repo.list()?;
            if tasks.is_empty() {
                output::print_info("No tasks available");
                return Ok(());
            }

            if !tasks
                .iter()
                .any(|task| matches_list_mode(task, ListMode::Closed))
            {
                output::print_info("No closed tasks available");
                return Ok(());
            }

            let allowed_modes = [ListMode::Closed, ListMode::All];
            let options = picker::TaskPickerOptions {
                prompt: "Select task to reopen",
                default_mode: ListMode::Closed,
                allowed_modes: &allowed_modes,
            };

            match picker::pick_task(&tasks, options)? {
                Some(id) => id,
                None => {
                    output::print_info("Operation cancelled");
                    return Ok(());
                }
            }
        }
    };

    let mut task = repo.read(&id)?;

    if !task.reopen() {
        output::print_info(&format!("Task {} is already open", id));
        return Ok(());
    }

    repo.update(&task)?;
    output::print_info(&format!("Reopened task: {}", id));
    Ok(())
}
