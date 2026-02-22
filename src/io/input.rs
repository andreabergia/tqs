use crate::app::app_error::AppError;
use dialoguer::{Input, theme::ColorfulTheme};
use std::io::Read;

fn has_tty() -> bool {
    dialoguer::console::Term::stderr().is_term()
}

fn is_test_mode() -> bool {
    std::env::var("TQS_TEST_MODE").is_ok()
}

pub fn prompt_input(prompt: &str) -> Result<String, AppError> {
    if !has_tty() && !is_test_mode() {
        return Err(AppError::NoTty);
    }

    if is_test_mode() {
        eprintln!("{prompt}");
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(AppError::Io)?;
        Ok(line.trim().to_string())
    } else {
        let theme = ColorfulTheme::default();
        Input::with_theme(&theme)
            .with_prompt(prompt)
            .interact()
            .map_err(AppError::from)
    }
}

pub fn prompt_multiline(prompt: &str) -> Result<Option<String>, AppError> {
    if !has_tty() && !is_test_mode() {
        return Err(AppError::NoTty);
    }

    eprintln!("{prompt}");

    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .map_err(AppError::Io)?;

    let description = if buffer.trim().is_empty() {
        None
    } else {
        Some(buffer.trim().to_string())
    };

    Ok(description)
}
