mod actions;
mod app_state;
mod event;
mod ui;
mod widgets;

use std::io;
use std::time::Duration;

use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::app_error::AppError;
use crate::storage::config::ResolvedConfig;
use crate::storage::repo::TaskRepo;

use actions::SideEffect;
use app_state::TuiApp;

const POLL_TIMEOUT: Duration = Duration::from_millis(250);

pub fn run(config: ResolvedConfig, repo: TaskRepo) -> Result<(), AppError> {
    let mut app = TuiApp::new(config, repo)?;

    // Set up terminal
    enable_raw_mode().map_err(|e| AppError::message(format!("failed to enable raw mode: {e}")))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| AppError::message(format!("failed to enter alternate screen: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| AppError::message(format!("failed to create terminal: {e}")))?;

    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal (always, even on error)
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> Result<(), AppError> {
    loop {
        terminal
            .draw(|frame| ui::draw(frame, app))
            .map_err(|e| AppError::message(format!("failed to draw: {e}")))?;

        if let Some(Event::Key(key)) = poll_event()? {
            match event::handle_key(app, key)? {
                SideEffect::None => {}
                SideEffect::Quit => return Ok(()),
            }
        }
    }
}

fn poll_event() -> Result<Option<Event>, AppError> {
    event::poll_event(POLL_TIMEOUT).map_err(|e| AppError::message(format!("event error: {e}")))
}
