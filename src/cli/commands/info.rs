use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Info {
    pub id: Option<String>,
}

pub fn handle_info(Info { id }: Info, root: Option<PathBuf>, global: bool) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global);

    let config = helpers::PickerConfig {
        prompt: "Select task to view",
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

    let task = repo.read(&id)?;
    output::print_task_detail(&task);
    Ok(())
}
