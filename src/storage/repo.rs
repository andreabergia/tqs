use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};

use crate::app::app_error::AppError;
use crate::domain::{
    id::validate_user_id,
    task::{Queue, Task},
};
use crate::storage::{
    config::QueueDirs,
    format::{parse_task_markdown, render_task_markdown},
};

#[derive(Debug, Clone)]
pub struct StoredTask {
    pub task: Task,
    pub path: PathBuf,
}

pub struct TaskRepo {
    root: PathBuf,
    queue_dirs: QueueDirs,
}

impl TaskRepo {
    pub fn new(root: PathBuf, queue_dirs: QueueDirs) -> Self {
        Self { root, queue_dirs }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn queue_dir(&self, queue: Queue) -> PathBuf {
        self.root.join(self.queue_dirs.dir_name(queue))
    }

    pub fn task_path(&self, queue: Queue, id: &str) -> PathBuf {
        self.queue_dir(queue).join(format!("{id}.md"))
    }

    pub fn id_exists(&self, id: &str) -> bool {
        self.find_by_id(id).is_ok()
    }

    pub fn create(&self, task: &Task) -> Result<PathBuf, AppError> {
        validate_user_id(&task.id)?;
        let path = self.task_path(task.queue, &task.id);
        self.ensure_path_is_within_root(&path)?;

        if self.id_exists(&task.id) {
            return Err(AppError::usage(format!("id '{}' already exists", task.id)));
        }

        fs::create_dir_all(self.queue_dir(task.queue))?;
        fs::write(&path, render_task_markdown(task)?)?;
        Ok(path)
    }

    pub fn read(&self, id: &str) -> Result<Task, AppError> {
        Ok(self.find_by_id(id)?.task)
    }

    pub fn find_by_id(&self, id: &str) -> Result<StoredTask, AppError> {
        validate_user_id(id)?;
        let mut matches = self
            .scan_all()?
            .into_iter()
            .filter(|stored| stored.task.id == id)
            .collect::<Vec<_>>();

        match matches.len() {
            0 => Err(AppError::not_found(id)),
            1 => Ok(matches.remove(0)),
            _ => Err(AppError::message(format!(
                "multiple tasks found with id '{}'",
                id
            ))),
        }
    }

    pub fn update(&self, task: &Task) -> Result<PathBuf, AppError> {
        validate_user_id(&task.id)?;
        let existing = self.find_by_id(&task.id)?;
        let target_path = self.task_path(task.queue, &task.id);
        self.ensure_path_is_within_root(&target_path)?;
        fs::create_dir_all(self.queue_dir(task.queue))?;
        fs::write(&target_path, render_task_markdown(task)?)?;

        if existing.path != target_path && existing.path.exists() {
            fs::remove_file(existing.path)?;
        }

        Ok(target_path)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let stored = self.find_by_id(id)?;
        fs::remove_file(stored.path)?;
        Ok(())
    }

    pub fn move_to_queue(
        &self,
        id: &str,
        queue: Queue,
        now: DateTime<Utc>,
    ) -> Result<(Task, PathBuf, bool), AppError> {
        let mut task = self.read(id)?;
        let changed = task.move_to(queue, now);
        let path = self.update(&task)?;
        Ok((task, path, changed))
    }

    pub fn replace_edited(
        &self,
        original_id: &str,
        content: &str,
        now: DateTime<Utc>,
    ) -> Result<(Task, PathBuf), AppError> {
        let existing = self.find_by_id(original_id)?;
        let mut task = parse_task_markdown(content).map_err(|e| {
            AppError::invalid_task_file(existing.path.to_string_lossy().to_string(), e.to_string())
        })?;

        if task.id != original_id {
            return Err(AppError::usage("editing a task cannot change its id"));
        }

        task.normalize(now);
        let path = self.update(&task)?;
        Ok((task, path))
    }

    pub fn finalize_added_edit(
        &self,
        original_id: &str,
        path: &Path,
        content: &str,
        now: DateTime<Utc>,
    ) -> Result<(Task, PathBuf), AppError> {
        self.ensure_path_is_within_root(path)?;

        let mut task = parse_task_markdown(content).map_err(|error| {
            AppError::invalid_task_file(path.to_string_lossy().to_string(), error.to_string())
        })?;

        if task.id != original_id {
            return Err(AppError::usage("editing a task cannot change its id"));
        }

        task.normalize(now);
        let path = self.update(&task)?;
        Ok((task, path))
    }

    pub fn scan_all(&self) -> Result<Vec<StoredTask>, AppError> {
        let mut tasks = Vec::new();

        if !self.root.exists() {
            return Ok(tasks);
        }

        for queue in Queue::all().iter().copied() {
            let dir = self.queue_dir(queue);
            if !dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }

                match self.read_task_from_path(&path) {
                    Ok(task) => tasks.push(StoredTask { task, path }),
                    Err(AppError::InvalidTaskFile { path, reason }) => {
                        eprintln!("Warning: skipping malformed task file {path}: {reason}");
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        tasks.sort_by(|left, right| {
            left.task
                .queue
                .cmp(&right.task.queue)
                .then_with(|| right.task.updated_at.cmp(&left.task.updated_at))
                .then_with(|| left.task.id.cmp(&right.task.id))
        });
        Ok(tasks)
    }

    pub fn list(&self) -> Result<Vec<Task>, AppError> {
        Ok(self
            .scan_all()?
            .into_iter()
            .map(|stored| stored.task)
            .collect())
    }

    pub fn list_queue(&self, queue: Queue) -> Result<Vec<Task>, AppError> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|task| task.queue == queue)
            .collect())
    }

    fn read_task_from_path(&self, path: &Path) -> Result<Task, AppError> {
        self.ensure_path_is_within_root(path)?;
        let content = fs::read_to_string(path)?;
        let task = parse_task_markdown(&content).map_err(|error| {
            AppError::invalid_task_file(path.to_string_lossy().to_string(), error.to_string())
        })?;

        let expected_filename = format!("{}.md", task.id);
        if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename.as_str()) {
            return Err(AppError::invalid_task_file(
                path.to_string_lossy().to_string(),
                "task id does not match filename",
            ));
        }

        Ok(task)
    }

    fn ensure_path_is_within_root(&self, path: &Path) -> Result<(), AppError> {
        let canonical_root = self.root.canonicalize().or_else(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                fs::create_dir_all(&self.root)?;
                self.root.canonicalize()
            } else {
                Err(error)
            }
        })?;

        let parent = path.parent().unwrap_or(path);
        fs::create_dir_all(parent)?;
        let canonical_parent = parent.canonicalize()?;

        if canonical_parent.starts_with(&canonical_root) {
            Ok(())
        } else {
            Err(AppError::path_traversal_attempt(format!(
                "{} resolves outside root {}",
                path.display(),
                canonical_root.display()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TaskRepo;
    use crate::domain::task::{Queue, Task};
    use crate::storage::config::QueueDirs;
    use chrono::Utc;
    use std::fs;
    use tempfile::TempDir;

    fn task(id: &str, title: &str, queue: Queue) -> Task {
        let mut task = Task::new(id, title, Utc::now());
        task.queue = queue;
        task
    }

    #[test]
    fn create_read_and_move_across_queues() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        let task = task("task-1", "Ship v2", Queue::Inbox);

        let created_path = repo.create(&task).expect("task should be created");
        assert!(created_path.ends_with("inbox/task-1.md"));

        let (moved_task, moved_path, changed) = repo
            .move_to_queue("task-1", Queue::Now, Utc::now())
            .expect("task should move");
        assert!(changed);
        assert_eq!(moved_task.queue, Queue::Now);
        assert!(moved_path.ends_with("now/task-1.md"));
        assert!(!created_path.exists());
    }

    #[test]
    fn scans_all_queue_directories() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("task-1", "Inbox task", Queue::Inbox))
            .expect("inbox task should be created");
        repo.create(&task("task-2", "Later task", Queue::Later))
            .expect("later task should be created");

        let tasks = repo.list().expect("tasks should list");
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn malformed_files_are_skipped_during_scan() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        repo.create(&task("good", "Good task", Queue::Inbox))
            .expect("good task should be created");

        let inbox = temp.path().join("inbox");
        fs::create_dir_all(&inbox).expect("inbox should exist");
        fs::write(inbox.join("bad.md"), "---\nid: bad\nqueue: inbox\n---\n")
            .expect("bad file should be written");

        let tasks = repo.list().expect("scan should succeed");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "good");
    }

    #[test]
    fn update_keeps_filename_stable_when_title_changes() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(temp.path().to_path_buf(), QueueDirs::default());
        let mut task = task("task-1", "Old title", Queue::Inbox);
        repo.create(&task).expect("task should be created");

        task.title = "New title".to_string();
        let path = repo.update(&task).expect("task should update");
        assert!(path.ends_with("inbox/task-1.md"));
    }

    #[test]
    fn queue_directory_names_can_be_overridden() {
        let temp = TempDir::new().expect("temp dir should exist");
        let repo = TaskRepo::new(
            temp.path().to_path_buf(),
            QueueDirs {
                inbox: "capture".to_string(),
                now: "focus".to_string(),
                next: "up-next".to_string(),
                later: "backlog".to_string(),
                done: "archive".to_string(),
            },
        );

        repo.create(&task("task-1", "Ship v2", Queue::Inbox))
            .expect("task should be created");

        assert!(temp.path().join("capture").join("task-1.md").exists());
        assert!(!temp.path().join("inbox").join("task-1.md").exists());
    }
}
