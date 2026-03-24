use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::app_error::AppError;
use crate::domain::task::Queue;
use crate::storage::config::ResolvedConfig;
use crate::storage::editor::{ResolvedEditor, format_program_name, format_program_path};
use crate::storage::format::parse_task_markdown;
use crate::storage::id_state;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub scope: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl DoctorReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .count()
    }

    pub fn ok_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Ok)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}

pub fn run(config: &ResolvedConfig, fix: bool) -> Result<DoctorReport, AppError> {
    let mut diagnostics = Vec::new();

    diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Ok,
        scope: "config".to_string(),
        message: format!("resolved tasks_root to {}", config.tasks_root.display()),
    });

    let overlapping_queue_dirs = duplicate_queue_dirs(config);
    if overlapping_queue_dirs.is_empty() {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Ok,
            scope: "config".to_string(),
            message: "queue directory mappings are unique".to_string(),
        });
    } else {
        for (dir_name, queues) in overlapping_queue_dirs {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                scope: "config".to_string(),
                message: format!(
                    "queue directory '{}' is assigned to multiple queues: {}",
                    dir_name,
                    queues.join(", ")
                ),
            });
        }
    }

    diagnose_path(
        &mut diagnostics,
        "tasks_root",
        &config.tasks_root,
        MissingPathSeverity::Warning,
    )?;

    match &config.daily_notes_dir {
        Some(path) => diagnose_path(
            &mut diagnostics,
            "daily_notes_dir",
            path,
            MissingPathSeverity::Warning,
        )?,
        None => diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Ok,
            scope: "daily_notes_dir".to_string(),
            message: "unset".to_string(),
        }),
    }

    diagnose_editor(&mut diagnostics);

    if duplicate_queue_dirs(config).is_empty() {
        diagnose_task_files(&mut diagnostics, config)?;
    } else {
        diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Warning,
            scope: "tasks".to_string(),
            message: "skipped task scan because queue directory mappings overlap".to_string(),
        });
    }

    diagnose_state_files(&mut diagnostics, config, fix)?;

    Ok(DoctorReport { diagnostics })
}

#[derive(Debug, Clone, Copy)]
enum MissingPathSeverity {
    Warning,
}

fn diagnose_path(
    diagnostics: &mut Vec<Diagnostic>,
    scope: &str,
    path: &Path,
    missing_severity: MissingPathSeverity,
) -> Result<(), AppError> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Ok,
                    scope: scope.to_string(),
                    message: format!("{} exists", path.display()),
                });
            } else {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    scope: scope.to_string(),
                    message: format!("{} exists but is not a directory", path.display()),
                });
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let severity = match missing_severity {
                MissingPathSeverity::Warning => DiagnosticSeverity::Warning,
            };
            diagnostics.push(Diagnostic {
                severity,
                scope: scope.to_string(),
                message: format!("{} does not exist yet", path.display()),
            });
        }
        Err(error) => return Err(AppError::Io(error)),
    }

    Ok(())
}

fn diagnose_editor(diagnostics: &mut Vec<Diagnostic>) {
    match ResolvedEditor::resolve() {
        Ok(editor) => {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Ok,
                scope: "editor".to_string(),
                message: format!("resolved command to '{}'", editor.command),
            });

            match editor.executable_path() {
                Some(path) => diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Ok,
                    scope: "editor".to_string(),
                    message: format!(
                        "executable '{}' is available at {}",
                        format_program_name(&editor.program),
                        format_program_path(&path)
                    ),
                }),
                None => diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    scope: "editor".to_string(),
                    message: format!(
                        "executable '{}' was not found on PATH",
                        format_program_name(&editor.program)
                    ),
                }),
            }
        }
        Err(error) => diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            scope: "editor".to_string(),
            message: error.to_string(),
        }),
    }
}

fn diagnose_task_files(
    diagnostics: &mut Vec<Diagnostic>,
    config: &ResolvedConfig,
) -> Result<(), AppError> {
    let root_metadata = match fs::metadata(&config.tasks_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(AppError::Io(error)),
    };

    if !root_metadata.is_dir() {
        return Ok(());
    }

    let mut per_id_paths: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut scanned_files = 0usize;
    let mut seen_dirs = HashSet::new();

    for queue in Queue::all().iter().copied() {
        let dir_name = config.queue_dirs.dir_name(queue).to_string();
        if !seen_dirs.insert(dir_name.clone()) {
            continue;
        }

        let dir = config.tasks_root.join(&dir_name);
        match fs::metadata(&dir) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        scope: "tasks".to_string(),
                        message: format!("queue directory {} is not a directory", dir.display()),
                    });
                    continue;
                }

                let mut queue_file_count = 0usize;
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if !path.is_file() {
                        continue;
                    }

                    if path.extension().and_then(|value| value.to_str()) != Some("md") {
                        continue;
                    }

                    queue_file_count += 1;
                    scanned_files += 1;
                    diagnose_task_file(diagnostics, &path, queue, &mut per_id_paths)?;
                }

                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Ok,
                    scope: "tasks".to_string(),
                    message: format!(
                        "scanned {} Markdown task file(s) in {}",
                        queue_file_count,
                        dir.display()
                    ),
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Ok,
                    scope: "tasks".to_string(),
                    message: format!("queue directory {} is absent", dir.display()),
                });
            }
            Err(error) => return Err(AppError::Io(error)),
        }
    }

    for (id, paths) in per_id_paths {
        if paths.len() > 1 {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Error,
                scope: "tasks".to_string(),
                message: format!(
                    "duplicate task id '{}' found in {}",
                    id,
                    display_paths(&paths)
                ),
            });
        }
    }

    diagnostics.push(Diagnostic {
        severity: DiagnosticSeverity::Ok,
        scope: "tasks".to_string(),
        message: format!("scanned {} Markdown task file(s) total", scanned_files),
    });

    Ok(())
}

fn diagnose_task_file(
    diagnostics: &mut Vec<Diagnostic>,
    path: &Path,
    expected_queue: Queue,
    per_id_paths: &mut BTreeMap<String, Vec<PathBuf>>,
) -> Result<(), AppError> {
    let content = fs::read_to_string(path)?;
    match parse_task_markdown(&content) {
        Ok(task) => {
            let expected_filename = format!("{}.md", task.id);
            if path.file_name().and_then(|value| value.to_str()) != Some(expected_filename.as_str())
            {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    scope: "tasks".to_string(),
                    message: format!(
                        "{} has id '{}' but filename should be {}",
                        path.display(),
                        task.id,
                        expected_filename
                    ),
                });
            }

            if task.queue != expected_queue {
                diagnostics.push(Diagnostic {
                    severity: DiagnosticSeverity::Error,
                    scope: "tasks".to_string(),
                    message: format!(
                        "{} declares queue '{}' but is stored under '{}'",
                        path.display(),
                        task.queue,
                        expected_queue
                    ),
                });
            }

            per_id_paths
                .entry(task.id)
                .or_default()
                .push(path.to_path_buf());
        }
        Err(error) => diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            scope: "tasks".to_string(),
            message: format!("{} is malformed: {}", path.display(), error),
        }),
    }

    Ok(())
}

fn duplicate_queue_dirs(config: &ResolvedConfig) -> Vec<(String, Vec<String>)> {
    let mut by_dir: HashMap<String, BTreeSet<String>> = HashMap::new();
    for queue in Queue::all().iter().copied() {
        by_dir
            .entry(config.queue_dirs.dir_name(queue).to_string())
            .or_default()
            .insert(queue.to_string());
    }

    let mut duplicates = by_dir
        .into_iter()
        .filter_map(|(dir, queues)| {
            if queues.len() > 1 {
                Some((dir, queues.into_iter().collect::<Vec<_>>()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    duplicates.sort_by(|left, right| left.0.cmp(&right.0));
    duplicates
}

fn diagnose_state_files(
    diagnostics: &mut Vec<Diagnostic>,
    config: &ResolvedConfig,
    fix: bool,
) -> Result<(), AppError> {
    let id_gen_dir = config.state_dir.join("id-generator");

    let entries = match fs::read_dir(&id_gen_dir) {
        Ok(entries) => entries,
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::NotADirectory =>
        {
            return Ok(());
        }
        Err(error) => return Err(AppError::Io(error)),
    };

    let active_path = id_state::state_file_path(&config.state_dir, &config.tasks_root);

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        if path == active_path {
            continue;
        }

        if fix {
            match fs::remove_file(&path) {
                Ok(()) => {
                    diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Ok,
                        scope: "state".to_string(),
                        message: format!("removed orphaned state file {}", path.display()),
                    });
                }
                Err(error) => {
                    diagnostics.push(Diagnostic {
                        severity: DiagnosticSeverity::Error,
                        scope: "state".to_string(),
                        message: format!(
                            "failed to remove orphaned state file {}: {}",
                            path.display(),
                            error
                        ),
                    });
                }
            }
        } else {
            diagnostics.push(Diagnostic {
                severity: DiagnosticSeverity::Warning,
                scope: "state".to_string(),
                message: format!(
                    "orphaned state file {} (use --fix to remove)",
                    path.display()
                ),
            });
        }
    }

    Ok(())
}

fn display_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticSeverity, run};
    use crate::storage::config::{QueueDirs, ResolvedConfig};
    use crate::test_support::LockedEnv;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn config(root: &Path) -> ResolvedConfig {
        ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.to_path_buf(),
            state_dir: root.join(".tqs"),
            daily_notes_dir: None,
            queue_dirs: QueueDirs::default(),
        }
    }

    #[test]
    fn doctor_reports_duplicate_task_ids() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        fs::create_dir_all(root.join("inbox")).expect("inbox dir should exist");
        fs::create_dir_all(root.join("next")).expect("next dir should exist");
        fs::write(
            root.join("inbox").join("task-1.md"),
            "---\nid: task-1\ntitle: Inbox copy\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\ncompleted_at: null\ndaily_note: null\n---\n# Inbox\n",
        )
        .expect("inbox task should be written");
        fs::write(
            root.join("next").join("task-1.md"),
            "---\nid: task-1\ntitle: Next copy\nqueue: next\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\ncompleted_at: null\ndaily_note: null\n---\n# Next\n",
        )
        .expect("next task should be written");

        let report = run(&config(root), false).expect("doctor should succeed");
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message.contains("duplicate task id 'task-1'")
        }));
    }

    #[test]
    fn doctor_reports_queue_dir_overlap() {
        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(
            &ResolvedConfig {
                obsidian_vault_dir: None,
                tasks_root: temp.path().to_path_buf(),
                state_dir: temp.path().join(".tqs"),
                daily_notes_dir: None,
                queue_dirs: QueueDirs {
                    inbox: "shared".to_string(),
                    now: "shared".to_string(),
                    next: "next".to_string(),
                    later: "later".to_string(),
                    done: "done".to_string(),
                },
            },
            false,
        )
        .expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic
                    .message
                    .contains("queue directory 'shared' is assigned to multiple queues")
        }));
    }

    #[test]
    fn doctor_reports_resolved_editor_command() {
        let mut env = LockedEnv::new(&["VISUAL", "EDITOR"]);
        env.set("VISUAL", "sh -c 'exit 0' sh");
        env.remove("EDITOR");

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path()), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Ok
                && diagnostic.scope == "editor"
                && diagnostic.message.contains("resolved command to 'sh -c '")
        }));
    }

    #[test]
    fn doctor_reports_missing_editor_executable() {
        let mut env = LockedEnv::new(&["VISUAL", "EDITOR"]);
        env.set("VISUAL", "definitely-not-a-real-editor-tqs");
        env.remove("EDITOR");

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path()), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "editor"
                && diagnostic
                    .message
                    .contains("definitely-not-a-real-editor-tqs")
        }));
        assert!(report.has_errors());
    }

    #[test]
    fn doctor_reports_invalid_editor_command() {
        let mut env = LockedEnv::new(&["VISUAL", "EDITOR"]);
        env.set("VISUAL", "\"unterminated");
        env.remove("EDITOR");

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path()), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "editor"
                && diagnostic.message.contains("invalid editor command")
        }));
        assert!(report.has_errors());
    }

    #[test]
    fn doctor_reports_non_directory_tasks_root() {
        let temp = TempDir::new().expect("temp dir should exist");
        let tasks_root = temp.path().join("tasks.md");
        fs::write(&tasks_root, "not a directory").expect("file should be written");

        let report = run(&config(&tasks_root), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "tasks_root"
                && diagnostic.message.contains("exists but is not a directory")
        }));
    }

    #[test]
    fn doctor_reports_non_directory_daily_notes_dir() {
        let temp = TempDir::new().expect("temp dir should exist");
        let daily_notes = temp.path().join("daily.md");
        fs::write(&daily_notes, "not a directory").expect("file should be written");

        let report = run(
            &ResolvedConfig {
                obsidian_vault_dir: None,
                tasks_root: temp.path().join("tasks"),
                state_dir: temp.path().join("tasks").join(".tqs"),
                daily_notes_dir: Some(daily_notes.clone()),
                queue_dirs: QueueDirs::default(),
            },
            false,
        )
        .expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "daily_notes_dir"
                && diagnostic.message.contains(&format!(
                    "{} exists but is not a directory",
                    daily_notes.display()
                ))
        }));
    }

    #[test]
    fn doctor_reports_queue_paths_that_are_not_directories() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        fs::create_dir_all(root).expect("root should exist");
        fs::write(root.join("inbox"), "not a directory").expect("file should be written");

        let report = run(&config(root), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "tasks"
                && diagnostic.message.contains("queue directory")
                && diagnostic.message.contains("is not a directory")
        }));
    }

    #[test]
    fn doctor_reports_filename_mismatch_diagnostics() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        fs::create_dir_all(root.join("inbox")).expect("inbox dir should exist");
        fs::write(
            root.join("inbox").join("renamed.md"),
            "---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n",
        )
        .expect("task should be written");

        let report = run(&config(root), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic
                    .message
                    .contains("renamed.md has id 'task-1' but filename should be task-1.md")
        }));
    }

    #[test]
    fn doctor_warns_and_skips_task_scan_when_queue_mappings_overlap() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        fs::create_dir_all(root.join("shared")).expect("shared dir should exist");
        fs::write(
            root.join("shared").join("bad.md"),
            "---\nid: bad\nqueue: inbox\n---\n",
        )
        .expect("bad task should be written");

        let report = run(
            &ResolvedConfig {
                obsidian_vault_dir: None,
                tasks_root: root.to_path_buf(),
                state_dir: root.join(".tqs"),
                daily_notes_dir: None,
                queue_dirs: QueueDirs {
                    inbox: "shared".to_string(),
                    now: "shared".to_string(),
                    next: "next".to_string(),
                    later: "later".to_string(),
                    done: "done".to_string(),
                },
            },
            false,
        )
        .expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Warning
                && diagnostic
                    .message
                    .contains("skipped task scan because queue directory mappings overlap")
        }));
        assert!(!report.diagnostics.iter().any(|diagnostic| {
            diagnostic.scope == "tasks" && diagnostic.message.contains("bad.md is malformed")
        }));
    }

    #[test]
    fn doctor_warns_about_orphaned_state_files() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        let state_dir = root.join(".tqs");
        let id_gen_dir = state_dir.join("id-generator");
        fs::create_dir_all(&id_gen_dir).expect("id-generator dir should exist");

        let active_path = crate::storage::id_state::state_file_path(&state_dir, root);
        fs::write(
            &active_path,
            "version = 1\nwidth = 3\nnext_value = \"0\"\nissued_count = \"0\"\n",
        )
        .expect("active state file should be written");
        let orphan = id_gen_dir.join("deadbeef.toml");
        fs::write(&orphan, "stale").expect("orphan should be written");

        let report = run(&config(root), false).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Warning
                && diagnostic.scope == "state"
                && diagnostic.message.contains("orphaned state file")
                && diagnostic.message.contains("deadbeef.toml")
        }));
    }

    #[test]
    fn doctor_fix_removes_orphaned_state_files() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        let state_dir = root.join(".tqs");
        let id_gen_dir = state_dir.join("id-generator");
        fs::create_dir_all(&id_gen_dir).expect("id-generator dir should exist");

        let active_path = crate::storage::id_state::state_file_path(&state_dir, root);
        fs::write(
            &active_path,
            "version = 1\nwidth = 3\nnext_value = \"0\"\nissued_count = \"0\"\n",
        )
        .expect("active state file should be written");
        let orphan = id_gen_dir.join("deadbeef.toml");
        fs::write(&orphan, "stale").expect("orphan should be written");

        let report = run(&config(root), true).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Ok
                && diagnostic.scope == "state"
                && diagnostic.message.contains("removed orphaned state file")
        }));
        assert!(!orphan.exists());
        assert!(active_path.exists());
    }

    #[test]
    fn doctor_ignores_non_toml_files_in_state_dir() {
        let temp = TempDir::new().expect("temp dir should exist");
        let root = temp.path();
        let state_dir = root.join(".tqs");
        let id_gen_dir = state_dir.join("id-generator");
        fs::create_dir_all(&id_gen_dir).expect("id-generator dir should exist");

        fs::write(id_gen_dir.join("something.lock"), "lock").expect("lock should be written");
        fs::write(id_gen_dir.join("something.tmp"), "tmp").expect("tmp should be written");

        let report = run(&config(root), false).expect("doctor should succeed");

        assert!(!report.diagnostics.iter().any(|diagnostic| {
            diagnostic.scope == "state" && diagnostic.message.contains("orphaned")
        }));
    }
}
