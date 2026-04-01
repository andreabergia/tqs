use std::{fs, path::PathBuf, process::Command};

use clap::Parser;

use crate::app::app_error::AppError;
use crate::app::operations::{self, EditOutcome};
use crate::cli::commands::helpers;
use crate::io::output;

#[derive(Debug, Parser)]
#[command(about = "Edit a task")]
pub struct Edit {
    pub task: Option<String>,
}

pub fn handle_edit(Edit { task }: Edit, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to edit")? else {
        return Ok(());
    };

    let original_content = fs::read_to_string(&stored.path)?;
    let editor = helpers::resolve_editor()?;
    let status = Command::new(&editor.program)
        .args(&editor.args)
        .arg(&stored.path)
        .status()?;
    if !status.success() {
        return Err(AppError::message("editor command failed"));
    }

    match operations::apply_edit(&repo, &stored.task.id, &stored.path, &original_content)? {
        EditOutcome::Unchanged => {
            output::print_info(&format!(
                "No changes made: {} ({})",
                stored.task.id,
                stored.path.display()
            ));
        }
        EditOutcome::Applied => {
            let updated = repo.find_by_id(&stored.task.id)?;
            output::print_info(&format!(
                "Edited task: {} ({})",
                updated.task.id,
                updated.path.display()
            ));
        }
    }
    Ok(())
}
