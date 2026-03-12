use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;

use crate::{app::app_error::AppError, domain::task::Queue};

const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigInspection {
    pub config_path: Option<PathBuf>,
    pub file_exists: bool,
    pub explicit_root: Option<PathBuf>,
    pub env_root: Option<PathBuf>,
    pub resolved: Option<ResolvedConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub obsidian_vault_dir: Option<PathBuf>,
    pub tasks_root: PathBuf,
    pub state_dir: PathBuf,
    pub daily_notes_dir: Option<PathBuf>,
    pub queue_dirs: QueueDirs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueDirs {
    pub(crate) inbox: String,
    pub(crate) now: String,
    pub(crate) next: String,
    pub(crate) later: String,
    pub(crate) done: String,
}

impl Default for QueueDirs {
    fn default() -> Self {
        Self {
            inbox: "inbox".to_string(),
            now: "now".to_string(),
            next: "next".to_string(),
            later: "later".to_string(),
            done: "done".to_string(),
        }
    }
}

impl QueueDirs {
    pub fn dir_name(&self, queue: Queue) -> &str {
        match queue {
            Queue::Inbox => &self.inbox,
            Queue::Now => &self.now,
            Queue::Next => &self.next,
            Queue::Later => &self.later,
            Queue::Done => &self.done,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    obsidian_vault_dir: Option<PathBuf>,
    tasks_root: Option<PathBuf>,
    daily_notes_dir: Option<PathBuf>,
    #[serde(default)]
    queues: QueueDirsOverride,
}

#[derive(Debug, Default, Deserialize)]
struct QueueDirsOverride {
    inbox: Option<String>,
    now: Option<String>,
    next: Option<String>,
    later: Option<String>,
    done: Option<String>,
}

impl QueueDirsOverride {
    fn has_overrides(&self) -> bool {
        self.inbox.is_some()
            || self.now.is_some()
            || self.next.is_some()
            || self.later.is_some()
            || self.done.is_some()
    }
}

pub fn resolve(explicit_root: Option<PathBuf>) -> Result<ResolvedConfig, AppError> {
    let file_config = load_file_config()?;
    let tasks_root = explicit_root
        .or_else(|| env_path("TQS_ROOT"))
        .or_else(|| {
            file_config
                .as_ref()
                .and_then(|config| config.tasks_root.clone())
        })
        .ok_or_else(missing_tasks_root_error)?;

    let daily_notes_dir = file_config
        .as_ref()
        .and_then(|config| config.daily_notes_dir.clone());
    let queue_dirs = file_config
        .as_ref()
        .map(|config| build_queue_dirs(&config.queues))
        .transpose()?
        .unwrap_or_default();
    let state_dir = file_config
        .as_ref()
        .and_then(|config| config.obsidian_vault_dir.clone())
        .unwrap_or_else(|| tasks_root.clone())
        .join(".tqs");

    Ok(ResolvedConfig {
        obsidian_vault_dir: file_config
            .as_ref()
            .and_then(|config| config.obsidian_vault_dir.clone()),
        tasks_root,
        state_dir,
        daily_notes_dir,
        queue_dirs,
    })
}

pub fn inspect(explicit_root: Option<PathBuf>) -> Result<ConfigInspection, AppError> {
    let config_path = config_path();
    let file_exists = config_path.as_ref().is_some_and(|path| path.exists());
    let env_root = env_path("TQS_ROOT");
    let resolved = match resolve(explicit_root.clone()) {
        Ok(resolved) => Some(resolved),
        Err(error) if error.to_string().starts_with("missing tasks_root;") => None,
        Err(error) => return Err(error),
    };

    Ok(ConfigInspection {
        config_path,
        file_exists,
        explicit_root,
        env_root,
        resolved,
    })
}

pub fn starter_config(config_path: Option<&Path>) -> String {
    let mut message = String::new();
    let config_display = config_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.config/tqs/config.toml".to_string());
    message.push_str("To get started:\n");
    message.push_str("  1. Try a one-off root: tqs --root ~/tasks add \"My first task\"\n");
    message.push_str(&format!("  2. Or create {config_display} with:\n"));
    message.push_str("     tasks_root = \"~/tasks\"\n");
    message.push_str("     # Optional for Obsidian users:\n");
    message.push_str("     # obsidian_vault_dir = \"~/vault\"\n");
    message
}

fn missing_tasks_root_error() -> AppError {
    let config_path = config_path();
    let location = config_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "~/.config/tqs/config.toml".to_string());

    AppError::message(format!(
        "missing tasks_root; pass --root, set TQS_ROOT, or configure it in {location}\n\n{}",
        starter_config(config_path.as_deref())
    ))
}

fn load_file_config() -> Result<Option<FileConfig>, AppError> {
    let Some(path) = config_path() else {
        return Ok(None);
    };

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)?;
    let mut parsed: FileConfig = toml::from_str(&contents).map_err(|error| {
        AppError::message(format!("invalid config file {}: {error}", path.display()))
    })?;

    let base_dir = path.parent().unwrap_or(Path::new("."));
    parsed.obsidian_vault_dir = parsed
        .obsidian_vault_dir
        .map(|value| absolutize_from(base_dir, value));
    parsed.tasks_root = parsed
        .tasks_root
        .map(|value| absolutize_from(base_dir, value));
    parsed.daily_notes_dir = parsed
        .daily_notes_dir
        .map(|value| absolutize_from(base_dir, value));
    apply_obsidian_alias(&mut parsed)?;

    Ok(Some(parsed))
}

fn apply_obsidian_alias(config: &mut FileConfig) -> Result<(), AppError> {
    let Some(vault_dir) = config.obsidian_vault_dir.as_ref() else {
        return Ok(());
    };

    if config.tasks_root.is_some() {
        return Err(AppError::message(
            "invalid config: obsidian_vault_dir cannot be combined with tasks_root",
        ));
    }

    if config.daily_notes_dir.is_some() {
        return Err(AppError::message(
            "invalid config: obsidian_vault_dir cannot be combined with daily_notes_dir",
        ));
    }

    if config.queues.has_overrides() {
        return Err(AppError::message(
            "invalid config: obsidian_vault_dir cannot be combined with queue directory overrides",
        ));
    }

    config.tasks_root = Some(vault_dir.join("Tasks"));
    config.daily_notes_dir = Some(vault_dir.join("Daily Notes"));
    Ok(())
}

fn build_queue_dirs(overrides: &QueueDirsOverride) -> Result<QueueDirs, AppError> {
    let defaults = QueueDirs::default();

    Ok(QueueDirs {
        inbox: queue_dir_name(overrides.inbox.as_deref(), &defaults.inbox)?,
        now: queue_dir_name(overrides.now.as_deref(), &defaults.now)?,
        next: queue_dir_name(overrides.next.as_deref(), &defaults.next)?,
        later: queue_dir_name(overrides.later.as_deref(), &defaults.later)?,
        done: queue_dir_name(overrides.done.as_deref(), &defaults.done)?,
    })
}

fn queue_dir_name(value: Option<&str>, default: &str) -> Result<String, AppError> {
    match value {
        None => Ok(default.to_string()),
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(AppError::message(
                    "invalid config: queue directory names cannot be empty",
                ));
            }

            let path = Path::new(trimmed);
            let mut components = path.components();
            match (components.next(), components.next()) {
                (Some(Component::Normal(_)), None) => Ok(trimmed.to_string()),
                _ => Err(AppError::message(format!(
                    "invalid config: queue directory '{trimmed}' must be a single path segment"
                ))),
            }
        }
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(xdg_config) = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Some(xdg_config.join("tqs").join(CONFIG_FILE_NAME));
    }

    env_path("HOME").map(|home| home.join(".config").join("tqs").join(CONFIG_FILE_NAME))
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn absolutize_from(base_dir: &Path, value: PathBuf) -> PathBuf {
    if value.is_absolute() {
        value
    } else {
        base_dir.join(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{QueueDirsOverride, build_queue_dirs, inspect, resolve};
    use crate::test_support::LockedEnv;
    use std::{fs, path::PathBuf};
    use tempfile::TempDir;

    #[test]
    fn resolve_uses_cli_root_before_env_and_config() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "tasks_root = '/config/tasks'\n",
        )
        .expect("config file should exist");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());
        env.set("TQS_ROOT", "/env/tasks");

        let resolved = resolve(Some(PathBuf::from("/cli/tasks"))).expect("config should resolve");
        assert_eq!(resolved.tasks_root, PathBuf::from("/cli/tasks"));
        assert_eq!(resolved.state_dir, PathBuf::from("/cli/tasks").join(".tqs"));
    }

    #[test]
    fn resolve_reads_paths_from_config_file() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "tasks_root = 'tasks'\ndaily_notes_dir = 'daily'\n[queues]\nnow = 'focus'\ndone = 'archive'\n",
        )
        .expect("config file should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let resolved = resolve(None).expect("config should resolve");
        assert_eq!(resolved.obsidian_vault_dir, None);
        assert_eq!(resolved.tasks_root, config_dir.join("tasks"));
        assert_eq!(resolved.state_dir, config_dir.join("tasks").join(".tqs"));
        assert_eq!(resolved.daily_notes_dir, Some(config_dir.join("daily")));
        assert_eq!(
            resolved
                .queue_dirs
                .dir_name(crate::domain::task::Queue::Now),
            "focus"
        );
        assert_eq!(
            resolved
                .queue_dirs
                .dir_name(crate::domain::task::Queue::Done),
            "archive"
        );
    }

    #[test]
    fn inspect_reports_missing_config_file_and_root_sources() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let inspection = inspect(None).expect("inspection should succeed");

        assert_eq!(
            inspection.config_path,
            Some(config_home.join("tqs").join("config.toml"))
        );
        assert!(!inspection.file_exists);
        assert!(inspection.resolved.is_none());
    }

    #[test]
    fn missing_tasks_root_error_includes_starter_config() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let error = resolve(None).expect_err("missing root should fail");
        let message = error.to_string();

        assert!(message.contains("missing tasks_root"));
        assert!(message.contains("To get started:"));
        assert!(message.contains("tqs --root ~/tasks add \"My first task\""));
        assert!(
            message.contains(
                &config_home
                    .join("tqs")
                    .join("config.toml")
                    .display()
                    .to_string()
            )
        );
    }

    #[test]
    fn resolve_errors_when_tasks_root_is_missing() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", temp.path().as_os_str());

        let error = resolve(None).expect_err("missing config should error");
        assert!(
            error
                .to_string()
                .contains("missing tasks_root; pass --root, set TQS_ROOT")
        );
    }

    #[test]
    fn queue_directory_overrides_must_be_single_path_segments() {
        let error = build_queue_dirs(&QueueDirsOverride {
            inbox: Some("../escape".to_string()),
            now: None,
            next: None,
            later: None,
            done: None,
        })
        .expect_err("invalid queue dir should error");

        assert!(
            error
                .to_string()
                .contains("queue directory '../escape' must be a single path segment")
        );
    }

    #[test]
    fn resolve_derives_paths_from_obsidian_vault_dir() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "obsidian_vault_dir = 'vault'\n",
        )
        .expect("config file should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let resolved = resolve(None).expect("config should resolve");
        assert_eq!(resolved.obsidian_vault_dir, Some(config_dir.join("vault")));
        assert_eq!(resolved.tasks_root, config_dir.join("vault").join("Tasks"));
        assert_eq!(resolved.state_dir, config_dir.join("vault").join(".tqs"));
        assert_eq!(
            resolved.daily_notes_dir,
            Some(config_dir.join("vault").join("Daily Notes"))
        );
    }

    #[test]
    fn resolve_rejects_mixing_obsidian_vault_dir_with_tasks_root() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "obsidian_vault_dir = 'vault'\ntasks_root = 'tasks'\n",
        )
        .expect("config file should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let error = resolve(None).expect_err("config should be rejected");
        assert!(
            error
                .to_string()
                .contains("obsidian_vault_dir cannot be combined with tasks_root")
        );
    }

    #[test]
    fn resolve_rejects_mixing_obsidian_vault_dir_with_daily_notes_dir() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "obsidian_vault_dir = 'vault'\ndaily_notes_dir = 'daily'\n",
        )
        .expect("config file should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let error = resolve(None).expect_err("config should be rejected");
        assert!(
            error
                .to_string()
                .contains("obsidian_vault_dir cannot be combined with daily_notes_dir")
        );
    }

    #[test]
    fn resolve_rejects_mixing_obsidian_vault_dir_with_queue_overrides() {
        let mut env = LockedEnv::new(&["XDG_CONFIG_HOME", "TQS_ROOT"]);
        let temp = TempDir::new().expect("temp dir should exist");
        let config_home = temp.path().join("config-home");
        let config_dir = config_home.join("tqs");
        fs::create_dir_all(&config_dir).expect("config dir should exist");
        fs::write(
            config_dir.join("config.toml"),
            "obsidian_vault_dir = 'vault'\n[queues]\ninbox = 'capture'\n",
        )
        .expect("config file should exist");
        env.remove("TQS_ROOT");
        env.set("XDG_CONFIG_HOME", config_home.as_os_str());

        let error = resolve(None).expect_err("config should be rejected");
        assert!(
            error
                .to_string()
                .contains("obsidian_vault_dir cannot be combined with queue directory overrides")
        );
    }
}
