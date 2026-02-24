use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::domain::id::validate_user_id;
use crate::io::input;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Move {
    pub old_id: Option<String>,
    pub new_id: Option<String>,
}

pub fn handle_move(
    Move { old_id, new_id }: Move,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global);

    let config = helpers::PickerConfig {
        prompt: "Select task to move",
        default_mode: ListMode::All,
        allowed_modes: &[ListMode::All, ListMode::Open, ListMode::Closed],
        empty_message: "No tasks available",
        cancel_message: "Operation cancelled",
        status_check: None,
        status_check_message: None,
    };

    let Some(old_id) = helpers::resolve_id(old_id, &repo, config)? else {
        return Ok(());
    };

    let new_id = match new_id {
        Some(id) => id,
        None => input::prompt_input("New task ID:")?,
    };

    validate_user_id(&new_id)?;

    if repo.id_exists(&new_id) {
        return Err(AppError::usage(format!("id '{}' already exists", new_id)));
    }

    repo.rename_task(&old_id, &new_id)?;
    output::print_info(&format!("Moved task: {old_id} -> {new_id}"));
    Ok(())
}
