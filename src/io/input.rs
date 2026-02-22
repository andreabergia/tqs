use crate::app::app_error::AppError;
use dialoguer::{theme::ColorfulTheme, Input};

pub fn prompt_input(prompt: &str) -> Result<String, AppError> {
    if !dialoguer::console::Term::stderr().is_term() {
        return Err(AppError::NoTty);
    }

    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(AppError::from)
}

pub fn prompt_multiline(prompt: &str) -> Result<Option<String>, AppError> {
    if !dialoguer::console::Term::stderr().is_term() {
        return Err(AppError::NoTty);
    }

    eprintln!("{prompt}");

    let mut lines = Vec::new();
    for line in std::io::stdin().lines() {
        lines.push(line.map_err(AppError::Io)?);
    }

    let combined = lines.join("\n");
    let description = if combined.trim().is_empty() {
        None
    } else {
        Some(combined)
    };

    Ok(description)
}
