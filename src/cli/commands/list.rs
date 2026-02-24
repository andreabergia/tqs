use clap::Parser;

use crate::app::app_error::AppError;
use crate::domain::filter::{ListMode, matches_keywords};
use crate::domain::task::{Task, TaskStatus};
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

#[derive(Debug, Parser)]
pub struct List {
    pub keywords: Vec<String>,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub closed: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn handle_list(
    List {
        keywords,
        all,
        closed,
        verbose,
    }: List,
    root: Option<std::path::PathBuf>,
    global: bool,
) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root, global);
    let repo = TaskRepo::new(storage_root);

    let list_mode = if closed {
        ListMode::Closed
    } else if all {
        ListMode::All
    } else {
        ListMode::Open
    };

    let tasks = match list_mode {
        ListMode::Open => repo.list_filtered(TaskStatus::Open)?,
        ListMode::Closed => repo.list_filtered(TaskStatus::Closed)?,
        ListMode::All => repo.list()?,
    };

    let filtered_tasks: Vec<Task> = tasks
        .into_iter()
        .filter(|task| matches_keywords(task, &keywords))
        .collect();

    if verbose {
        output::print_tasks_verbose(&filtered_tasks);
    } else {
        output::print_tasks_simple(&filtered_tasks);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::List;
    use clap::Parser;

    #[test]
    fn parses_list_command() {
        let list = List::parse_from(["list", "foo", "bar"]);
        assert_eq!(list.keywords, vec!["foo", "bar"]);
    }
}
