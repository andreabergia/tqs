use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::io::picker;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Info {
    pub id: Option<String>,
}

pub fn handle_info(Info { id }: Info, root: Option<PathBuf>) -> Result<(), AppError> {
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

            match picker::pick_task(&tasks, "Select task to view")? {
                Some(id) => id,
                None => {
                    output::print_info("Operation cancelled");
                    return Ok(());
                }
            }
        }
    };

    let task = repo.read(&id)?;
    output::print_task_detail(&task);
    Ok(())
}
