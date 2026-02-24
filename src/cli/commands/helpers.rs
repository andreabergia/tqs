use std::path::PathBuf;

use crate::app::app_error::AppError;
use crate::domain::filter::{ListMode, matches_list_mode};
use crate::domain::id::validate_user_id;
use crate::io::output;
use crate::io::picker;
use crate::storage::repo::TaskRepo;
use crate::storage::root;

pub fn parse_editor_command() -> Result<(String, Vec<String>), AppError> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut parts = shell_words::split(&editor)
        .map_err(|e| AppError::message(format!("invalid editor command '{}': {}", editor, e)))?;

    if parts.is_empty() {
        return Err(AppError::message("editor command is empty"));
    }

    let program = parts.remove(0);
    Ok((program, parts))
}

pub struct PickerConfig<'a> {
    pub prompt: &'a str,
    pub default_mode: ListMode,
    pub allowed_modes: &'a [ListMode],
    pub empty_message: &'a str,
    pub cancel_message: &'a str,
    pub status_check: Option<ListMode>,
    pub status_check_message: Option<&'a str>,
}

pub fn resolve_repo(root: Option<PathBuf>, global: bool) -> TaskRepo {
    let storage_root = root::resolve_root(root, global);
    TaskRepo::new(storage_root)
}

pub fn resolve_id(
    id: Option<String>,
    repo: &TaskRepo,
    config: PickerConfig<'_>,
) -> Result<Option<String>, AppError> {
    let id = match id {
        Some(id) => {
            validate_user_id(&id)?;
            return Ok(Some(id));
        }
        None => {
            let tasks = repo.list()?;
            if tasks.is_empty() {
                output::print_info(config.empty_message);
                return Ok(None);
            }

            if let Some(check_mode) = config.status_check
                && !tasks.iter().any(|task| matches_list_mode(task, check_mode))
            {
                output::print_info(config.status_check_message.unwrap_or(config.empty_message));
                return Ok(None);
            }

            let options = picker::TaskPickerOptions {
                prompt: config.prompt,
                default_mode: config.default_mode,
                allowed_modes: config.allowed_modes,
            };

            match picker::pick_task(&tasks, options)? {
                Some(id) => id,
                None => {
                    output::print_info(config.cancel_message);
                    return Ok(None);
                }
            }
        }
    };

    Ok(Some(id))
}
