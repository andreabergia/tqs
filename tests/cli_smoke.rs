use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use chrono::Local;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;

fn tqs_cmd() -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.env("XDG_CONFIG_HOME", "/dev/null/tqs-test-config");
    cmd
}

fn write_task(root: &std::path::Path, queue: &str, id: &str, title: &str, body: &str) {
    let queue_dir = root.join(queue);
    fs::create_dir_all(&queue_dir).expect("queue dir should exist");
    fs::write(
        queue_dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: {title}\nqueue: {queue}\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n{body}"
        ),
    )
    .expect("task file should be written");
}

#[test]
fn help_command_works() {
    tqs_cmd().arg("--help").assert().success().stdout(
        contains("Terminal task queue")
            .and(contains("Task Commands:"))
            .and(contains("Workflow Commands:"))
            .and(contains("Setup Commands:"))
            .and(contains("Add a task"))
            .and(contains("List tasks"))
            .and(contains("Check configuration and task storage health"))
            .and(contains("Options:")),
    );
}

#[test]
fn bare_command_shows_getting_started_without_config() {
    tqs_cmd().assert().success().stdout(
        contains("Welcome to tqs!")
            .and(contains("To get started:"))
            .and(contains("tqs config")),
    );
}

#[test]
fn bare_command_shows_dashboard_when_tasks_exist() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Ship v2", "# Ship v2");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(contains("Ship v2").and(contains("Review PR")));
}

#[test]
fn bare_command_shows_getting_started_when_no_tasks() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(contains("Welcome to tqs!").and(contains("To get started:")));
}

#[test]
fn global_flag_is_rejected() {
    tqs_cmd()
        .arg("--global")
        .arg("list")
        .assert()
        .failure()
        .stderr(contains("unexpected argument '--global'"));
}

#[test]
fn add_creates_task_in_inbox() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    let path = temp.path().join("inbox").join("task-1.md");
    assert!(path.exists());
    let content = fs::read_to_string(path).expect("task should exist");
    assert!(content.contains("# Ship v2"));
    assert!(!content.contains("## Context"));
    assert!(!content.contains("## Notes"));
}

#[test]
fn add_generates_short_crockford_ids_and_persists_allocator_state() {
    let temp = TempDir::new().expect("temp dir should exist");

    let assert = tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("Ship v2")
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout is utf-8");
    let id = stdout
        .strip_prefix("Created task: ")
        .and_then(|value| value.split_once(" ("))
        .map(|(id, _)| id)
        .expect("created-task output should include the generated ID");

    assert_eq!(id.len(), 3);
    assert!(
        id.bytes()
            .all(|byte| b"0123456789abcdefghjkmnpqrstvwxyz".contains(&byte))
    );
    assert!(temp.path().join("inbox").join(format!("{id}.md")).exists());
}

#[test]
fn list_without_queue_shows_dashboard() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(
            contains("now (1)")
                .and(contains("inbox (1)"))
                .and(contains("Review PR")),
        );
}

#[test]
fn list_queue_shows_only_requested_queue() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("now")
        .assert()
        .success()
        .stdout(
            contains("now (1)")
                .and(contains("Do now"))
                .and(contains("Review PR").not()),
        );
}

#[test]
fn now_command_shows_only_now_queue() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("now")
        .assert()
        .success()
        .stdout(
            contains("now (1)")
                .and(contains("Do now"))
                .and(contains("Review PR").not()),
        );
}

#[test]
fn inbox_command_shows_only_inbox_queue() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "now", "task-1", "Do now", "# Do now");
    write_task(temp.path(), "inbox", "task-2", "Review PR", "# Review PR");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("inbox")
        .assert()
        .success()
        .stdout(
            contains("inbox (1)")
                .and(contains("Review PR"))
                .and(contains("Do now").not()),
        );
}

#[test]
fn move_relocates_file_to_target_queue() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    tqs_cmd()
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

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Review outage notes")
        .assert()
        .success();

    tqs_cmd()
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

    tqs_cmd()
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
fn move_prompts_for_queue_when_missing() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    tqs_cmd()
        .env("TQS_TEST_MODE", "1")
        .write_stdin("now\n")
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Moved task: task-1"));

    assert!(!temp.path().join("inbox").join("task-1.md").exists());
    assert!(temp.path().join("now").join("task-1.md").exists());
}

#[test]
fn done_is_idempotent() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("done")
        .arg("--no-edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("done")
        .arg("--no-edit")
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

    tqs_cmd()
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

    tqs_cmd()
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

    tqs_cmd()
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Ship v2")
        .assert()
        .failure()
        .stderr(contains("unrecognized subcommand"));

    tqs_cmd()
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

    tqs_cmd()
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

    tqs_cmd()
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

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--no-edit")
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

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--no-edit")
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

    tqs_cmd()
        .env("XDG_CONFIG_HOME", temp.path())
        .env_remove("TQS_ROOT")
        .arg("list")
        .assert()
        .failure()
        .stderr(
            contains("missing tasks_root")
                .and(contains("To get started:"))
                .and(contains("tqs --root ~/tasks add \"My first task\"")),
        );
}

#[test]
fn config_command_prints_getting_started_guide_when_tasks_root_is_missing() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .env_remove("TQS_ROOT")
        .arg("config")
        .assert()
        .success()
        .stdout(
            contains(format!(
                "config_path = {}",
                config_home.join("tqs").join("config.toml").display()
            ))
            .and(contains("config_file = missing"))
            .and(contains("tasks_root = <unset>"))
            .and(contains("To get started:"))
            .and(contains("tasks_root = \"~/tasks\"")),
        );
}

#[test]
fn config_command_prints_effective_values_from_root_override() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("--root")
        .arg(temp.path())
        .arg("config")
        .assert()
        .success()
        .stdout(
            contains(format!("tasks_root = {}", temp.path().display()))
                .and(contains("daily_notes_dir = <unset>"))
                .and(contains("queue.inbox = inbox"))
                .and(contains("queue.done = done")),
        );
}

#[test]
fn config_command_prints_configured_daily_notes_and_queue_dirs() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let tasks_root = temp.path().join("configured-tasks");
    let daily_notes_dir = temp.path().join("daily");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        format!(
            "tasks_root = '{}'\ndaily_notes_dir = '{}'\n[queues]\ninbox = 'capture'\nnow = 'focus'\ndone = 'archive'\n",
            tasks_root.display(),
            daily_notes_dir.display()
        ),
    )
    .expect("config file should be written");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("config")
        .assert()
        .success()
        .stdout(
            contains(format!("tasks_root = {}", tasks_root.display()))
                .and(contains(format!(
                    "daily_notes_dir = {}",
                    daily_notes_dir.display()
                )))
                .and(contains("queue.inbox = capture"))
                .and(contains("queue.now = focus"))
                .and(contains("queue.done = archive")),
        );
}

#[test]
fn config_command_prints_obsidian_vault_alias_and_derived_paths() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let vault_dir = temp.path().join("vault");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        format!("obsidian_vault_dir = '{}'\n", vault_dir.display()),
    )
    .expect("config file should be written");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("config")
        .assert()
        .success()
        .stdout(
            contains(format!("obsidian_vault_dir = {}", vault_dir.display()))
                .and(contains(format!(
                    "tasks_root = {}",
                    vault_dir.join("Tasks").display()
                )))
                .and(contains(format!(
                    "daily_notes_dir = {}",
                    vault_dir.join("Daily Notes").display()
                ))),
        );
}

#[test]
fn command_fails_when_obsidian_vault_alias_is_mixed_with_tasks_root() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        "obsidian_vault_dir = '/vault'\ntasks_root = '/tasks'\n",
    )
    .expect("config file should be written");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("config")
        .assert()
        .failure()
        .stderr(contains(
            "obsidian_vault_dir cannot be combined with tasks_root",
        ));
}

#[test]
fn config_command_respects_root_precedence() {
    let temp = TempDir::new().expect("temp dir should exist");
    let config_home = temp.path().join("config-home");
    let config_dir = config_home.join("tqs");
    let cli_root = temp.path().join("cli-root");
    let env_root = temp.path().join("env-root");
    let config_root = temp.path().join("config-root");
    std::fs::create_dir_all(&config_dir).expect("config dir should exist");
    std::fs::write(
        config_dir.join("config.toml"),
        format!("tasks_root = '{}'\n", config_root.display()),
    )
    .expect("config file should be written");

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .env("TQS_ROOT", &env_root)
        .arg("--root")
        .arg(&cli_root)
        .arg("config")
        .assert()
        .success()
        .stdout(
            contains(format!("tasks_root = {}", cli_root.display()))
                .and(contains(format!("tasks_root = {}", env_root.display())).not())
                .and(contains(format!("tasks_root = {}", config_root.display())).not()),
        );
}

#[test]
fn doctor_reports_clean_state_for_empty_root() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env("VISUAL", "sh")
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(
            contains("[ok] config: resolved tasks_root")
                .and(contains("[ok] editor: resolved command to 'sh'"))
                .and(contains("[ok] editor: executable 'sh' is available"))
                .and(contains("[ok] tasks_root:"))
                .and(contains("summary:")),
        );
}

#[test]
fn doctor_fails_when_it_finds_invalid_task_files() {
    let temp = TempDir::new().expect("temp dir should exist");
    let inbox = temp.path().join("inbox");
    fs::create_dir_all(&inbox).expect("inbox dir should exist");
    fs::write(
        inbox.join("bad.md"),
        "---\nid: bad\nqueue: inbox\n---\n# Missing required fields\n",
    )
    .expect("bad task should be written");

    tqs_cmd()
        .env("VISUAL", "sh")
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(contains("[error] tasks:").and(contains("bad.md is malformed")))
        .stderr(contains("doctor found 1 error(s)"));
}

#[test]
fn doctor_fails_when_task_queue_disagrees_with_directory() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(temp.path(), "inbox", "task-1", "Ship v2", "# Ship v2");
    let path = temp.path().join("inbox").join("task-1.md");
    let content = fs::read_to_string(&path).expect("task should exist");
    fs::write(&path, content.replace("queue: inbox", "queue: now"))
        .expect("task should be updated");

    tqs_cmd()
        .env("VISUAL", "sh")
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(contains("declares queue 'now' but is stored under 'inbox'"))
        .stderr(contains("doctor found 1 error(s)"));
}

#[test]
fn doctor_fails_when_editor_executable_is_missing() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env("VISUAL", "definitely-not-a-real-editor-tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(contains(
            "[error] editor: executable 'definitely-not-a-real-editor-tqs'",
        ))
        .stderr(contains("doctor found 1 error(s)"));
}

#[test]
fn doctor_fails_when_editor_command_is_invalid() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env("VISUAL", "\"unterminated")
        .arg("--root")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(contains("[error] editor: invalid editor command"))
        .stderr(contains("doctor found 1 error(s)"));
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

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("--no-edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    let note_name = format!("{}.md", Local::now().format("%F"));
    let note = fs::read_to_string(daily_notes_dir.join(note_name)).expect("note should exist");
    assert!(note.contains("## Completed Tasks"));
    assert!(note.contains("- [x] [[tasks/done/task-1|Ship v2]]"));

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

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("--no-edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("Completed task: task-1"));

    tqs_cmd()
        .env("XDG_CONFIG_HOME", &config_home)
        .arg("done")
        .arg("--no-edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("already done"));

    let note_name = format!("{}.md", Local::now().format("%F"));
    let note = fs::read_to_string(daily_notes_dir.join(note_name)).expect("note should exist");
    assert_eq!(
        note.matches("- [x] [[tasks/done/task-1|Ship v2]]").count(),
        1
    );
}

#[test]
fn edit_updates_body_without_renaming_file() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_task(
        temp.path(),
        "inbox",
        "task-1",
        "Ship v2",
        "# Ship v2\n\n## Context\n\n## Notes\n\nOld body",
    );

    tqs_cmd()
        .env(
            "VISUAL",
            "sh -c 'cat <<\"EOF\" > \"$1\"\n---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Context\n\n## Notes\n\nUpdated body\nEOF' sh",
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
fn edit_with_unchanged_file_preserves_updated_at() {
    let temp = TempDir::new().expect("temp dir should exist");
    let original = "---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Context\n\n## Notes\n\nOld body";
    fs::create_dir_all(temp.path().join("inbox")).expect("queue dir should exist");
    fs::write(temp.path().join("inbox").join("task-1.md"), original).expect("task should exist");

    tqs_cmd()
        .env("VISUAL", "sh -c 'touch \"$1\"' sh")
        .arg("--root")
        .arg(temp.path())
        .arg("edit")
        .arg("task-1")
        .assert()
        .success()
        .stdout(contains("No changes made: task-1"));

    let path = temp.path().join("inbox").join("task-1.md");
    let content = fs::read_to_string(path).expect("task should exist");
    assert_eq!(content, original);
    assert!(content.contains("updated_at: 2026-03-09T10:34:12Z"));
}

#[test]
fn add_with_edit_persists_editor_changes() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env(
            "VISUAL",
            "sh -c 'cat <<\"EOF\" > \"$1\"\n---\nid: task-1\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Context\n\n## Notes\n\nEdited during add\nEOF' sh",
        )
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    let content =
        fs::read_to_string(temp.path().join("inbox").join("task-1.md")).expect("task should exist");
    assert!(content.contains("Edited during add"));
    assert!(content.contains("## Context"));
}

#[test]
fn add_with_edit_rejects_empty_file_and_restores_stub() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env("VISUAL", "sh -c ': > \"$1\"' sh")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .failure()
        .stderr(contains("task file cannot be empty"))
        .stdout(contains("Created task: task-1").not());

    let path = temp.path().join("inbox").join("task-1.md");
    let content = fs::read_to_string(path).expect("task should exist");
    assert!(content.contains("id: task-1"));
    assert!(content.contains("# Ship v2"));
}

#[test]
fn add_with_edit_rejects_malformed_content_and_restores_stub() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env(
            "VISUAL",
            "sh -c 'printf -- \"---\\nid: task-1\\n\" > \"$1\"' sh",
        )
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .failure()
        .stderr(contains("invalid task file").and(contains("missing frontmatter end delimiter")))
        .stdout(contains("Created task: task-1").not());

    let path = temp.path().join("inbox").join("task-1.md");
    let content = fs::read_to_string(path).expect("task should exist");
    assert!(content.contains("id: task-1"));
    assert!(content.contains("# Ship v2"));
}

#[test]
fn add_with_edit_rejects_id_changes_and_restores_stub() {
    let temp = TempDir::new().expect("temp dir should exist");

    tqs_cmd()
        .env(
            "VISUAL",
            "sh -c 'cat <<\"EOF\" > \"$1\"\n---\nid: renamed\ntitle: Ship v2\nqueue: inbox\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ncompleted_at: null\ndaily_note: null\n---\n# Ship v2\n\n## Context\n\n## Notes\nEOF' sh",
        )
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .failure()
        .stderr(contains("editing a task cannot change its id"))
        .stdout(contains("Created task: task-1").not());

    let path = temp.path().join("inbox").join("task-1.md");
    let content = fs::read_to_string(path).expect("task should exist");
    assert!(content.contains("id: task-1"));
    assert!(!content.contains("id: renamed"));
    assert!(content.contains("# Ship v2"));
}
