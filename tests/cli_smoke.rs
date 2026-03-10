use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn write_task(root: &std::path::Path, queue: &str, id: &str, title: &str, body: &str) {
    let queue_dir = root.join(queue);
    std::fs::create_dir_all(&queue_dir).expect("queue dir should exist");
    std::fs::write(
        queue_dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: {title}\nqueue: {queue}\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\ntags: []\nsource: null\nproject: null\ncompleted_at: null\ndaily_note: null\n---\n{body}"
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
