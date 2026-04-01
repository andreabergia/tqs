use std::fmt;
use std::fs;
use std::path::PathBuf;

use chrono::{Local, Utc};

use crate::app::app_error::AppError;
use crate::domain::task::{Queue, Task};
use crate::storage::config::ResolvedConfig;
use crate::storage::{daily_notes, repo::TaskRepo};

#[derive(Debug, Default)]
pub struct TriageSummary {
    pub moved_now: u32,
    pub moved_next: u32,
    pub moved_later: u32,
    pub moved_done: u32,
    pub deleted: u32,
    pub skipped: u32,
}

impl TriageSummary {
    pub fn record_move(&mut self, queue: Queue) {
        match queue {
            Queue::Now => self.moved_now += 1,
            Queue::Next => self.moved_next += 1,
            Queue::Later => self.moved_later += 1,
            Queue::Done => self.moved_done += 1,
            Queue::Inbox => {}
        }
    }

    pub fn is_empty(&self) -> bool {
        self.moved_now == 0
            && self.moved_next == 0
            && self.moved_later == 0
            && self.moved_done == 0
            && self.deleted == 0
            && self.skipped == 0
    }
}

impl fmt::Display for TriageSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.moved_now > 0 {
            parts.push(format!("{} to now", self.moved_now));
        }
        if self.moved_next > 0 {
            parts.push(format!("{} to next", self.moved_next));
        }
        if self.moved_later > 0 {
            parts.push(format!("{} to later", self.moved_later));
        }
        if self.moved_done > 0 {
            parts.push(format!("{} done", self.moved_done));
        }
        if self.deleted > 0 {
            parts.push(format!("{} deleted", self.deleted));
        }
        if self.skipped > 0 {
            parts.push(format!("{} skipped", self.skipped));
        }
        if parts.is_empty() {
            write!(f, "No changes")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Move a task to the done queue and append to daily notes if configured.
/// Returns the updated task and its path.
pub fn mark_done(
    repo: &TaskRepo,
    config: &ResolvedConfig,
    task_id: &str,
) -> Result<(Task, PathBuf), AppError> {
    let (mut task, path, _) = repo.move_to_queue(task_id, Queue::Done, Utc::now())?;

    if let Some(daily_notes_dir) = &config.daily_notes_dir {
        let note_date = Local::now().date_naive();
        let note = daily_notes::append_completion(daily_notes_dir, &path, note_date, &task)?;
        if task.daily_note.as_deref() != Some(note.note_name.as_str()) {
            task.daily_note = Some(note.note_name);
            repo.update(&task)?;
        }
    }

    Ok((task, path))
}

/// Result of applying an edit: either the task was unchanged, or it was updated.
pub enum EditOutcome {
    Unchanged,
    Applied,
}

/// Validate and apply an edited task file. Reads the file at `path`, checks for
/// empty content, compares with the original, and runs `replace_edited` with
/// full YAML validation. On any validation error, restores the original content.
///
/// The caller must pass the task's file path directly — this function cannot use
/// `find_by_id` because the file may contain malformed YAML at this point, and
/// `find_by_id` would skip it during scanning.
pub fn apply_edit(
    repo: &TaskRepo,
    task_id: &str,
    path: &std::path::Path,
    original_content: &str,
) -> Result<EditOutcome, AppError> {
    let edited_content = fs::read_to_string(path)?;

    if edited_content.trim().is_empty() {
        fs::write(path, original_content)?;
        return Err(AppError::message("task file cannot be empty"));
    }

    if edited_content == original_content {
        return Ok(EditOutcome::Unchanged);
    }

    // Restore the original content before calling replace_edited, because
    // replace_edited uses find_by_id internally which scans all files on disk.
    // If the file still has malformed YAML, find_by_id would skip it and
    // return not-found. Restoring first ensures the task is visible to the
    // repo. If replace_edited succeeds, it overwrites with the normalized
    // version anyway.
    fs::write(path, original_content)?;
    match repo.replace_edited(task_id, &edited_content, Utc::now()) {
        Ok(_) => Ok(EditOutcome::Applied),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::config::QueueDirs;
    use tempfile::TempDir;

    fn make_repo_with_task(temp: &TempDir) -> (TaskRepo, PathBuf, String) {
        let root = temp.path().to_path_buf();
        let repo = TaskRepo::new(root, QueueDirs::default());
        let task = Task::new("abc".to_string(), "Test task", Utc::now());
        let path = repo.create(&task).unwrap();
        let original = fs::read_to_string(&path).unwrap();
        (repo, path, original)
    }

    #[test]
    fn apply_edit_unchanged_returns_unchanged() {
        let temp = TempDir::new().unwrap();
        let (repo, path, original) = make_repo_with_task(&temp);

        let result = apply_edit(&repo, "abc", &path, &original).unwrap();
        assert!(matches!(result, EditOutcome::Unchanged));
    }

    #[test]
    fn apply_edit_valid_change_returns_applied() {
        let temp = TempDir::new().unwrap();
        let (repo, path, original) = make_repo_with_task(&temp);

        // Write a valid edit: change the body
        let edited = original.replace("# Test task", "# Test task\n\nNew body content");
        fs::write(&path, &edited).unwrap();

        let result = apply_edit(&repo, "abc", &path, &original).unwrap();
        assert!(matches!(result, EditOutcome::Applied));
    }

    #[test]
    fn apply_edit_empty_file_restores_original() {
        let temp = TempDir::new().unwrap();
        let (repo, path, original) = make_repo_with_task(&temp);

        fs::write(&path, "   \n").unwrap();

        let result = apply_edit(&repo, "abc", &path, &original);
        assert!(result.is_err());

        // Original content should be restored
        let on_disk = fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, original);
    }

    #[test]
    fn apply_edit_malformed_yaml_restores_original_and_task_survives() {
        let temp = TempDir::new().unwrap();
        let (repo, path, original) = make_repo_with_task(&temp);

        // Write malformed YAML to disk (as if the user saved garbage in $EDITOR)
        fs::write(&path, "---\nthis is not: [valid: yaml\n---\n").unwrap();

        let result = apply_edit(&repo, "abc", &path, &original);
        assert!(result.is_err());

        // Original content should be restored on disk
        let on_disk = fs::read_to_string(&path).unwrap();
        assert_eq!(on_disk, original);

        // Task should still be findable by the repo
        let task = repo.read("abc").unwrap();
        assert_eq!(task.title, "Test task");
    }
}
