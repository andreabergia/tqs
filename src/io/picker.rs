use crate::app::app_error::AppError;
use crate::domain::task::Task;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

pub fn pick_task(tasks: &[Task], prompt: &str) -> Result<Option<String>, AppError> {
    if tasks.is_empty() {
        return Ok(None);
    }

    if !dialoguer::console::Term::stderr().is_term() {
        return Err(AppError::NoTty);
    }

    let items: Vec<String> = tasks
        .iter()
        .map(|task| format!("{} - {}", task.id, task.summary))
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(&items)
        .interact_opt()?;

    Ok(selection.and_then(|idx| tasks.get(idx).map(|t| t.id.clone())))
}
