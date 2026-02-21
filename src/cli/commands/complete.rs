use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Complete {
    pub id: Option<String>,
}

pub fn handle_complete(
    Complete { id }: Complete,
    root: Option<std::path::PathBuf>,
) -> Result<(), AppError> {
    let id = id.ok_or_else(|| AppError::usage("id is required for complete command"))?;

    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);

    let mut task = repo.read(&id)?;

    if !task.close() {
        output::print_info(&format!("Task {} is already closed", id));
        return Ok(());
    }

    repo.update(&task)?;
    output::print_info(&format!("Completed task: {}", id));
    Ok(())
}
