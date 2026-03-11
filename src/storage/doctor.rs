use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::app_error::AppError;
use crate::domain::task::Queue;
use crate::storage::config::ResolvedConfig;
use crate::storage::editor::{ResolvedEditor, format_program_name, format_program_path};
use crate::storage::format::parse_task_markdown;

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

pub fn run(config: &ResolvedConfig) -> Result<DoctorReport, AppError> {
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
    use std::fs;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn config(root: &Path) -> ResolvedConfig {
        ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: root.to_path_buf(),
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
            "---\nid: task-1\ntitle: Inbox copy\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n# Inbox\n",
        )
        .expect("inbox task should be written");
        fs::write(
            root.join("next").join("task-1.md"),
            "---\nid: task-1\ntitle: Next copy\nqueue: next\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n# Next\n",
        )
        .expect("next task should be written");

        let report = run(&config(root)).expect("doctor should succeed");
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.message.contains("duplicate task id 'task-1'")
        }));
    }

    #[test]
    fn doctor_reports_queue_dir_overlap() {
        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&ResolvedConfig {
            obsidian_vault_dir: None,
            tasks_root: temp.path().to_path_buf(),
            daily_notes_dir: None,
            queue_dirs: QueueDirs {
                inbox: "shared".to_string(),
                now: "shared".to_string(),
                next: "next".to_string(),
                later: "later".to_string(),
                done: "done".to_string(),
            },
        })
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
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "sh -c 'exit 0' sh");
            std::env::remove_var("EDITOR");
        }

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path())).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Ok
                && diagnostic.scope == "editor"
                && diagnostic.message.contains("resolved command to 'sh -c '")
        }));

        unsafe {
            std::env::remove_var("VISUAL");
        }
    }

    #[test]
    fn doctor_reports_missing_editor_executable() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "definitely-not-a-real-editor-tqs");
            std::env::remove_var("EDITOR");
        }

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path())).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "editor"
                && diagnostic
                    .message
                    .contains("definitely-not-a-real-editor-tqs")
        }));
        assert!(report.has_errors());

        unsafe {
            std::env::remove_var("VISUAL");
        }
    }

    #[test]
    fn doctor_reports_invalid_editor_command() {
        let _guard = env_lock().lock().expect("env lock should work");
        unsafe {
            std::env::set_var("VISUAL", "\"unterminated");
            std::env::remove_var("EDITOR");
        }

        let temp = TempDir::new().expect("temp dir should exist");
        let report = run(&config(temp.path())).expect("doctor should succeed");

        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.scope == "editor"
                && diagnostic.message.contains("invalid editor command")
        }));
        assert!(report.has_errors());

        unsafe {
            std::env::remove_var("VISUAL");
        }
    }
}
