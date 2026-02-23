use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Delete {
    pub id: Option<String>,
}

pub fn handle_delete(Delete { id }: Delete, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root);

    let config = helpers::PickerConfig {
        prompt: "Select task to delete",
        default_mode: ListMode::All,
        allowed_modes: &[ListMode::All, ListMode::Open, ListMode::Closed],
        empty_message: "No tasks available",
        cancel_message: "Operation cancelled",
        status_check: None,
        status_check_message: None,
    };

    let Some(id) = helpers::resolve_id(id, &repo, config)? else {
        return Ok(());
    };

    repo.delete(&id)?;
    output::print_info(&format!("Deleted task: {id}"));
    Ok(())
}
