use std::fmt;
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
