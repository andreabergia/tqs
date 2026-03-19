use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;

#[derive(Debug, Parser)]
#[command(about = "Move a task to the now queue")]
pub struct Start {
    pub task: Option<String>,
}

pub fn handle_start(Start { task }: Start, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to start")? else {
        return Ok(());
    };

    if stored.task.queue == Queue::Now {
        output::print_info(&format!("Task {} is already in now", stored.task.id));
        return Ok(());
    }

    let (task, path, _) = repo.move_to_queue(&stored.task.id, Queue::Now, Utc::now())?;
    output::print_info(&format!("Started task: {} ({})", task.id, path.display()));
    Ok(())
}
