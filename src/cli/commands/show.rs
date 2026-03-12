use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::io::output;

#[derive(Debug, Parser)]
#[command(about = "Show task details")]
pub struct Show {
    pub task: Option<String>,
}

pub fn handle_show(Show { task }: Show, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to show")? else {
        return Ok(());
    };

    output::print_task_detail(&stored.task, &stored.path);
    Ok(())
}
