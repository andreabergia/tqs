use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use super::app_state::{Mode, TuiApp};

/// Poll for a crossterm event, returning None on timeout.
pub fn poll_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

/// Map a key event to a state mutation on the app, given the current mode.
pub fn handle_key(app: &mut TuiApp, key: KeyEvent) {
    match app.mode {
        Mode::Normal => handle_normal_key(app, key),
    }
}

fn handle_normal_key(app: &mut TuiApp, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true
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

        _ => {}
    }
}
