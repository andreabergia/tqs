use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn write_raw_task(
    root: &std::path::Path,
    queue: &str,
    id: &str,
    frontmatter_tail: &str,
    body: &str,
) {
    let queue_dir = root.join(queue);
    std::fs::create_dir_all(&queue_dir).expect("queue dir should exist");
    std::fs::write(
        queue_dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: Test task\nqueue: {queue}\ncreated_at: 2026-03-09T10:34:12Z\nupdated_at: 2026-03-09T10:34:12Z\n{frontmatter_tail}---\n{body}"
        ),
    )
    .expect("task file should be written");
}

#[test]
fn invalid_queue_is_rejected_cleanly() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("archive")
        .assert()
        .failure()
        .stderr(contains("invalid queue 'archive'"));
}

#[test]
fn malformed_files_are_skipped_during_list() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_raw_task(
        temp.path(),
        "inbox",
        "good",
        "tags: []\ncompleted_at: null\ndaily_note: null\n",
        "# Good task",
    );
    write_raw_task(
        temp.path(),
        "inbox",
        "bad",
        "updated_at: not-a-date\n",
        "# Bad task",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("inbox")
        .assert()
        .success()
        .stdout(contains("good").and(contains("bad").not()))
        .stderr(contains("Warning: skipping malformed task file"));
}

#[test]
fn done_moves_file_and_sets_completed_at() {
    let temp = TempDir::new().expect("temp dir should exist");
    write_raw_task(
        temp.path(),
        "inbox",
        "task-1",
        "tags: []\ncompleted_at: null\ndaily_note: null\n",
        "# Task",
    );

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("done")
        .arg("task-1")
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join("done").join("task-1.md"))
        .expect("done file should exist");
    assert!(content.contains("queue: done"));
    assert!(content.contains("completed_at:"));
    assert!(!content.contains("completed_at: null"));
}

#[test]
fn add_with_explicit_done_queue_sets_completed_at() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("--queue")
        .arg("done")
        .arg("Ship v2")
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join("done").join("task-1.md"))
        .expect("done file should exist");
    assert!(content.contains("completed_at:"));
    assert!(!content.contains("completed_at: null"));
}

#[test]
fn add_omits_removed_metadata_fields() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship v2")
        .assert()
        .success();

    let content = std::fs::read_to_string(temp.path().join("inbox").join("task-1.md"))
        .expect("task file should exist");
    assert!(!content.contains("source:"));
    assert!(!content.contains("project:"));
}

#[test]
fn ambiguous_task_reference_is_reported_cleanly_without_tty() {
    let temp = TempDir::new().expect("temp dir should exist");
    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-1")
        .arg("Ship release")
        .assert()
        .success();
    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--no-edit")
        .arg("--id")
        .arg("task-2")
        .arg("Ship release prep")
        .assert()
        .success();

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("show")
        .arg("Ship release")
        .assert()
        .failure()
        .stderr(contains("task reference is ambiguous: Ship release"));
}

#[test]
fn add_with_content_sets_body_and_skips_editor() {
    let temp = TempDir::new().expect("temp dir should exist");

    cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("add")
        .arg("--id")
        .arg("task-1")
        .arg("--content")
        .arg("Some details here")
        .arg("Ship v2")
        .assert()
        .success()
        .stdout(contains("Created task: task-1"));

    let content = std::fs::read_to_string(temp.path().join("inbox").join("task-1.md"))
        .expect("task file should exist");
    assert!(content.contains("# Ship v2"));
    assert!(content.contains("Some details here"));
}
