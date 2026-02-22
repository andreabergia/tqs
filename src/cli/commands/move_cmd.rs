use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct Move {
    pub old_id: String,
    pub new_id: String,
}

pub fn handle_move(Move { old_id, new_id }: Move, root: Option<PathBuf>) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);
    repo.rename_task(&old_id, &new_id)?;
    output::print_info(&format!("Moved task: {old_id} -> {new_id}"));
    Ok(())
}
