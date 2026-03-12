use std::path::PathBuf;

use chrono::{Local, Utc};
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::task::Queue;
use crate::io::output;
use crate::storage::{daily_notes, repo::TaskRepo};

#[derive(Debug, Parser)]
pub struct Done {
    pub task: Option<String>,
}

pub fn handle_done(Done { task }: Done, root: Option<PathBuf>) -> Result<(), AppError> {
    let resolved = helpers::resolve_config(root)?;
    let repo = TaskRepo::new(resolved.tasks_root, resolved.queue_dirs);
    let Some(stored) = helpers::resolve_task_ref(task, &repo, "Select task to complete")? else {
        return Ok(());
    };

    if stored.task.queue == Queue::Done {
        output::print_info(&format!("Task {} is already done", stored.task.id));
        return Ok(());
    }

    let (mut task, path, _) = repo.move_to_queue(&stored.task.id, Queue::Done, Utc::now())?;

    if let Some(daily_notes_dir) = resolved.daily_notes_dir {
        let note_date = Local::now().date_naive();
        let note = daily_notes::append_completion(&daily_notes_dir, &path, note_date, &task)?;
        if task.daily_note.as_deref() != Some(note.note_name.as_str()) {
            task.daily_note = Some(note.note_name);
            repo.update(&task)?;
        }
    }

    output::print_info(&format!("Completed task: {} ({})", task.id, path.display()));
    Ok(())
}
