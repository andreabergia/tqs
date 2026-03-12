use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;

#[derive(Debug, Parser)]
#[command(about = "List tasks")]
pub struct List {
    #[arg(value_parser = helpers::parse_queue)]
    pub queue: Option<Queue>,
}

pub enum QueueSelection {
    Inbox,
    Now,
}

pub fn handle_list(List { queue }: List, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;

    match queue {
        Some(queue) => print_resolved_queue(queue, &repo)?,
        None => {
            let tasks = repo.list()?;
            output::print_dashboard(&tasks);
        }
    }

    Ok(())
}

pub fn print_queue(selection: QueueSelection, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root)?;
    let queue = match selection {
        QueueSelection::Inbox => Queue::Inbox,
        QueueSelection::Now => Queue::Now,
    };
    print_resolved_queue(queue, &repo)
}

fn print_resolved_queue(
    queue: Queue,
    repo: &crate::storage::repo::TaskRepo,
) -> Result<(), AppError> {
    let tasks = repo.list_queue(queue)?;
    output::print_queue_tasks(queue, &tasks);
    Ok(())
}
