use std::path::PathBuf;

use chrono::{Local, Utc};

use crate::app::app_error::AppError;
use crate::domain::task::{Queue, Task};
use crate::storage::config::ResolvedConfig;
use crate::storage::{daily_notes, repo::TaskRepo};

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
