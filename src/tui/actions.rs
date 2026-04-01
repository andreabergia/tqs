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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::config::{QueueDirs, ResolvedConfig};
    use crate::storage::repo::TaskRepo;
    use chrono::Utc;
    use tempfile::TempDir;

    fn make_app_with_tasks(temp: &TempDir, tasks: &[(&str, Queue)]) -> TuiApp {
        let root = temp.path().to_path_buf();
        let config = ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.clone(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        };
        let repo = TaskRepo::new(root.clone(), QueueDirs::default());
        for (id, queue) in tasks {
            let mut task = Task::new(id.to_string(), &format!("Task {id}"), Utc::now());
            task.queue = *queue;
            repo.create(&task).unwrap();
        }
        TuiApp::new(config, repo).unwrap()
    }

    // --- mark_done ---

    #[test]
    fn mark_done_moves_task_to_done_queue() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        mark_done(&mut app).unwrap();
        let task = app.repo.read("a1").unwrap();
        assert_eq!(task.queue, Queue::Done);
    }

    #[test]
    fn mark_done_noop_when_no_selection() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox)]);
        // Sidebar is on Now which is empty, so no selection
        let result = mark_done(&mut app).unwrap();
        assert!(matches!(result, SideEffect::None));
    }

    #[test]
    fn mark_done_noop_when_already_done() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Done)]);
        app.jump_to_queue(Queue::Done);
        mark_done(&mut app).unwrap();
        assert!(
            app.active_status_message()
                .unwrap()
                .contains("already done")
        );
    }

    // --- start_task ---

    #[test]
    fn start_task_moves_to_now() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox)]);
        app.jump_to_queue(Queue::Inbox);
        start_task(&mut app).unwrap();
        let task = app.repo.read("a1").unwrap();
        assert_eq!(task.queue, Queue::Now);
    }

    // --- move_to_queue ---

    #[test]
    fn move_to_queue_changes_queue() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        move_to_queue(&mut app, Queue::Later).unwrap();
        let task = app.repo.read("a1").unwrap();
        assert_eq!(task.queue, Queue::Later);
    }

    #[test]
    fn move_to_queue_noop_when_same_queue() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        move_to_queue(&mut app, Queue::Now).unwrap();
        assert!(app.active_status_message().unwrap().contains("already in"));
    }

    // --- confirm_delete ---

    #[test]
    fn confirm_delete_removes_task() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        app.mode = Mode::ConfirmDelete {
            task_id: "a1".to_string(),
            from_triage: false,
        };
        confirm_delete(&mut app).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.repo.read("a1").is_err());
    }

    #[test]
    fn confirm_delete_noop_in_wrong_mode() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Now)]);
        // Mode is Normal, not ConfirmDelete
        let result = confirm_delete(&mut app).unwrap();
        assert!(matches!(result, SideEffect::None));
    }

    #[test]
    fn confirm_delete_from_triage_advances() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("a2", Queue::Inbox)]);
        app.enter_triage();
        app.mode = Mode::ConfirmDelete {
            task_id: "a1".to_string(),
            from_triage: true,
        };
        confirm_delete(&mut app).unwrap();
        assert_eq!(app.triage.summary.deleted, 1);
        // Should have advanced; triage index is now 1
        assert_eq!(app.triage.index, 1);
    }

    // --- triage_move ---

    #[test]
    fn triage_move_to_now() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("a2", Queue::Inbox)]);
        app.enter_triage();
        let first_id = app.triage.task_ids[0].clone();
        triage_move(&mut app, Queue::Now).unwrap();
        let task = app.repo.read(&first_id).unwrap();
        assert_eq!(task.queue, Queue::Now);
        assert_eq!(app.triage.summary.moved_now, 1);
    }

    #[test]
    fn triage_move_to_done_uses_mark_done() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox)]);
        app.enter_triage();
        triage_move(&mut app, Queue::Done).unwrap();
        let task = app.repo.read("a1").unwrap();
        assert_eq!(task.queue, Queue::Done);
        assert_eq!(app.triage.summary.moved_done, 1);
    }

    // --- triage_skip ---

    #[test]
    fn triage_skip_increments_skipped_and_advances() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox), ("a2", Queue::Inbox)]);
        app.enter_triage();
        triage_skip(&mut app).unwrap();
        assert_eq!(app.triage.summary.skipped, 1);
        assert_eq!(app.triage.index, 1);
    }

    // --- triage_edit ---

    #[test]
    fn triage_edit_returns_suspend_side_effect() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[("a1", Queue::Inbox)]);
        app.enter_triage();
        let result = triage_edit(&mut app).unwrap();
        assert!(matches!(result, SideEffect::SuspendForEditor { task_id } if task_id == "a1"));
    }

    // --- submit_add_form ---

    #[test]
    fn submit_add_form_creates_task() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[]);
        app.mode = Mode::AddForm {
            title: "New task".to_string(),
            queue: Queue::Now,
        };
        submit_add_form(&mut app).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
        let tasks: Vec<_> = app.tasks.iter().filter(|t| t.queue == Queue::Now).collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "New task");
    }

    #[test]
    fn submit_add_form_empty_title_cancels() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[]);
        app.mode = Mode::AddForm {
            title: "   ".to_string(),
            queue: Queue::Inbox,
        };
        submit_add_form(&mut app).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.tasks.is_empty());
    }

    #[test]
    fn submit_add_form_noop_in_wrong_mode() {
        let temp = TempDir::new().unwrap();
        let mut app = make_app_with_tasks(&temp, &[]);
        // Mode is Normal
        let result = submit_add_form(&mut app).unwrap();
        assert!(matches!(result, SideEffect::None));
        assert!(app.tasks.is_empty());
    }
}
