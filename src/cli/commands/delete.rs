use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Delete {
    pub id: String,
}

pub fn handle_delete(Delete { id }: Delete, root: Option<PathBuf>) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);
    repo.delete(&id)?;
    output::print_info(&format!("Deleted task: {id}"));
    Ok(())
}
