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

    let task_id = match id {
        Some(provided_id) => {
            if repo.id_exists(&provided_id) {
                return Err(AppError::usage(&format!(
                    "id '{}' already exists",
                    provided_id
                )));
            }
            provided_id
        }
        None => {
            let generator = IdGenerator::new(|id| repo.id_exists(id));
            generator.generate()
        }
    };

    let (summary, description) = match (summary, description) {
        (Some(s), d) => (s, d),
        (None, Some(_)) => {
            return Err(AppError::usage("missing summary"));
        }
        (None, None) => {
            let summary = input::prompt_input("Summary:")?;
            let description = input::prompt_multiline("Description:")?;
            (summary, description)
        }
    };

    let task = Task::new(task_id, Utc::now(), summary, description);
    repo.create(&task)?;

    output::print_info(&format!("Created task: {}", task.id));
    Ok(())
}
