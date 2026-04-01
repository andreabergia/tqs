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
    let (task_id, from_triage) = match &app.mode {
        Mode::ConfirmDelete {
            task_id,
            from_triage,
        } => (task_id.clone(), *from_triage),
        _ => return Ok(SideEffect::None),
    };

    if from_triage {
        app.repo.delete(&task_id)?;
        app.triage.summary.deleted += 1;
        app.refresh()?;
        app.mode = Mode::Triage;
        app.advance_triage_or_finish();
    } else {
        app.repo.delete(&task_id)?;
        app.mode = Mode::Normal;
        app.refresh()?;
        app.set_status(format!("Deleted: {task_id}"));
    }
    Ok(SideEffect::None)
}

pub fn triage_move(app: &mut TuiApp, queue: Queue) -> Result<SideEffect, AppError> {
    let Some(task) = app.current_triage_task() else {
        return Ok(SideEffect::None);
    };
    let task_id = task.id.clone();

    if queue == Queue::Done {
        operations::mark_done(&app.repo, &app.config, &task_id)?;
        app.triage.summary.record_move(Queue::Done);
    } else {
        app.repo.move_to_queue(&task_id, queue, Utc::now())?;
        app.triage.summary.record_move(queue);
    }

    app.refresh()?;
    app.advance_triage_or_finish();
    Ok(SideEffect::None)
}

pub fn triage_skip(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    app.triage.summary.skipped += 1;
    app.advance_triage_or_finish();
    Ok(SideEffect::None)
}

pub fn triage_edit(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    let Some(task) = app.current_triage_task() else {
        return Ok(SideEffect::None);
    };
    Ok(SideEffect::SuspendForEditor {
        task_id: task.id.clone(),
    })
}

pub fn submit_add_form(app: &mut TuiApp) -> Result<SideEffect, AppError> {
    let (title, queue) = match &app.mode {
        Mode::AddForm { title, queue } => (title.trim().to_string(), *queue),
        _ => return Ok(SideEffect::None),
    };

    if title.is_empty() {
        app.mode = Mode::Normal;
        return Ok(SideEffect::None);
    }

    let allocator = SharedIdAllocator::new(&app.config);
    let id = allocator.generate(&app.repo)?;
    let mut task = Task::new(id, &title, Utc::now());
    task.queue = queue;
    app.repo.create(&task)?;

    app.mode = Mode::Normal;
    app.refresh()?;
    app.set_status(format!("Added: {title}"));
    Ok(SideEffect::None)
}
