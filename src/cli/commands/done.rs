use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Done {
    pub task: Option<String>,
}

pub fn handle_done(
    Done { task }: Done,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global);
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to complete")? else {
        return Ok(());
    };

    if stored.task.queue == Queue::Done {
        output::print_info(&format!("Task {} is already done", stored.task.id));
        return Ok(());
    }

    let (task, path, _) = repo.move_to_queue(&stored.task.id, Queue::Done, Utc::now())?;
    output::print_info(&format!("Completed task: {} ({})", task.id, path.display()));
    Ok(())
}
