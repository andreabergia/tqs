use chrono::Utc;

use crate::app::app_error::AppError;
use crate::app::operations;
use crate::domain::task::Queue;

use super::app_state::{Mode, TuiApp};

pub enum SideEffect {
    None,
    Quit,
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
