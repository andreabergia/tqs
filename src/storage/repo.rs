use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::app::app_error::AppError;
use crate::domain::task::{Task, TaskStatus};
use crate::storage::format::{parse_task_markdown, render_task_markdown};

pub struct TaskRepo {
    root: PathBuf,
}

impl TaskRepo {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn task_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.md"))
    }

    pub fn id_exists(&self, id: &str) -> bool {
        self.task_path(id).exists()
    }

    pub fn create(&self, task: &Task) -> Result<(), AppError> {
        let path = self.task_path(&task.id);
        if path.exists() {
            return Err(AppError::message(format!(
                "task with id {} already exists",
                task.id
            )));
        }

        fs::create_dir_all(&self.root)?;
        let markdown = render_task_markdown(task)?;
        fs::write(&path, markdown)?;
        Ok(())
    }

    pub fn read(&self, id: &str) -> Result<Task, AppError> {
        let path = self.task_path(id);
        let content = fs::read_to_string(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::not_found(id)
            } else {
                AppError::from(e)
            }
        })?;

        parse_task_markdown(&content).map_err(|e| {
            AppError::invalid_task_file(path.to_string_lossy().to_string(), e.to_string())
        })
    }

    pub fn update(&self, task: &Task) -> Result<(), AppError> {
        let path = self.task_path(&task.id);
        if !path.exists() {
            return Err(AppError::not_found(&task.id));
        }

        let markdown = render_task_markdown(task)?;
        fs::write(&path, markdown)?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(AppError::not_found(id));
        }

        fs::remove_file(&path)?;
        Ok(())
    }

    pub fn rename_task(&self, old_id: &str, new_id: &str) -> Result<(), AppError> {
        let old_path = self.task_path(old_id);
        let new_path = self.task_path(new_id);

        if !old_path.exists() {
            return Err(AppError::not_found(old_id));
        }

        if new_path.exists() {
            return Err(AppError::message(format!(
                "task with id {new_id} already exists"
            )));
        }

        let mut task = self.read(old_id)?;
        task.id = new_id.to_string();
        let markdown = render_task_markdown(&task)?;
        fs::write(&new_path, markdown)?;
        fs::remove_file(&old_path)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Task>, AppError> {
        let mut tasks = Vec::new();

        if !self.root.exists() {
            return Ok(tasks);
        }

        let entries = fs::read_dir(&self.root)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str());
            if ext != Some("md") {
                continue;
            }

            match self.read_task_from_path(&path) {
                Ok(task) => tasks.push(task),
                Err(AppError::InvalidTaskFile { path, reason }) => {
                    eprintln!("Warning: skipping malformed task file {path}: {reason}");
                }
                Err(e) => return Err(e),
            }
        }

        tasks.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        Ok(tasks)
    }

    pub fn list_filtered(&self, status: TaskStatus) -> Result<Vec<Task>, AppError> {
        let all = self.list()?;
        Ok(all.into_iter().filter(|t| t.status == status).collect())
    }

    fn read_task_from_path(&self, path: &Path) -> Result<Task, AppError> {
        let content = fs::read_to_string(path)?;
        parse_task_markdown(&content).map_err(|e| {
            AppError::invalid_task_file(path.to_string_lossy().to_string(), e.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TaskRepo;
    use crate::domain::task::{Task, TaskStatus};
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_task(id: &str, summary: &str) -> Task {
        Task::new(id, Utc::now(), summary, None)
    }

    #[test]
    fn create_and_read_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let task = create_test_task("test-1234", "Test task");

        repo.create(&task).expect("task should be created");
        let read = repo.read(&task.id).expect("task should be read");

        assert_eq!(read.id, task.id);
        assert_eq!(read.summary, task.summary);
        assert_eq!(read.status, TaskStatus::Open);
    }

    #[test]
    fn create_fails_for_existing_id() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let task = create_test_task("test-1234", "Test task");

        repo.create(&task).expect("task should be created");
        let result = repo.create(&task);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn read_returns_not_found_for_missing_id() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        let result = repo.read("nonexistent");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn update_existing_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let mut task = create_test_task("test-1234", "Original summary");

        repo.create(&task).expect("task should be created");

        task.summary = "Updated summary".to_string();
        task.close();

        repo.update(&task).expect("task should be updated");
        let read = repo.read(&task.id).expect("task should be read");

        assert_eq!(read.summary, "Updated summary");
        assert_eq!(read.status, TaskStatus::Closed);
    }

    #[test]
    fn update_fails_for_missing_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let task = create_test_task("test-1234", "Test task");

        let result = repo.update(&task);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_existing_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let task = create_test_task("test-1234", "Test task");

        repo.create(&task).expect("task should be created");
        repo.delete(&task.id).expect("task should be deleted");

        let result = repo.read(&task.id);
        assert!(result.is_err());
    }

    #[test]
    fn delete_fails_for_missing_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        let result = repo.delete("nonexistent");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn list_returns_all_tasks_sorted() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        let now = Utc::now();
        let task1 = Task::new("alpha-1234", now, "First", None);
        let task2 = Task::new(
            "beta-5678",
            now + chrono::Duration::seconds(1),
            "Second",
            None,
        );
        let task3 = Task::new(
            "gamma-90ab",
            now + chrono::Duration::seconds(2),
            "Third",
            None,
        );

        repo.create(&task1).expect("task1 should be created");
        repo.create(&task3).expect("task3 should be created");
        repo.create(&task2).expect("task2 should be created");

        let tasks = repo.list().expect("tasks should be listed");

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].id, "gamma-90ab");
        assert_eq!(tasks[1].id, "beta-5678");
        assert_eq!(tasks[2].id, "alpha-1234");
    }

    #[test]
    fn list_skips_non_markdown_files() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        fs::write(temp.path().join("other.txt"), "not a task").expect("txt file should be written");

        let task = create_test_task("test-1234", "Test task");
        repo.create(&task).expect("task should be created");

        let tasks = repo.list().expect("tasks should be listed");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "test-1234");
    }

    #[test]
    fn list_skips_malformed_files() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        let task = create_test_task("test-1234", "Test task");
        repo.create(&task).expect("task should be created");

        let tasks = repo.list().expect("tasks should be listed");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "test-1234");
    }

    #[test]
    fn id_exists_returns_true_for_existing_task() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());
        let task = create_test_task("test-1234", "Test task");

        repo.create(&task).expect("task should be created");

        assert!(repo.id_exists("test-1234"));
        assert!(!repo.id_exists("nonexistent"));
    }

    #[test]
    fn list_filtered_by_status() {
        let temp = TempDir::new().expect("temp dir should be created");
        let repo = TaskRepo::new(temp.path().to_path_buf());

        let task1 = create_test_task("open-1234", "Open task");
        let mut task2 = create_test_task("closed-5678", "Closed task");
        task2.close();

        repo.create(&task1).expect("task1 should be created");
        repo.create(&task2).expect("task2 should be created");

        let open_tasks = repo
            .list_filtered(TaskStatus::Open)
            .expect("open tasks should be listed");
        assert_eq!(open_tasks.len(), 1);
        assert_eq!(open_tasks[0].id, "open-1234");

        let closed_tasks = repo
            .list_filtered(TaskStatus::Closed)
            .expect("closed tasks should be listed");
        assert_eq!(closed_tasks.len(), 1);
        assert_eq!(closed_tasks[0].id, "closed-5678");
    }
}
