use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::io::{input, output};

#[derive(Debug, Parser)]
#[command(about = "Delete a task permanently")]
pub struct Delete {
    pub task: Option<String>,

    /// Prompt for confirmation before deleting
    #[arg(long, short = 'i')]
    pub interactive: bool,
}

pub fn handle_delete(
    Delete { task, interactive }: Delete,
    root: Option<PathBuf>,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to delete")? else {
        return Ok(());
    };

    if interactive {
        let confirmed =
            input::prompt_confirm(&format!("Permanently delete '{}'?", stored.task.title))?;
        if !confirmed {
            output::print_info("Delete cancelled");
            return Ok(());
        }
    }

    let id = stored.task.id.clone();
    repo.delete(&id)?;
    output::print_info(&format!("Deleted task: {id}"));
    Ok(())
}
