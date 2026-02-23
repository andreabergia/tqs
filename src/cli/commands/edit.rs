use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::filter::ListMode;
use crate::io::output;
use crate::storage::format::{parse_task_markdown, render_task_markdown};
use clap::Parser;
use dialoguer::{Select, theme::ColorfulTheme};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Edit {
    pub id: Option<String>,
}

pub fn handle_edit(Edit { id }: Edit, root: Option<PathBuf>) -> Result<(), AppError> {
    let repo = helpers::resolve_repo(root);

    let config = helpers::PickerConfig {
        prompt: "Select task to edit",
        default_mode: ListMode::All,
        allowed_modes: &[ListMode::All, ListMode::Open, ListMode::Closed],
        empty_message: "No tasks available",
        cancel_message: "Operation cancelled",
        status_check: None,
        status_check_message: None,
    };

    let Some(id) = helpers::resolve_id(id, &repo, config)? else {
        return Ok(());
    };

    let task = repo.read(&id)?;
    let original_markdown = render_task_markdown(&task)?;
    let file_path = repo.task_path(&id);

    std::fs::write(&file_path, &original_markdown)?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let exit_status = std::process::Command::new(&editor)
        .arg(&file_path)
        .status()?;

    if !exit_status.success() {
        eprintln!(
            "Editor exited with non-zero status: {:?}",
            exit_status.code()
        );
        return Err(AppError::message(format!("editor '{}' failed", editor)));
    }

    let edited_content = std::fs::read_to_string(&file_path)?;

    if edited_content.trim().is_empty() {
        output::print_error("Task file is empty");
        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .items(["Restore original content", "Delete task", "Abort"])
            .default(0)
            .interact()?;

        match choice {
            0 => {
                std::fs::write(&file_path, &original_markdown)?;
                output::print_info("Restored original content");
                return Ok(());
            }
            1 => {
                repo.delete(&id)?;
                output::print_info("Task deleted due to empty file");
                return Ok(());
            }
            _ => {
                return Err(AppError::message("Task file is empty"));
            }
        }
    }

    match parse_task_markdown(&edited_content) {
        Ok(edited_task) => {
            if edited_task.id != id {
                output::print_error(&format!(
                    "ID in file ({}) does not match filename ({})",
                    edited_task.id, id
                ));
                let choice = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("What would you like to do?")
                    .items([
                        "Restore original ID in file",
                        "Rename file to match new ID",
                        "Abort",
                    ])
                    .default(0)
                    .interact()?;

                match choice {
                    0 => {
                        let mut restored_task = edited_task;
                        restored_task.id = id.clone();
                        let restored_markdown = render_task_markdown(&restored_task)?;
                        std::fs::write(&file_path, &restored_markdown)?;
                        output::print_info("Restored original ID in file");
                        Ok(())
                    }
                    1 => {
                        repo.rename_task(&id, &edited_task.id)?;
                        output::print_info(&format!("Renamed task: {} -> {}", id, edited_task.id));
                        Ok(())
                    }
                    _ => Err(AppError::message(format!(
                        "ID mismatch: file has '{}' but filename is '{}.md'",
                        edited_task.id, id
                    ))),
                }
            } else {
                output::print_info(&format!("Edited task: {id}"));
                Ok(())
            }
        }
        Err(e) => {
            output::print_error(&format!("Task file is invalid: {e}"));
            let choice = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What would you like to do?")
                .items(["Restore original content", "Abort"])
                .default(0)
                .interact()?;

            match choice {
                0 => {
                    std::fs::write(&file_path, &original_markdown)?;
                    output::print_info("Restored original content");
                    Ok(())
                }
                _ => Err(AppError::invalid_task_file(
                    file_path.to_string_lossy().to_string(),
                    e.to_string(),
                )),
            }
        }
    }
}
