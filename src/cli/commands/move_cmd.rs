use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct Move {
    pub task: Option<String>,

    #[arg(value_parser = helpers::parse_queue)]
    pub queue: Option<Queue>,
}

pub fn handle_move(
    Move { task, queue }: Move,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global)?;
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to move")? else {
        return Ok(());
    };

    let queue = queue.ok_or_else(|| AppError::usage("missing target queue"))?;
    if stored.task.queue == queue {
        output::print_info(&format!("Task {} is already in {}", stored.task.id, queue));
        return Ok(());
    }

    let (task, path, _) = repo.move_to_queue(&stored.task.id, queue, Utc::now())?;
    output::print_info(&format!("Moved task: {} ({})", task.id, path.display()));
    Ok(())
}
