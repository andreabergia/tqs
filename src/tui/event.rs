use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::app::app_error::AppError;
use crate::domain::task::Queue;

use super::actions::{self, SideEffect};
use super::app_state::{FocusedPanel, Mode, TuiApp};

/// Poll for a crossterm event, returning None on timeout.
pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Map a key event to state mutations on the app, given the current mode.
/// Returns a SideEffect that the main loop may need to handle.
pub fn handle_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match &app.mode {
        Mode::Normal => handle_normal_key(app, key),
        Mode::AddForm { .. } => handle_add_form_key(app, key),
        Mode::ConfirmDelete { .. } => handle_confirm_delete_key(app, key),
        Mode::MoveTarget { .. } => handle_move_target_key(app, key),
        Mode::Search { .. } => handle_search_key(app, key),
        Mode::Triage => handle_triage_key(app, key),
    }
}

fn handle_normal_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(SideEffect::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(SideEffect::Quit);
        }

        // Panel focus navigation
        KeyCode::Char('h') | KeyCode::Left => {
            app.focused_panel = app.focused_panel.left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.focused_panel = app.focused_panel.right();
        }

        // Vertical navigation — depends on focused panel
        KeyCode::Char('j') | KeyCode::Down => match app.focused_panel {
            FocusedPanel::Sidebar => app.next_queue(),
            FocusedPanel::TaskList => app.select_next_task(),
            FocusedPanel::Detail => {
                app.detail_scroll = app.detail_scroll.saturating_add(1);
            }
        },
        KeyCode::Char('k') | KeyCode::Up => match app.focused_panel {
            FocusedPanel::Sidebar => app.prev_queue(),
            FocusedPanel::TaskList => app.select_prev_task(),
            FocusedPanel::Detail => {
                app.detail_scroll = app.detail_scroll.saturating_sub(1);
            }
        },

        // Tab cycles queues regardless of panel focus
        KeyCode::Tab => app.next_queue(),
        KeyCode::BackTab => app.prev_queue(),

        // Direct queue jump (1-6) regardless of panel focus
        KeyCode::Char(c @ '1'..='6') => {
            let index = (c as usize) - ('1' as usize);
            app.select_queue_by_index(index);
        }

        // Task actions
        KeyCode::Char('d') => return actions::mark_done(app),
        KeyCode::Char('s') => return actions::start_task(app),
        KeyCode::Char('m') => {
            if app.selected_task().is_some() {
                app.mode = Mode::MoveTarget { from_triage: false };
            }
        }
        KeyCode::Char('x') => {
            if let Some(task) = app.selected_task() {
                app.mode = Mode::ConfirmDelete {
                    task_id: task.id.clone(),
                    from_triage: false,
                };
            }
        }

        // Search
        KeyCode::Char('/') => {
            app.mode = Mode::Search {
                query: String::new(),
                results: Vec::new(),
                list_state: ListState::default(),
            };
            app.update_search_results();
        }

        // Triage
        KeyCode::Char('t') => app.enter_triage(),

        // Add task
        KeyCode::Char('a') => {
            app.mode = Mode::AddForm {
                title: String::new(),
                queue: Queue::Inbox,
            };
        }

        // Edit in $EDITOR
        KeyCode::Char('e') => {
            if let Some(task) = app.selected_task() {
                return Ok(SideEffect::SuspendForEditor {
                    task_id: task.id.clone(),
                });
            }
        }

        // Refresh
        KeyCode::Char('r') => {
            app.refresh()?;
            app.set_status("Refreshed");
        }

        _ => {}
    }
    Ok(SideEffect::None)
}

fn handle_add_form_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    use super::widgets::add_form;

    match key.code {
        KeyCode::Enter => return actions::submit_add_form(app),
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Tab => {
            if let Mode::AddForm { queue, .. } = &mut app.mode {
                *queue = add_form::cycle_queue(*queue);
            }
        }
        KeyCode::BackTab => {
            if let Mode::AddForm { queue, .. } = &mut app.mode {
                *queue = add_form::cycle_queue_back(*queue);
            }
        }
        KeyCode::Backspace => {
            if let Mode::AddForm { title, .. } = &mut app.mode {
                title.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::AddForm { title, .. } = &mut app.mode {
                title.push(c);
            }
        }
        _ => {}
    }
    Ok(SideEffect::None)
}

fn handle_confirm_delete_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => actions::confirm_delete(app),
        _ => {
            let from_triage = matches!(
                app.mode,
                Mode::ConfirmDelete {
                    from_triage: true,
                    ..
                }
            );
            app.mode = if from_triage {
                Mode::Triage
            } else {
                Mode::Normal
            };
            Ok(SideEffect::None)
        }
    }
}

fn handle_search_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            app.select_search_result();
        }
        KeyCode::Down | KeyCode::Tab => {
            if let Mode::Search {
                results,
                list_state,
                ..
            } = &mut app.mode
            {
                let count = results.len();
                if count > 0 {
                    let current = list_state.selected().unwrap_or(0);
                    let next = if current + 1 >= count { 0 } else { current + 1 };
                    list_state.select(Some(next));
                }
            }
        }
        KeyCode::Up | KeyCode::BackTab => {
            if let Mode::Search {
                results,
                list_state,
                ..
            } = &mut app.mode
            {
                let count = results.len();
                if count > 0 {
                    let current = list_state.selected().unwrap_or(0);
                    let prev = if current == 0 { count - 1 } else { current - 1 };
                    list_state.select(Some(prev));
                }
            }
        }
        KeyCode::Backspace => {
            if let Mode::Search { query, .. } = &mut app.mode {
                query.pop();
            }
            app.update_search_results();
        }
        KeyCode::Char(c) => {
            if let Mode::Search { query, .. } = &mut app.mode {
                query.push(c);
            }
            app.update_search_results();
        }
        _ => {}
    }
    Ok(SideEffect::None)
}

fn handle_triage_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        // Same keys as normal mode
        KeyCode::Char('d') => actions::triage_move(app, Queue::Done),
        KeyCode::Char('s') => actions::triage_move(app, Queue::Now),
        KeyCode::Char('m') => {
            if app.current_triage_task().is_some() {
                app.mode = Mode::MoveTarget { from_triage: true };
            }
            Ok(SideEffect::None)
        }
        KeyCode::Char('x') => {
            if let Some(task) = app.current_triage_task() {
                app.mode = Mode::ConfirmDelete {
                    task_id: task.id.clone(),
                    from_triage: true,
                };
            }
            Ok(SideEffect::None)
        }
        KeyCode::Char('e') => actions::triage_edit(app),

        // Triage-specific
        KeyCode::Char(' ') => actions::triage_skip(app),
        KeyCode::Char('q') | KeyCode::Esc => {
            let summary = app.triage.summary.to_string();
            app.mode = Mode::Normal;
            app.set_status(format!("Triage: {summary}"));
            Ok(SideEffect::None)
        }
        _ => Ok(SideEffect::None),
    }
}

fn handle_move_target_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    let from_triage = matches!(app.mode, Mode::MoveTarget { from_triage: true });
    let cancel_mode = if from_triage {
        Mode::Triage
    } else {
        Mode::Normal
    };

    match key.code {
        KeyCode::Char('i') | KeyCode::Char('1') => do_move(app, Queue::Inbox, from_triage),
        KeyCode::Char('n') | KeyCode::Char('2') => do_move(app, Queue::Now, from_triage),
        KeyCode::Char('x') | KeyCode::Char('3') => do_move(app, Queue::Next, from_triage),
        KeyCode::Char('l') | KeyCode::Char('4') => do_move(app, Queue::Later, from_triage),
        _ => {
            app.mode = cancel_mode;
            Ok(SideEffect::None)
        }
    }
}

fn do_move(app: &mut TuiApp, queue: Queue, from_triage: bool) -> Result<SideEffect, AppError> {
    if from_triage {
        app.mode = Mode::Triage;
        actions::triage_move(app, queue)
    } else {
        app.mode = Mode::Normal;
        actions::move_to_queue(app, queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;
    use crate::storage::config::{QueueDirs, ResolvedConfig};
    use crate::storage::repo::TaskRepo;
    use chrono::Utc;
    use tempfile::TempDir;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_app(temp: &TempDir) -> TuiApp {
        let root = temp.path().to_path_buf();
        let config = ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.clone(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        };
        let repo = TaskRepo::new(root, QueueDirs::default());
        TuiApp::new(config, repo).unwrap()
    }

    fn test_app_with_task(temp: &TempDir) -> TuiApp {
        let root = temp.path().to_path_buf();
        let config = ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.clone(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        };
        let repo = TaskRepo::new(root, QueueDirs::default());
        let mut task = Task::new("abc".to_string(), "Test task", Utc::now());
        task.queue = Queue::Now;
        repo.create(&task).unwrap();
        TuiApp::new(config, repo).unwrap()
    }

    #[test]
    fn q_quits() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        let result = handle_key(&mut app, key(KeyCode::Char('q'))).unwrap();
        assert!(matches!(result, SideEffect::Quit));
    }

    #[test]
    fn esc_quits() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        let result = handle_key(&mut app, key(KeyCode::Esc)).unwrap();
        assert!(matches!(result, SideEffect::Quit));
    }

    #[test]
    fn h_l_navigate_panels() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        app.focused_panel = FocusedPanel::TaskList;

        handle_key(&mut app, key(KeyCode::Char('h'))).unwrap();
        assert_eq!(app.focused_panel, FocusedPanel::Sidebar);

        handle_key(&mut app, key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.focused_panel, FocusedPanel::TaskList);

        handle_key(&mut app, key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.focused_panel, FocusedPanel::Detail);

        // Should not go past rightmost panel
        handle_key(&mut app, key(KeyCode::Char('l'))).unwrap();
        assert_eq!(app.focused_panel, FocusedPanel::Detail);
    }

    #[test]
    fn a_enters_add_form() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        handle_key(&mut app, key(KeyCode::Char('a'))).unwrap();
        assert!(matches!(app.mode, Mode::AddForm { .. }));
    }

    #[test]
    fn add_form_esc_cancels() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        app.mode = Mode::AddForm {
            title: "partial".to_string(),
            queue: Queue::Inbox,
        };

        handle_key(&mut app, key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn add_form_typing_appends_chars() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        app.mode = Mode::AddForm {
            title: String::new(),
            queue: Queue::Inbox,
        };

        handle_key(&mut app, key(KeyCode::Char('H'))).unwrap();
        handle_key(&mut app, key(KeyCode::Char('i'))).unwrap();
        assert!(matches!(
            &app.mode,
            Mode::AddForm { title, .. } if title == "Hi"
        ));

        handle_key(&mut app, key(KeyCode::Backspace)).unwrap();
        assert!(matches!(
            &app.mode,
            Mode::AddForm { title, .. } if title == "H"
        ));
    }

    #[test]
    fn slash_enters_search() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        handle_key(&mut app, key(KeyCode::Char('/'))).unwrap();
        assert!(matches!(app.mode, Mode::Search { .. }));
    }

    #[test]
    fn search_esc_returns_to_normal() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        app.mode = Mode::Search {
            query: String::new(),
            results: Vec::new(),
            list_state: ListState::default(),
        };
        handle_key(&mut app, key(KeyCode::Esc)).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn search_typing_updates_query() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        app.mode = Mode::Search {
            query: String::new(),
            results: Vec::new(),
            list_state: ListState::default(),
        };

        handle_key(&mut app, key(KeyCode::Char('T'))).unwrap();
        assert!(matches!(
            &app.mode,
            Mode::Search { query, results, .. } if query == "T" && !results.is_empty()
        ));
    }

    #[test]
    fn e_suspends_for_editor_when_task_selected() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        let result = handle_key(&mut app, key(KeyCode::Char('e'))).unwrap();
        assert!(matches!(result, SideEffect::SuspendForEditor { .. }));
    }

    #[test]
    fn x_enters_confirm_delete() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        handle_key(&mut app, key(KeyCode::Char('x'))).unwrap();
        assert!(matches!(app.mode, Mode::ConfirmDelete { .. }));
    }

    #[test]
    fn confirm_delete_cancel_returns_to_normal() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        app.mode = Mode::ConfirmDelete {
            task_id: "abc".to_string(),
            from_triage: false,
        };

        handle_key(&mut app, key(KeyCode::Char('n'))).unwrap();
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn m_enters_move_target() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        handle_key(&mut app, key(KeyCode::Char('m'))).unwrap();
        assert!(matches!(app.mode, Mode::MoveTarget { .. }));
    }

    #[test]
    fn move_target_n_moves_to_now() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app_with_task(&temp);
        app.mode = Mode::MoveTarget { from_triage: false };

        handle_key(&mut app, key(KeyCode::Char('n'))).unwrap();
        assert!(matches!(app.mode, Mode::Normal));

        // Task should have moved to now queue
        let tasks: Vec<_> = app.tasks.iter().filter(|t| t.queue == Queue::Now).collect();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn j_scrolls_detail_when_detail_focused() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        app.focused_panel = FocusedPanel::Detail;
        app.detail_scroll = 0;

        handle_key(&mut app, key(KeyCode::Char('j'))).unwrap();
        assert_eq!(app.detail_scroll, 1);

        handle_key(&mut app, key(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.detail_scroll, 0);

        // Should not underflow
        handle_key(&mut app, key(KeyCode::Char('k'))).unwrap();
        assert_eq!(app.detail_scroll, 0);
    }

    #[test]
    fn tab_cycles_queues() {
        let temp = TempDir::new().unwrap();
        let mut app = test_app(&temp);
        let initial = app.active_sidebar_index;

        handle_key(&mut app, key(KeyCode::Tab)).unwrap();
        assert_ne!(app.active_sidebar_index, initial);
    }
}
