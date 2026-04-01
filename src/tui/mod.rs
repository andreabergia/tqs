mod actions;
mod app_state;
mod event;
mod ui;
mod widgets;

use std::io;
use std::io::Write as _;
use std::process::Command;
use std::time::Duration;

use chrono::Utc;
use crossterm::{
    event::Event,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::app_error::AppError;
use crate::storage::config::ResolvedConfig;
use crate::storage::editor::ResolvedEditor;
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
                SideEffect::SuspendForEditor { task_id } => {
                    suspend_for_editor(terminal, app, &task_id)?;
                }
            }
        }
    }
}

fn suspend_for_editor(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    task_id: &str,
) -> Result<(), AppError> {
    // Leave TUI mode
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    // Run editor
    let result = run_editor(app, task_id);

    // Restore TUI mode
    let _ = enable_raw_mode();
    let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);
    let _ = terminal.hide_cursor();
    terminal
        .clear()
        .map_err(|e| AppError::message(format!("failed to clear terminal: {e}")))?;

    // Refresh regardless of editor outcome
    app.refresh()?;

    match result {
        Ok(()) => app.set_status(format!("Edited: {task_id}")),
        Err(e) => app.set_status(format!("Edit failed: {e}")),
    }

    Ok(())
}

fn run_editor(app: &TuiApp, task_id: &str) -> Result<(), AppError> {
    let (original_content, path) = app.repo.read_raw(task_id)?;

    let editor = ResolvedEditor::resolve()?;
    // Flush stdout before spawning editor
    io::stdout().flush().ok();

    let status = Command::new(&editor.program)
        .args(&editor.args)
        .arg(&path)
        .status()?;

    if !status.success() {
        return Err(AppError::message("editor command failed"));
    }

    let (edited_content, _) = app.repo.read_raw(task_id)?;
    if edited_content.trim().is_empty() {
        app.repo.write_raw(task_id, &original_content)?;
        return Err(AppError::message("task file cannot be empty"));
    }

    if edited_content != original_content {
        app.repo
            .replace_edited(task_id, &edited_content, Utc::now())?;
    }

    Ok(())
}

fn poll_event() -> Result<Option<Event>, AppError> {
    event::poll_event(POLL_TIMEOUT).map_err(|e| AppError::message(format!("event error: {e}")))
}
