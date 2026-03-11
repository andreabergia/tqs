use std::{fs, path::PathBuf, process::Command};

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Edit {
    pub task: Option<String>,
}

pub fn handle_edit(
    Edit { task }: Edit,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global)?;
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

    let edited_content = fs::read_to_string(&stored.path)?;
    if edited_content.trim().is_empty() {
        fs::write(&stored.path, original_content)?;
        return Err(AppError::message("task file cannot be empty"));
    }

    if edited_content == original_content {
        output::print_info(&format!(
            "No changes made: {} ({})",
            stored.task.id,
            stored.path.display()
        ));
        return Ok(());
    }

    match repo.replace_edited(&stored.task.id, &edited_content, Utc::now()) {
        Ok((task, path)) => {
            output::print_info(&format!("Edited task: {} ({})", task.id, path.display()));
            Ok(())
        }
        Err(error) => {
            fs::write(&stored.path, original_content)?;
            Err(error)
        }
    }
}
