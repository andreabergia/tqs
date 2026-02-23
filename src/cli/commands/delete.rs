use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::domain::filter::ListMode;
use crate::io::output;
use crate::io::picker;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Delete {
    pub id: Option<String>,
}

pub fn handle_delete(Delete { id }: Delete, root: Option<PathBuf>) -> Result<(), AppError> {
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

            let allowed_modes = [ListMode::All, ListMode::Open, ListMode::Closed];
            let options = picker::TaskPickerOptions {
                prompt: "Select task to delete",
                default_mode: ListMode::All,
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

    repo.delete(&id)?;
    output::print_info(&format!("Deleted task: {id}"));
    Ok(())
}
