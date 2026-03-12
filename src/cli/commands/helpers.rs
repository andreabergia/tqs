use std::{path::PathBuf, str::FromStr};

use crate::app::app_error::AppError;
use crate::domain::{filter::title_matches_query, task::Queue};
use crate::io::{input, output, picker};
use crate::storage::{
    config, config::ResolvedConfig, editor::ResolvedEditor, repo::StoredTask, repo::TaskRepo,
};

pub fn resolve_editor() -> Result<ResolvedEditor, AppError> {
    ResolvedEditor::resolve()
}

pub fn resolve_repo(root: Option<PathBuf>) -> Result<TaskRepo, AppError> {
    let resolved = resolve_config(root)?;
    Ok(repo_from_config(&resolved))
}

pub fn resolve_config(root: Option<PathBuf>) -> Result<ResolvedConfig, AppError> {
    config::resolve(root)
}

pub fn repo_from_config(resolved: &ResolvedConfig) -> TaskRepo {
    TaskRepo::new(resolved.tasks_root.clone(), resolved.queue_dirs.clone())
}

pub fn parse_queue(value: &str) -> Result<Queue, String> {
    Queue::from_str(value).map_err(|_| {
        format!(
            "invalid queue '{}'; expected one of: {}",
            value,
            Queue::all()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

pub fn resolve_task_ref(
    query: Option<String>,
    repo: &TaskRepo,
    prompt: &str,
) -> Result<Option<StoredTask>, AppError> {
    let tasks = repo.scan_all()?;
    if tasks.is_empty() {
        output::print_info("No tasks available");
        return Ok(None);
    }

    match query {
        Some(query) => resolve_query_against_tasks(query, tasks, prompt),
        None => pick_from(tasks, prompt, None),
    }
}

pub fn resolve_target_queue(
    current: Queue,
    queue: Option<Queue>,
) -> Result<Option<Queue>, AppError> {
    match queue {
        Some(queue) => Ok(Some(queue)),
        None => pick_queue(current),
    }
}

fn pick_queue(current: Queue) -> Result<Option<Queue>, AppError> {
    let options = Queue::all()
        .iter()
        .copied()
        .filter(|queue| *queue != current)
        .collect::<Vec<_>>();
    let labels = options.iter().map(ToString::to_string).collect::<Vec<_>>();

    match input::prompt_select("Select target queue", &labels)? {
        Some(index) => Ok(options.get(index).copied()),
        None => {
            output::print_info("Operation cancelled");
            Ok(None)
        }
    }
}

fn resolve_query_against_tasks(
    query: String,
    tasks: Vec<StoredTask>,
    prompt: &str,
) -> Result<Option<StoredTask>, AppError> {
    if let Some(task) = unique_match(tasks.iter().filter(|stored| stored.task.id == query)) {
        return Ok(Some(task.clone()));
    }

    let prefix_matches = tasks
        .iter()
        .filter(|stored| stored.task.id.starts_with(&query))
        .cloned()
        .collect::<Vec<_>>();
    if prefix_matches.len() == 1 {
        return Ok(prefix_matches.into_iter().next());
    }

    let title_matches = tasks
        .iter()
        .filter(|stored| title_matches_query(&stored.task, &query))
        .cloned()
        .collect::<Vec<_>>();
    if title_matches.len() == 1 {
        return Ok(title_matches.into_iter().next());
    }

    let ambiguous = if !prefix_matches.is_empty() {
        prefix_matches
    } else {
        title_matches
    };

    if ambiguous.is_empty() {
        return Err(AppError::not_found(query));
    }

    pick_from(ambiguous, prompt, Some(&query))
}

fn pick_from(
    tasks: Vec<StoredTask>,
    prompt: &str,
    ambiguous_query: Option<&str>,
) -> Result<Option<StoredTask>, AppError> {
    let selection = picker::pick_task(&tasks, picker::TaskPickerOptions { prompt });
    let selection = match (selection, ambiguous_query) {
        (Err(AppError::NoTty), Some(query)) => return Err(AppError::ambiguous_task_ref(query)),
        (result, _) => result?,
    };

    match selection {
        Some(index) => Ok(tasks.get(index).cloned()),
        None if ambiguous_query.is_some() => Err(AppError::ambiguous_task_ref(
            ambiguous_query.expect("query should exist"),
        )),
        None => {
            output::print_info("Operation cancelled");
            Ok(None)
        }
    }
}

fn unique_match<'a>(mut matches: impl Iterator<Item = &'a StoredTask>) -> Option<&'a StoredTask> {
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    } else {
        Some(first)
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_target_queue, resolve_task_ref};
    use crate::app::app_error::AppError;
    use crate::domain::task::{Queue, Task};
    use crate::storage::{config::QueueDirs, repo::TaskRepo};
    use chrono::Utc;
    use tempfile::TempDir;

    fn task(id: &str, title: &str, queue: Queue) -> Task {
        let mut task = Task::new(id, title, Utc::now());
        task.queue = queue;
        task
    }

    #[test]
    fn resolve_task_ref_returns_none_for_empty_repo() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());

        let resolved = resolve_task_ref(Some("task-1".to_string()), &repo, "Select task")
            .expect("empty repo should not fail");

        assert!(resolved.is_none());
    }

    #[test]
    fn resolve_task_ref_prefers_exact_id_over_title_match() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("ship-v2", "Review release plan", Queue::Inbox))
            .expect("exact-id task should be created");
        repo.create(&task("task-2", "ship-v2", Queue::Inbox))
            .expect("title-match task should be created");

        let resolved = resolve_task_ref(Some("ship-v2".to_string()), &repo, "Select task")
            .expect("query should resolve")
            .expect("task should be found");

        assert_eq!(resolved.task.id, "ship-v2");
    }

    #[test]
    fn resolve_task_ref_supports_unique_id_prefixes() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("task-1234", "Ship v2", Queue::Inbox))
            .expect("task should be created");
        repo.create(&task("other-1", "Review docs", Queue::Inbox))
            .expect("other task should be created");

        let resolved = resolve_task_ref(Some("task-12".to_string()), &repo, "Select task")
            .expect("prefix query should resolve")
            .expect("task should be found");

        assert_eq!(resolved.task.id, "task-1234");
    }

    #[test]
    fn resolve_task_ref_returns_ambiguous_error_without_tty() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("task-1234", "Ship v2", Queue::Inbox))
            .expect("first task should be created");
        repo.create(&task("task-1235", "Ship docs", Queue::Inbox))
            .expect("second task should be created");

        let err = resolve_task_ref(Some("task-12".to_string()), &repo, "Select task")
            .expect_err("ambiguous query should fail without a tty");

        assert!(matches!(err, AppError::AmbiguousTaskRef { .. }));
    }

    #[test]
    fn resolve_task_ref_returns_no_tty_when_picker_is_required() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("task-1", "Ship v2", Queue::Inbox))
            .expect("task should be created");

        let err = resolve_task_ref(None, &repo, "Select task")
            .expect_err("picker should fail without a tty");

        assert!(matches!(err, AppError::NoTty));
    }

    #[test]
    fn resolve_target_queue_returns_supplied_queue() {
        let resolved = resolve_target_queue(Queue::Inbox, Some(Queue::Now))
            .expect("explicit queue should resolve");

        assert_eq!(resolved, Some(Queue::Now));
    }

    #[test]
    fn resolve_target_queue_returns_no_tty_without_interaction() {
        let err = resolve_target_queue(Queue::Inbox, None)
            .expect_err("missing queue without tty should fail");

        assert!(matches!(err, AppError::NoTty));
    }
}
