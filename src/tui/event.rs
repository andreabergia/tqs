use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::app::app_error::AppError;
use crate::domain::task::Queue;

use super::actions::{self, SideEffect};
use super::app_state::{Mode, TuiApp};

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
        Mode::ConfirmDelete { .. } => handle_confirm_delete_key(app, key),
        Mode::MoveTarget => handle_move_target_key(app, key),
    }
}

fn handle_normal_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(SideEffect::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(SideEffect::Quit);
        }

        // Task navigation
        KeyCode::Char('j') | KeyCode::Down => app.select_next_task(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev_task(),

        // Queue navigation
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => app.next_queue(),
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => app.prev_queue(),

        // Direct queue jump (1-5)
        KeyCode::Char(c @ '1'..='5') => {
            let index = (c as usize) - ('1' as usize);
            app.select_queue(index);
        }

        // Toggle detail pane
        KeyCode::Char('p') | KeyCode::Enter => {
            app.detail_visible = !app.detail_visible;
            app.detail_scroll = 0;
        }

        // Detail pane scrolling (Shift+J/K)
        KeyCode::Char('J') => {
            if app.detail_visible {
                app.detail_scroll = app.detail_scroll.saturating_add(1);
            }
        }
        KeyCode::Char('K') => {
            if app.detail_visible {
                app.detail_scroll = app.detail_scroll.saturating_sub(1);
            }
        }

        // Task actions
        KeyCode::Char('d') => return actions::mark_done(app),
        KeyCode::Char('s') => return actions::start_task(app),
        KeyCode::Char('m') => {
            if app.selected_task().is_some() {
                app.mode = Mode::MoveTarget;
            }
        }
        KeyCode::Char('x') => {
            if let Some(task) = app.selected_task() {
                app.mode = Mode::ConfirmDelete {
                    task_id: task.id.clone(),
                };
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

fn handle_confirm_delete_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => actions::confirm_delete(app),
        _ => {
            app.mode = Mode::Normal;
            Ok(SideEffect::None)
        }
    }
}

fn handle_move_target_key(app: &mut TuiApp, key: KeyEvent) -> Result<SideEffect, AppError> {
    let result = match key.code {
        KeyCode::Char('i') | KeyCode::Char('1') => actions::move_to_queue(app, Queue::Inbox),
        KeyCode::Char('n') | KeyCode::Char('2') => actions::move_to_queue(app, Queue::Now),
        KeyCode::Char('x') | KeyCode::Char('3') => actions::move_to_queue(app, Queue::Next),
        KeyCode::Char('l') | KeyCode::Char('4') => actions::move_to_queue(app, Queue::Later),
        _ => {
            app.mode = Mode::Normal;
            return Ok(SideEffect::None);
        }
    };
    app.mode = Mode::Normal;
    result
}
