use std::{path::PathBuf, str::FromStr};

use crate::app::app_error::AppError;
use crate::domain::{filter::matches_query, task::Queue};
use crate::io::{output, picker};
use crate::storage::{repo::StoredTask, repo::TaskRepo, root};

pub fn parse_editor_command() -> Result<(String, Vec<String>), AppError> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut parts = shell_words::split(&editor).map_err(|error| {
        AppError::message(format!("invalid editor command '{}': {}", editor, error))
    })?;

    if parts.is_empty() {
        return Err(AppError::message("editor command is empty"));
    }

    let program = parts.remove(0);
    Ok((program, parts))
}

pub fn resolve_repo(root: Option<PathBuf>, global: bool) -> TaskRepo {
    TaskRepo::new(root::resolve_root(root, global))
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
        None => pick_from(tasks, prompt),
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
        .filter(|stored| matches_query(&stored.task, &query))
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

    pick_from(ambiguous, prompt)
}

fn pick_from(tasks: Vec<StoredTask>, prompt: &str) -> Result<Option<StoredTask>, AppError> {
    match picker::pick_task(&tasks, picker::TaskPickerOptions { prompt })? {
        Some(index) => Ok(tasks.get(index).cloned()),
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
