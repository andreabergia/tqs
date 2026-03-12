use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    app::app_error::AppError,
    domain::id::{
        MAX_GENERATED_ID_WIDTH, MIN_GENERATED_ID_WIDTH, encode_generated_id, id_space_size,
        next_sequence_value,
    },
    storage::{config::ResolvedConfig, repo::TaskRepo},
};

const STATE_FILE_VERSION: u32 = 1;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(20);
const MAX_LOCK_ATTEMPTS: usize = 250;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedIdAllocator {
    state_path: PathBuf,
    lock_path: PathBuf,
}

impl SharedIdAllocator {
    pub fn new(config: &ResolvedConfig) -> Self {
        let state_path = state_file_path(&config.state_dir, &config.tasks_root);
        let lock_path = state_path.with_extension("lock");
        Self {
            state_path,
            lock_path,
        }
    }

    pub fn state_path(&self) -> &Path {
        &self.state_path
    }

    pub fn generate(&self, repo: &TaskRepo) -> Result<String, AppError> {
        let _lock = FileLock::acquire(&self.lock_path)?;
        let mut state = self.load_state()?;

        loop {
            state.advance_width_if_exhausted()?;
            let modulus = id_space_size(state.width)?;
            let start = state.next_value % modulus;
            let mut current = start;
            let mut scanned = 0u128;

            while scanned < modulus {
                let id = encode_generated_id(current, state.width)?;
                let next = next_sequence_value(current, state.width)?;
                current = next;
                scanned += 1;

                if repo.id_exists(&id) {
                    continue;
                }

                state.next_value = next;
                state.issued_count += 1;
                self.save_state(&state)?;
                return Ok(id);
            }

            state.width += 1;
            state.next_value = 0;
            state.issued_count = 0;
        }
    }

    fn load_state(&self) -> Result<GeneratorState, AppError> {
        match fs::read_to_string(&self.state_path) {
            Ok(contents) => {
                let persisted: PersistedGeneratorState = toml::from_str(&contents)
                    .unwrap_or_else(|_| PersistedGeneratorState::default());
                GeneratorState::from_persisted(persisted)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(GeneratorState::default())
            }
            Err(error) => Err(AppError::Io(error)),
        }
    }

    fn save_state(&self, state: &GeneratorState) -> Result<(), AppError> {
        let parent = self.state_path.parent().ok_or_else(|| {
            AppError::message("allocator state path is missing a parent directory")
        })?;
        fs::create_dir_all(parent)?;

        let persisted = PersistedGeneratorState::from(state);
        let serialized = toml::to_string(&persisted).map_err(|error| {
            AppError::message(format!("failed to serialize allocator state: {error}"))
        })?;
        let temp_path = self.state_path.with_extension("tmp");
        fs::write(&temp_path, serialized)?;
        fs::rename(temp_path, &self.state_path)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratorState {
    width: u8,
    next_value: u128,
    issued_count: u128,
}

impl Default for GeneratorState {
    fn default() -> Self {
        Self {
            width: MIN_GENERATED_ID_WIDTH,
            next_value: 0,
            issued_count: 0,
        }
    }
}

impl GeneratorState {
    fn from_persisted(persisted: PersistedGeneratorState) -> Result<Self, AppError> {
        let width = if (MIN_GENERATED_ID_WIDTH..=MAX_GENERATED_ID_WIDTH).contains(&persisted.width)
        {
            persisted.width
        } else {
            MIN_GENERATED_ID_WIDTH
        };
        let next_value = persisted.next_value.parse::<u128>().unwrap_or_default();
        let issued_count = persisted.issued_count.parse::<u128>().unwrap_or_default();

        let mut state = Self {
            width,
            next_value,
            issued_count,
        };
        state.advance_width_if_exhausted()?;
        Ok(state)
    }

    fn advance_width_if_exhausted(&mut self) -> Result<(), AppError> {
        while self.issued_count >= id_space_size(self.width)? {
            self.width += 1;
            self.next_value = 0;
            self.issued_count = 0;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedGeneratorState {
    version: u32,
    width: u8,
    next_value: String,
    issued_count: String,
}

impl Default for PersistedGeneratorState {
    fn default() -> Self {
        Self::from(&GeneratorState::default())
    }
}

impl From<&GeneratorState> for PersistedGeneratorState {
    fn from(state: &GeneratorState) -> Self {
        Self {
            version: STATE_FILE_VERSION,
            width: state.width,
            next_value: state.next_value.to_string(),
            issued_count: state.issued_count.to_string(),
        }
    }
}

fn state_file_path(state_dir: &Path, tasks_root: &Path) -> PathBuf {
    let root = tasks_root
        .canonicalize()
        .unwrap_or_else(|_| tasks_root.to_path_buf());
    let hash = stable_path_hash(&root);
    state_dir.join("id-generator").join(format!("{hash}.toml"))
}

fn stable_path_hash(path: &Path) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in path.to_string_lossy().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

struct FileLock {
    path: PathBuf,
}

impl FileLock {
    fn acquire(path: &Path) -> Result<Self, AppError> {
        let parent = path.parent().ok_or_else(|| {
            AppError::message("allocator lock path is missing a parent directory")
        })?;
        fs::create_dir_all(parent)?;

        for _ in 0..MAX_LOCK_ATTEMPTS {
            match OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(_) => {
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) => return Err(AppError::Io(error)),
            }
        }

        Err(AppError::message(format!(
            "timed out waiting for allocator lock {}",
            path.display()
        )))
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        sync::{Arc, Barrier},
        thread,
    };

    use tempfile::TempDir;

    use crate::{
        domain::task::{Queue, Task},
        storage::{
            config::{QueueDirs, ResolvedConfig},
            repo::TaskRepo,
        },
    };

    use super::{GeneratorState, SharedIdAllocator, stable_path_hash};

    fn config(root: &Path, state_dir: &Path) -> ResolvedConfig {
        ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.to_path_buf(),
            state_dir: state_dir.to_path_buf(),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        }
    }

    fn task(id: &str, title: &str) -> Task {
        let mut task = Task::new(id.to_string(), title.to_string(), chrono::Utc::now());
        task.queue = Queue::Inbox;
        task
    }

    #[test]
    fn allocator_persists_state_under_repo_local_directory() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path().join("tasks");
        let state_dir = temp.path().join(".tqs");
        let repo = TaskRepo::new(root.clone(), QueueDirs::default());
        let allocator = SharedIdAllocator::new(&config(&root, &state_dir));

        let id = allocator.generate(&repo).expect("ID should generate");

        assert_eq!(id.len(), 3);
        assert!(allocator.state_path().starts_with(&state_dir));
        assert!(allocator.state_path().exists());
    }

    #[test]
    fn allocator_recovers_from_corrupt_state_file() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path().join("tasks");
        let state_dir = temp.path().join(".tqs");
        let repo = TaskRepo::new(root.clone(), QueueDirs::default());
        let allocator = SharedIdAllocator::new(&config(&root, &state_dir));
        let parent = allocator
            .state_path()
            .parent()
            .expect("state path should have a parent");
        fs::create_dir_all(parent).expect("state parent should exist");
        fs::write(allocator.state_path(), "not valid toml").expect("state file should exist");

        let id = allocator.generate(&repo).expect("allocator should recover");

        assert_eq!(id, "000");
    }

    #[test]
    fn allocator_skips_existing_ids_when_state_is_missing() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path().join("tasks");
        let state_dir = temp.path().join(".tqs");
        let repo = TaskRepo::new(root.clone(), QueueDirs::default());
        repo.create(&task("000", "Existing task"))
            .expect("existing task should be created");
        let allocator = SharedIdAllocator::new(&config(&root, &state_dir));

        let id = allocator
            .generate(&repo)
            .expect("allocator should generate");

        assert_ne!(id, "000");
        assert_eq!(id.len(), 3);
    }

    #[test]
    fn allocator_advances_to_wider_ids_after_exhaustion() {
        let mut state = GeneratorState {
            width: 3,
            next_value: 0,
            issued_count: 32u128.pow(3),
        };

        state
            .advance_width_if_exhausted()
            .expect("state should advance");

        assert_eq!(state.width, 4);
        assert_eq!(state.next_value, 0);
        assert_eq!(state.issued_count, 0);
    }

    #[test]
    fn state_path_is_keyed_by_task_root() {
        let temp = TempDir::new().expect("temp dir should exist");
        let state_dir = temp.path().join(".tqs");
        let first_root = temp.path().join("tasks-a");
        let second_root = temp.path().join("tasks-b");
        let first = SharedIdAllocator::new(&config(&first_root, &state_dir));
        let second = SharedIdAllocator::new(&config(&second_root, &state_dir));

        assert_ne!(first.state_path(), second.state_path());
        assert!(
            first
                .state_path()
                .display()
                .to_string()
                .contains(&stable_path_hash(&first_root))
        );
    }

    #[test]
    fn allocator_lock_serializes_concurrent_generation() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path().join("tasks");
        let state_dir = temp.path().join(".tqs");
        let repo = Arc::new(TaskRepo::new(root.clone(), QueueDirs::default()));
        let config = config(&root, &state_dir);
        let first = SharedIdAllocator::new(&config);
        let second = SharedIdAllocator::new(&config);
        let barrier = Arc::new(Barrier::new(2));

        let first_barrier = Arc::clone(&barrier);
        let first_repo = Arc::clone(&repo);
        let first_handle = thread::spawn(move || {
            first_barrier.wait();
            first.generate(&first_repo)
        });

        let second_barrier = Arc::clone(&barrier);
        let second_repo = Arc::clone(&repo);
        let second_handle = thread::spawn(move || {
            second_barrier.wait();
            second.generate(&second_repo)
        });

        let first_id = first_handle
            .join()
            .expect("first thread should finish")
            .expect("first ID should generate");
        let second_id = second_handle
            .join()
            .expect("second thread should finish")
            .expect("second ID should generate");

        assert_ne!(first_id, second_id);
    }
}
