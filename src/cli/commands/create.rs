use std::path::PathBuf;

use clap::Parser;

use crate::app::app_error::AppError;
use crate::domain::id::IdGenerator;
use crate::domain::task::Task;
use crate::io::output;
use crate::storage::repo::TaskRepo;
use crate::storage::root;
use chrono::Utc;

#[derive(Debug, Parser)]
pub struct Create {
    #[arg(long)]
    pub description: Option<String>,

    pub summary: Option<String>,
}

pub fn handle_create(
    Create {
        summary,
        description,
    }: Create,
    root: Option<PathBuf>,
) -> Result<(), AppError> {
    let storage_root = root::resolve_root(root);
    let repo = TaskRepo::new(storage_root);
    let generator = IdGenerator::new(|id| repo.id_exists(id));

    let summary = match summary {
        Some(s) => s,
        None => {
            eprintln!("Error: summary is required");
            return Err(AppError::usage("missing summary"));
        }
    };

    let task = Task::new(generator.generate(), Utc::now(), summary, description);
    repo.create(&task)?;

    output::print_info(&format!("Created task: {}", task.id));
    Ok(())
}
