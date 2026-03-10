use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use chrono::Local;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;

fn write_task(root: &std::path::Path, queue: &str, id: &str, title: &str, body: &str) {
    let queue_dir = root.join(queue);
    fs::create_dir_all(&queue_dir).expect("queue dir should exist");
    fs::write(
        queue_dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: {title}\nqueue: {queue}\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n{body}"
        ),
    )
    .expect("task file should be written");
}

fn write_task_with_metadata(
    root: &std::path::Path,
    queue: &str,
    id: &str,
    title: &str,
    body: &str,
    tags: &[&str],
    source: Option<&str>,
    project: Option<&str>,
) {
    let queue_dir = root.join(queue);
    fs::create_dir_all(&queue_dir).expect("queue dir should exist");
    let tags = if tags.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", tags.join(", "))
    };
    let source = source.unwrap_or("null");
    let project = project.unwrap_or("null");
    fs::write(
        queue_dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: {title}\nqueue: {queue}\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: {tags}\nsource: {source}\nproject: {project}\ncompleted_at: null\ndaily_note: null\n---\n{body}"
        ),
    )
    .expect("task file should be written");
}

#[test]
fn help_command_works() {
    cargo_bin_cmd!("tqs")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Terminal task queue"));
}

#[test]
fn add_creates_task_in_inbox() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    assert!(temp.path().join("inbox").join("task-1.md").exists());
}

#[test]
fn list_without_queue_shows_dashboard() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(
            contains("now")
                .and(contains("inbox"))
                .and(contains("Review PR")),
        );
}

#[test]
fn list_queue_shows_only_requested_queue() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("now")
        .assert()
        .success()
        .stdout(
            contains("now")
                .and(contains("Do now"))
                .and(contains("Review PR").not()),
        );
}

#[test]
fn move_relocates_file_to_target_queue() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg("task-1")
        .arg("now")
        .assert()
        .success()
        .stdout(contains("Moved task: task-1"));

    assert!(!temp.path().join("inbox").join("task-1.md").exists());
    assert!(temp.path().join("now").join("task-1.md").exists());
}

#[test]
fn move_promotes_task_from_inbox_to_now() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Review outage notes")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg("task-1")
        .arg("now")
        .assert()
        .success()
        .stdout(contains("Moved task: task-1"));

    assert!(!temp.path().join("inbox").join("task-1.md").exists());
    assert!(temp.path().join("now").join("task-1.md").exists());
}

#[test]
fn move_is_noop_when_task_is_already_in_target_queue() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg("task-1")
        .arg("now")
        .assert()
        .success()
        .stdout(contains("Task task-1 is already in now"));

    assert!(temp.path().join("now").join("task-1.md").exists());
    assert!(!temp.path().join("inbox").join("task-1.md").exists());
}

#[test]
fn done_is_idempotent() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("done")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("done")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("already done"));
}

#[test]
fn show_prints_metadata_path_and_body() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "inbox",
        "task-1",
        "Ship v2",
        "# Ship v2\n\n## Notes",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("show")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Path:").and(contains("# Ship v2")));
}

#[test]
fn find_matches_body_text() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "next",
        "task-1",
        "Investigate billing",
        "# Investigate billing\n\nLook at cost explorer.",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("find")
        .arg("cost explorer")
        .assert()
        .success()
        .stdout(contains("task-1").and(contains("Investigate billing")));
}

#[test]
fn old_command_names_are_rejected() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Ship v2")
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg("task-1")
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));
}

#[test]
fn show_resolves_unique_title_substring() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "inbox",
        "task-1",
        "Ship v2 release",
        "# Ship v2 release",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("show")
        .arg("v2 rel")
        .assert()
        .success()
        .stdout(contains("ID:").and(contains("task-1")));
}

#[test]
fn show_does_not_resolve_body_text_as_task_reference() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "inbox",
        "task-1",
        "Ship v2 release",
        "# Ship v2 release\n\nLook at cost explorer.",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("show")
        .arg("cost explorer")
        .assert()
        .failure()
        .stderr(contains("task not found: cost explorer"));
}

#[test]
fn add_reads_tasks_root_from_config_file() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let tasks_root = temp.path().join("configured-tasks");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        format!("tasks_root = '{}'\n", tasks_root.display()),
    )
    .expect("config file should be written");

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    assert!(tasks_root.join("inbox").join("task-1.md").exists());
}

#[test]
fn add_uses_configured_queue_directory_names() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let tasks_root = temp.path().join("configured-tasks");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            "tasks_root = '{}'\n[queues]\ninbox = 'capture'\ndone = 'archive'\n",
            tasks_root.display()
        ),
    )
    .expect("config file should be written");

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    assert!(tasks_root.join("capture").join("task-1.md").exists());
    assert!(!tasks_root.join("inbox").join("task-1.md").exists());
}

#[test]
fn command_fails_cleanly_when_tasks_root_is_not_configured() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", temp.path())
        .env_remove("TQS_ROOT")
        .arg("list")
        .assert()
        .failure()
        .stderr(contains("missing tasks_root"));
}

#[test]
fn done_appends_completion_to_daily_note_when_configured() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let tasks_root = temp.path().join("tasks");
    let daily_notes_dir = temp.path().join("daily");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::write(
        config_dir.join("config.toml"),
        format!(
            "tasks_root = '{}'\ndaily_notes_dir = '{}'\n",
            tasks_root.display(),
            daily_notes_dir.display()
        ),
    )
    .expect("config file should be written");

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    let note_name = format!("{}.md", Local::now().format("%F"));
    let note = fs::read_to_string(daily_notes_dir.join(note_name)).expect("note should exist");
    assert!(note.contains("## Completed Tasks"));
    assert!(note.contains("- [x] Ship v2 (task-1)"));

    let task =
        fs::read_to_string(tasks_root.join("done").join("task-1.md")).expect("task should exist");
    assert!(task.contains(&format!("daily_note: {}.md", Local::now().format("%F"))));
}

#[test]
fn done_with_daily_notes_does_not_duplicate_note_entry() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let tasks_root = temp.path().join("tasks");
    let daily_notes_dir = temp.path().join("daily");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::write(
        config_dir.join("config.toml"),
        format!(
            "tasks_root = '{}'\ndaily_notes_dir = '{}'\n",
            tasks_root.display(),
            daily_notes_dir.display()
        ),
    )
    .expect("config file should be written");

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    cargo_bin_cmd!("tqs")
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("already done"));

    let note_name = format!("{}.md", Local::now().format("%F"));
    let note = fs::read_to_string(daily_notes_dir.join(note_name)).expect("note should exist");
    assert_eq!(note.matches("- [x] Ship v2 (task-1)").count(), 1);
}

#[test]
fn edit_updates_body_without_renaming_file() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "inbox",
        "task-1",
        "Ship v2",
        "# Ship v2\n\n## Notes\n\nOld body",
    );

    cargo_bin_cmd!("tqs")
        .env(
            "VISUAL",
            "sh -c 'cat <<\"EOF\" > \"$1\"\n---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Notes\n\nUpdated body\nEOF' sh",
        )
        .arg("--root")
        .arg(temp.path())
        .arg("edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Edited task: task-1"));

    let path = temp.path().join("inbox").join("task-1.md");
    assert!(path.exists());
    let content = fs::read_to_string(path).expect("task should exist");
    assert!(content.contains("Updated body"));
    assert!(!content.contains("Old body"));
}

#[test]
fn add_with_edit_persists_editor_changes() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .env(
            "VISUAL",
            "sh -c 'cat <<\"EOF\" > \"$1\"\n---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Notes\n\nEdited during add\nEOF' sh",
        )
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("--edit")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    let content =
        fs::read_to_string(temp.path().join("inbox").join("task-1.md")).expect("task should exist");
    assert!(content.contains("Edited during add"));
    assert!(!content.contains("## Context"));
}

#[test]
fn find_matches_tags_source_and_project() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task_with_metadata(
        temp.path(),
        "next",
        "task-1",
        "Investigate billing",
        "# Investigate billing",
        &["aws", "finance"],
        Some("email"),
        Some("platform-costs"),
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("find")
        .arg("finance")
        .assert()
        .success()
        .stdout(contains("task-1").and(contains("Investigate billing")));

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("find")
        .arg("email")
        .assert()
        .success()
        .stdout(contains("task-1").and(contains("Investigate billing")));

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("find")
        .arg("platform-costs")
        .assert()
        .success()
        .stdout(contains("task-1").and(contains("Investigate billing")));
}
