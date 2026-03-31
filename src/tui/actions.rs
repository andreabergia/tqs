use chrono::Utc;

use crate::app::app_error::AppError;
use crate::app::operations;
use crate::domain::task::{Queue, Task};
use crate::storage::id_state::SharedIdAllocator;

use super::app_state::{Mode, TuiApp};

pub enum SideEffect {
    None,
    Quit,
    SuspendForEditor { task_id: String },
}

pub fn mark_done(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    let Some(task) = app.selected_task() else {
        return Ok(SideEffect::None);
    };
    if task.queue == Queue::Done {
        app.set_status(format!("{} is already done", task.id));
        return Ok(SideEffect::None);
    }
    let task_id = task.id.clone();
    operations::mark_done(&app.repo, &app.config, &task_id)?;
    app.refresh()?;
    app.set_status(format!("Completed: {task_id}"));
    Ok(SideEffect::None)
}

pub fn start_task(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    move_to_queue(app, Queue::Now)
}

pub fn move_to_queue(app: &mut TuiApp, queue: Queue) -> Result<SideEffect, AppError> {
    let Some(task) = app.selected_task() else {
        return Ok(SideEffect::None);
    };
    if task.queue == queue {
        app.set_status(format!("{} is already in {queue}", task.id));
        return Ok(SideEffect::None);
    }
    let task_id = task.id.clone();
    app.repo.move_to_queue(&task_id, queue, Utc::now())?;
    app.refresh()?;
    app.set_status(format!("Moved {task_id} to {queue}"));
    Ok(SideEffect::None)
}

pub fn confirm_delete(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    let Mode::ConfirmDelete { task_id } = app.mode.clone() else {
        return Ok(SideEffect::None);
    };
    app.repo.delete(&task_id)?;
    app.mode = Mode::Normal;
    app.refresh()?;
    app.set_status(format!("Deleted: {task_id}"));
    Ok(SideEffect::None)
}

pub fn submit_add_form(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    let title = app.add_title.trim().to_string();
    if title.is_empty() {
        app.mode = Mode::Normal;
        return Ok(SideEffect::None);
    }

    let allocator = SharedIdAllocator::new(&app.config);
    let id = allocator.generate(&app.repo)?;
    let mut task = Task::new(id, &title, Utc::now());
    task.queue = app.add_queue;
    app.repo.create(&task)?;

    app.mode = Mode::Normal;
    app.add_title.clear();
    app.add_queue = Queue::Inbox;
    app.refresh()?;
    app.set_status(format!("Added: {title}"));
    Ok(SideEffect::None)
}
