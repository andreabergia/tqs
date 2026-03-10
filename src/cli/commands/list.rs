use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;

#[derive(Debug, Parser)]
pub struct List {
    #[arg(value_parser = helpers::parse_queue)]
    pub queue: Option<Queue>,
}

pub fn handle_list(
    List { queue }: List,
    root: Option<PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root, global)?;

    match queue {
        Some(queue) => {
            let tasks = repo.list_queue(queue)?;
            output::print_queue_tasks(queue, &tasks);
        }
        None => {
            let tasks = repo.list()?;
            output::print_dashboard(&tasks);
        }
    }

    Ok(())
}
