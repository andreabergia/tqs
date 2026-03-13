use std::{fs, path::PathBuf, process::Command};

use chrono::Utc;
use clap::Parser;

use crate::app::app_error::AppError;
use crate::cli::commands::helpers;
use crate::domain::{id::validate_user_id, task::Task};
use crate::{
    io::{input, output},
    storage::id_state::SharedIdAllocator,
};

#[derive(Debug, Parser)]
#[command(about = "Add a task")]
pub struct Add {
    pub title: Option<String>,

    #[arg(long, value_parser = helpers::parse_queue)]
    pub queue: Option<crate::domain::task::Queue>,

    #[arg(long)]
    pub no_edit: bool,

    #[arg(long)]
    pub content: Option<String>,

    #[arg(long, hide = true)]
    pub id: Option<String>,
}

pub fn handle_add(
    Add {
        title,
        queue,
        no_edit,
        content,
        id,
    }: Add,
    root: Option<PathBuf>,
) -> Result<(), AppError> {
    let resolved = helpers::resolve_config(root)?;
    let repo = helpers::repo_from_config(&resolved);
    let title = match title {
        Some(title) => title,
        None => input::prompt_input("Title:")?,
    };

    let task_id = match id {
        Some(id) => {
            validate_user_id(&id)?;
            if repo.id_exists(&id) {
                return Err(AppError::usage(format!("id '{}' already exists", id)));
            }
            id
        }
        None => SharedIdAllocator::new(&resolved).generate(&repo)?,
    };

    let now = Utc::now();
    let mut task = Task::new(task_id, title, now);

    if let Some(ref body) = content {
        task.body = format!("# {}\n\n{}\n", task.title, body);
    }

    if let Some(queue) = queue {
        task.move_to(queue, now);
        if task.queue != queue {
            task.queue = queue;
            task.normalize(now);
        }
    }

    let path = repo.create(&task)?;

    if !no_edit && content.is_none() {
        let original_content = fs::read_to_string(&path)?;
        let editor = helpers::resolve_editor()?;
        let status = Command::new(&editor.program)
            .args(&editor.args)
            .arg(&path)
            .status()?;
        if !status.success() {
            return Err(AppError::message("editor command failed"));
        }

        let edited_content = fs::read_to_string(&path)?;
        if edited_content.trim().is_empty() {
            fs::write(&path, original_content)?;
            return Err(AppError::message("task file cannot be empty"));
        }

        if edited_content != original_content
            && let Err(error) =
                repo.finalize_added_edit(&task.id, &path, &edited_content, Utc::now())
        {
            fs::write(&path, original_content)?;
            return Err(error);
        }
    }

    output::print_info(&format!("Created task: {} ({})", task.id, path.display()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Add;
    use clap::Parser;

    #[test]
    fn parses_add_command() {
        let add = Add::parse_from(["add", "Ship v2"]);
        assert_eq!(add.title.as_deref(), Some("Ship v2"));
    }
}
