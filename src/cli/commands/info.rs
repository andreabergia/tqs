use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Info {
    pub id: Option<String>,
}

pub fn handle_info(Info { id }: Info, root: Option<PathBuf>) -> Result<(), AppError> {
    let id = id.ok_or_else(|| AppError::usage("id is required for info command"))?;

    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);

    let task = repo.read(&id)?;
    output::print_task_detail(&task);
    Ok(())
}
