use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::domain::id::IdGenerator;
use crate::domain::task::Task;
use crate::io::input;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;
use chrono::Utc;

#[derive(Debug, Parser)]
pub struct Create {
    #[arg(long)]
    pub id: Option<String>,

    #[arg(long)]
    pub description: Option<String>,

    pub summary: Option<String>,
}

pub fn handle_create(
    Create {
        id,
        summary,
        description,
    }: Create,
    root: Option<PathBuf>,
) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);

    let (task_id, summary, description) = match (id, summary, description) {
        (Some(provided_id), Some(s), d) => {
            if repo.id_exists(&provided_id) {
                return Err(AppError::usage(format!(
                    "id '{}' already exists",
                    provided_id
                )));
            }
            (provided_id, s, d)
        }
        (None, Some(s), d) => {
            let generator = IdGenerator::new(|id| repo.id_exists(id));
            (generator.generate(), s, d)
        }
        (None, None, Some(_)) => {
            return Err(AppError::usage("missing summary"));
        }
        (None, None, None) => {
            let summary = input::prompt_input("Summary:")?;
            let user_id = input::prompt_input_optional("ID (blank to auto-generate):")?;
            let task_id = if user_id.is_empty() {
                let generator = IdGenerator::new(|id| repo.id_exists(id));
                generator.generate()
            } else {
                if repo.id_exists(&user_id) {
                    return Err(AppError::usage(format!("id '{}' already exists", user_id)));
                }
                user_id
            };
            let description = input::prompt_multiline("Description:")?;
            (task_id, summary, description)
        }
        (Some(_provided_id), None, Some(_)) => {
            return Err(AppError::usage("missing summary"));
        }
        (Some(provided_id), None, None) => {
            if repo.id_exists(&provided_id) {
                return Err(AppError::usage(format!(
                    "id '{}' already exists",
                    provided_id
                )));
            }
            let summary = input::prompt_input("Summary:")?;
            let description = input::prompt_multiline("Description:")?;
            (provided_id, summary, description)
        }
    };

    let task = Task::new(task_id, Utc::now(), summary, description);
    repo.create(&task)?;

    output::print_info(&format!("Created task: {}", task.id));
    Ok(())
}
