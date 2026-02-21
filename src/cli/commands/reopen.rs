use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
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
    let id = id.ok_or_else(|| AppError::usage("id is required for reopen command"))?;

    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);

    let mut task = repo.read(&id)?;

    if !task.reopen() {
        output::print_info(&format!("Task {} is already open", id));
        return Ok(());
    }

    repo.update(&task)?;
    output::print_info(&format!("Reopened task: {}", id));
    Ok(())
}
