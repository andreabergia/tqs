use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Reopen {
    pub id: Option<String>,
}

pub fn handle_reopen(
    Reopen { id }: Reopen,
    root: Option<std::path::PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global);

    let config = helpers::PickerConfig {
        prompt: "Select task to reopen",
        default_mode: ListMode::Closed,
        allowed_modes: &[ListMode::Closed, ListMode::All],
        empty_message: "No tasks available",
        cancel_message: "Operation cancelled",
        status_check: Some(ListMode::Closed),
        status_check_message: Some("No closed tasks available"),
    };

    let Some(id) = helpers::resolve_id(id, &repo, config)? else {
        return Ok(());
    };

    let mut task = repo.read(&id)?;

    if !task.reopen() {
        output::print_info(&format!("Task {} is already open", id));
        return Ok(());
    }

    repo.update(&task)?;
    output::print_info(&format!("Reopened task: {}", id));
    Ok(())
}
