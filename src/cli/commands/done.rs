use std::path::PathBuf;
use std::{fs, process::Command};

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::app::operations;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;
use crate::storage::repo::TaskRepo;

#[derive(Debug, Parser)]
#[command(about = "Mark a task as done")]
pub struct Done {
    pub task: Option<String>,

    #[arg(long)]
    pub no_edit: bool,
}

pub fn handle_done(Done { task, no_edit }: Done, root: Option<PathBuf>) -> Result<(), AppError> {
    let resolved = helpers::resolve_config(root)?;
    let repo = TaskRepo::new(resolved.tasks_root.clone(), resolved.queue_dirs.clone());
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to complete")? else {
        return Ok(());
    };

    if stored.task.queue == Queue::Done {
        output::print_info(&format!("Task {} is already done", stored.task.id));
        return Ok(());
    }

    let (task, path) = operations::mark_done(&repo, &resolved, &stored.task.id)?;

    if !no_edit {
        let original_content = fs::read_to_string(&path)?;
        let editor = helpers::resolve_editor()?;
        let status = Command::new(&editor.program)
            .args(&editor.args)
            .arg(&path)
            .status()?;
        if !status.success() {
            return Err(AppError::message("editor command failed"));
        }

        let edited_content = fs::read_to_string(&path)?;
        if edited_content.trim().is_empty() {
            fs::write(&path, original_content)?;
            return Err(AppError::message("task file cannot be empty"));
        }

        if edited_content != original_content {
            match repo.replace_edited(&task.id, &edited_content, Utc::now()) {
                Ok((task, path)) => {
                    output::print_info(&format!(
                        "Completed task: {} ({})",
                        task.id,
                        path.display()
                    ));
                }
                Err(error) => {
                    fs::write(&path, original_content)?;
                    return Err(error);
                }
            }
            return Ok(());
        }
    }

    output::print_info(&format!("Completed task: {} ({})", task.id, path.display()));

    Ok(())
}
