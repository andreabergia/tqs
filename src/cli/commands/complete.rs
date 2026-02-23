use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Complete {
    pub id: Option<String>,
}

pub fn handle_complete(
    Complete { id }: Complete,
    root: Option<std::path::PathBuf>,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root);

    let config = helpers::PickerConfig {
        prompt: "Select task to complete",
        default_mode: ListMode::Open,
        allowed_modes: &[ListMode::Open, ListMode::All],
        empty_message: "No tasks available",
        cancel_message: "Operation cancelled",
        status_check: Some(ListMode::Open),
        status_check_message: Some("No open tasks available"),
    };

    let Some(id) = helpers::resolve_id(id, &repo, config)? else {
        return Ok(());
    };

    let mut task = repo.read(&id)?;

    if !task.close() {
        output::print_info(&format!("Task {} is already closed", id));
        return Ok(());
    }

    repo.update(&task)?;
    output::print_info(&format!("Completed task: {}", id));
    Ok(())
}
