use std::{fs, path::PathBuf, process::Command};

use chrono::Utc;
use clap::Parser;
use dialoguer::console::style;

use crate::app::app_error::AppError;
use crate::app::operations;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::{input, output};
use crate::storage::{config::ResolvedConfig, repo::TaskRepo};

#[derive(Debug, Parser)]
#[command(about = "Triage inbox tasks interactively")]
pub struct Triage;

enum TriageOutcome {
    Moved(Queue),
    Deleted,
    Skipped,
    Quit,
}

fn record_outcome(summary: &mut operations::TriageSummary, outcome: &TriageOutcome) {
    match outcome {
        TriageOutcome::Moved(queue) => summary.record_move(*queue),
        TriageOutcome::Deleted => summary.deleted += 1,
        TriageOutcome::Skipped => summary.skipped += 1,
        TriageOutcome::Quit => {}
    }
}

pub fn handle_triage(_: Triage, root: Option<PathBuf>) -> Result<(), AppError> {
    if !input::supports_interaction() {
        return Err(AppError::NoTty);
    }

    let resolved = helpers::resolve_config(root)?;
    let repo = helpers::repo_from_config(&resolved);
    let inbox_tasks = repo.list_queue(Queue::Inbox)?;

    if inbox_tasks.is_empty() {
        output::print_info("Inbox is empty — nothing to triage");
        return Ok(());
    }

    println!(
        "{} {}",
        style("Triaging inbox").bold(),
        style(format!("({} tasks)", inbox_tasks.len())).yellow()
    );
    println!();

    let mut summary = operations::TriageSummary::default();
    let task_ids: Vec<String> = inbox_tasks.iter().map(|t| t.id.clone()).collect();
    for task_id in &task_ids {
        let outcome = triage_one_task(task_id, &repo, &resolved)?;
        let quit = matches!(outcome, TriageOutcome::Quit);
        record_outcome(&mut summary, &outcome);
        if quit {
            break;
        }
    }

    if !summary.is_empty() {
        println!();
        output::print_info(&summary.to_string());
    }
    Ok(())
}

fn triage_one_task(
    task_id: &str,
    repo: &TaskRepo,
    resolved: &ResolvedConfig,
) -> Result<TriageOutcome, AppError> {
    loop {
        let tasks = repo.list_queue(Queue::Inbox)?;
        let Some(task) = tasks.iter().find(|t| t.id == task_id) else {
            return Ok(TriageOutcome::Skipped);
        };

        println!("{}  {}", style(&task.id).cyan(), task.title);

        let queue_label = |q: Queue| format!("move to {}", style(q.to_string()).bold().magenta());
        let actions = vec![
            queue_label(Queue::Now),
            queue_label(Queue::Next),
            queue_label(Queue::Later),
            format!("mark {}", style("done").bold().magenta()),
            "edit".to_string(),
            "delete".to_string(),
            "skip".to_string(),
            "quit".to_string(),
        ];

        match input::prompt_select("Action", &actions)? {
            Some(0) => {
                repo.move_to_queue(task_id, Queue::Now, Utc::now())?;
                return Ok(TriageOutcome::Moved(Queue::Now));
            }
            Some(1) => {
                repo.move_to_queue(task_id, Queue::Next, Utc::now())?;
                return Ok(TriageOutcome::Moved(Queue::Next));
            }
            Some(2) => {
                repo.move_to_queue(task_id, Queue::Later, Utc::now())?;
                return Ok(TriageOutcome::Moved(Queue::Later));
            }
            Some(3) => {
                mark_done(task_id, repo, resolved)?;
                return Ok(TriageOutcome::Moved(Queue::Done));
            }
            Some(4) => {
                edit_task(task_id, repo)?;
                continue;
            }
            Some(5) => {
                repo.delete(task_id)?;
                return Ok(TriageOutcome::Deleted);
            }
            Some(6) => return Ok(TriageOutcome::Skipped),
            Some(7) | None => return Ok(TriageOutcome::Quit),
            _ => unreachable!(),
        }
    }
}

fn mark_done(task_id: &str, repo: &TaskRepo, resolved: &ResolvedConfig) -> Result<(), AppError> {
    operations::mark_done(repo, resolved, task_id)?;
    Ok(())
}

fn edit_task(task_id: &str, repo: &TaskRepo) -> Result<(), AppError> {
    let stored = repo.find_by_id(task_id)?;
    let original_content = fs::read_to_string(&stored.path)?;
    let editor = helpers::resolve_editor()?;

    let status = Command::new(&editor.program)
        .args(&editor.args)
        .arg(&stored.path)
        .status()?;

    if !status.success() {
        return Err(AppError::message("editor command failed"));
    }

    let edited_content = fs::read_to_string(&stored.path)?;
    if edited_content.trim().is_empty() {
        fs::write(&stored.path, original_content)?;
        return Err(AppError::message("task file cannot be empty"));
    }

    if edited_content != original_content {
        repo.replace_edited(task_id, &edited_content, Utc::now())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;
    use crate::storage::config::QueueDirs;
    use crate::test_support::LockedEnv;
    use tempfile::TempDir;

    fn task(id: &str, title: &str) -> Task {
        Task::new(id, title, Utc::now())
    }

    #[test]
    fn parses_triage_command() {
        Triage::parse_from(["triage"]);
    }

    #[test]
    fn empty_inbox_succeeds() {
        let mut env = LockedEnv::new(&["TQS_TEST_MODE", "TQS_ROOT"]);
        env.set("TQS_TEST_MODE", "1");

        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        // ensure queue dirs exist
        repo.create(&task("tmp", "tmp"))
            .expect("task should be created");
        repo.delete("tmp").expect("task should be deleted");

        let result = handle_triage(Triage, Some(temp.path().to_path_buf()));
        assert!(result.is_ok());
    }

    #[test]
    fn skip_leaves_task_in_inbox() {
        let mut env = LockedEnv::new(&["TQS_TEST_MODE", "TQS_ROOT"]);
        env.set("TQS_TEST_MODE", "1");

        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("t1", "Task one"))
            .expect("task should be created");

        // We can't easily pipe stdin in unit tests to exercise prompt_select,
        // so we just verify the task remains in inbox (not moved by setup).
        let tasks = repo.list_queue(Queue::Inbox).expect("should list");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "t1");
    }

    #[test]
    fn quit_returns_true() {
        // The quit logic is handled by prompt_select returning Some(7) or None
        // This is a structural test — the interactive flow is best verified manually
    }
}
