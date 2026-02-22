use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn help_command_works() {
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--help").assert().success().stdout(contains("tqs"));
}

#[test]
fn create_task_with_summary() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Buy groceries")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn create_task_with_description() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Write documentation")
        .arg("--description")
        .arg("Write user guide and API docs")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn create_task_with_explicit_id() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("--id")
        .arg("my-custom-task-id")
        .arg("Buy groceries")
        .assert()
        .success()
        .stdout(contains("Created task: my-custom-task-id"));
}

#[test]
fn create_task_with_existing_id_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("--id")
        .arg("duplicate-id")
        .arg("First task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("--id")
        .arg("duplicate-id")
        .arg("Second task")
        .assert()
        .failure()
        .stderr(contains("id 'duplicate-id' already exists"));
}

#[test]
fn create_without_summary_or_description_in_non_tty_fails() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("create")
        .assert()
        .failure()
        .stderr(contains("interactive input requires a TTY"));
}

#[test]
fn create_generates_unique_id() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task one")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task two")
        .assert()
        .success();

    let entries = std::fs::read_dir(temp.path()).expect("should read directory");
    let md_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();

    assert_eq!(md_files.len(), 2, "should create two task files");
}

#[test]
fn create_interactive_with_summary_only() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.env("TQS_TEST_MODE", "1")
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .write_stdin("Test task\n\n")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn create_interactive_with_summary_and_description() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.env("TQS_TEST_MODE", "1")
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .write_stdin("Write docs\nUser guide\nAPI docs\n")
        .assert()
        .success()
        .stdout(contains("Created task:"));

    let mut info_cmd = cargo_bin_cmd!("tqs");
    let output = info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let info_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .arg(task_id)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let info_text = String::from_utf8_lossy(&info_output);
    assert!(info_text.contains("User guide"));
    assert!(info_text.contains("API docs"));
}

#[test]
fn create_interactive_empty_description() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.env("TQS_TEST_MODE", "1")
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .write_stdin("Task\n\n")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn create_interactive_whitespace_only_description() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.env("TQS_TEST_MODE", "1")
        .arg("--root")
        .arg(temp.path())
        .arg("create")
        .write_stdin("Task\n   \n  \n")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn list_shows_open_tasks_by_default() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Open task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Another open task")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(contains("Open task"))
        .stdout(contains("Another open task"));
}

#[test]
fn list_with_all_shows_all_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Open task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to complete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg(task_id)
        .assert()
        .success();

    let mut list_all_cmd = cargo_bin_cmd!("tqs");
    list_all_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("--all")
        .assert()
        .success()
        .stdout(contains("Open task"))
        .stdout(contains("Task to complete"));
}

#[test]
fn list_with_closed_shows_closed_tasks_only() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Keep open")
        .assert()
        .success();

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Task to close\n---\n",
    ).expect("closed task file should be written");

    let mut list_closed_cmd = cargo_bin_cmd!("tqs");
    list_closed_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("--closed")
        .assert()
        .success()
        .stdout(contains("Task to close"))
        .stdout(contains("No tasks found").not());
}

#[test]
fn list_with_verbose_shows_more_columns() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Test task")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(contains("open"))
        .stdout(contains("UTC"));
}

#[test]
fn list_with_keywords_filters_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Buy groceries")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Write code")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("buy")
        .assert()
        .success()
        .stdout(contains("groceries"))
        .stdout(contains("code").not());
}

#[test]
fn list_with_multiple_keywords_uses_and_semantics() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Buy groceries")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Write code")
        .assert()
        .success();

    let mut cmd3 = cargo_bin_cmd!("tqs");
    cmd3.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Buy code")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("buy")
        .arg("code")
        .assert()
        .success()
        .stdout(contains("Buy code"))
        .stdout(contains("Buy groceries").not())
        .stdout(contains("Write code").not());
}

#[test]
fn list_with_no_matches_prints_no_tasks_found() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Buy groceries")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .arg("nonexistent")
        .assert()
        .success()
        .stdout(contains("No tasks found"));
}

#[test]
fn list_empty_repository_prints_no_tasks_found() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(contains("No tasks found"));
}

#[test]
fn list_skips_malformed_files_with_warning() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(temp.path().join("bad.md"), "not valid markdown")
        .expect("bad file should be written");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Valid task")
        .assert()
        .success();

    let mut list_cmd = cargo_bin_cmd!("tqs");
    list_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(contains("Valid task"))
        .stderr(contains("Warning"));
}

#[test]
fn complete_task_by_id() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to complete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("Completed task:"));
}

#[test]
fn complete_already_closed_task() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Already closed\n---\n",
    ).expect("closed task file should be written");

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg("closed-task")
        .assert()
        .success()
        .stdout(contains("already closed"));
}

#[test]
fn complete_nonexistent_task_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg("nonexistent-task")
        .assert()
        .failure()
        .stderr(contains("task not found"));
}

#[test]
fn complete_without_id_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .assert()
        .success()
        .stdout(contains("No open tasks available"));
}

#[test]
fn reopen_task_by_id() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Task to reopen\n---\n",
    ).expect("closed task file should be written");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .arg("closed-task")
        .assert()
        .success()
        .stdout(contains("Reopened task:"));
}

#[test]
fn reopen_already_open_task() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Already open")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("already open"));
}

#[test]
fn reopen_nonexistent_task_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .arg("nonexistent-task")
        .assert()
        .failure()
        .stderr(contains("task not found"));
}

#[test]
fn reopen_without_id_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .assert()
        .success()
        .stdout(contains("No closed tasks available"));
}

#[test]
fn info_task_by_id() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to view")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("ID:"))
        .stdout(contains("Status:"))
        .stdout(contains("Created at:"))
        .stdout(contains("Summary: Task to view"));
}

#[test]
fn info_task_with_description() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task with description")
        .arg("--description")
        .arg("# Notes\n- First item\n- Second item")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("# Notes"))
        .stdout(contains("First item"))
        .stdout(contains("Second item"));
}

#[test]
fn info_nonexistent_task_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .arg("nonexistent-task")
        .assert()
        .failure()
        .stderr(contains("task not found"));
}

#[test]
fn info_without_id_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .assert()
        .success()
        .stdout(contains("No tasks available"));
}

#[test]
fn delete_task_by_id() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to delete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut delete_cmd = cargo_bin_cmd!("tqs");
    delete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("delete")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("Deleted task:"));
}

#[test]
fn delete_nonexistent_task_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut delete_cmd = cargo_bin_cmd!("tqs");
    delete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("delete")
        .arg("nonexistent-task")
        .assert()
        .failure()
        .stderr(contains("task not found"));
}

#[test]
fn delete_removes_file() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to delete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut delete_cmd = cargo_bin_cmd!("tqs");
    delete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("delete")
        .arg(task_id)
        .assert()
        .success();

    let task_file = temp.path().join(format!("{task_id}.md"));
    assert!(!task_file.exists(), "task file should be removed");
}

#[test]
fn complete_without_id_in_non_tty_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to complete")
        .assert()
        .success();

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .assert()
        .failure()
        .stderr(contains("interactive input requires a TTY"));
}

#[test]
fn complete_picker_with_open_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("First task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Second task")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();

    let first_task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .arg(first_task_id)
        .assert()
        .success()
        .stdout(contains("Completed task:"));
}

#[test]
fn complete_picker_with_no_open_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("complete")
        .assert()
        .success()
        .stdout(contains("No open tasks available"));
}

#[test]
fn reopen_without_id_in_non_tty_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Task to reopen\n---\n",
    ).expect("closed task file should be written");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .assert()
        .failure()
        .stderr(contains("interactive input requires a TTY"));
}

#[test]
fn reopen_picker_with_closed_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Task to reopen\n---\n",
    ).expect("closed task file should be written");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .arg("closed-task")
        .assert()
        .success()
        .stdout(contains("Reopened task:"));
}

#[test]
fn reopen_picker_with_no_closed_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Open task")
        .assert()
        .success();

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("reopen")
        .assert()
        .success()
        .stdout(contains("No closed tasks available"));
}

#[test]
fn info_without_id_in_non_tty_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to view")
        .assert()
        .success();

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .assert()
        .failure()
        .stderr(contains("interactive input requires a TTY"));
}

#[test]
fn info_picker_with_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to view")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("ID:"))
        .stdout(contains("Status:"))
        .stdout(contains("Summary: Task to view"));
}

#[test]
fn info_picker_with_no_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("info")
        .assert()
        .success()
        .stdout(contains("No tasks available"));
}

#[test]
fn test_fuzzy_command_cre() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Test task with fuzzy")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn test_fuzzy_command_l() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Task to list")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("l")
        .assert()
        .success()
        .stdout(contains("Task to list"));
}

#[test]
fn test_fuzzy_command_l_with_flags() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Open task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("l")
        .arg("--all")
        .assert()
        .success()
        .stdout(contains("Open task"));
}

#[test]
fn test_fuzzy_command_c() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("c")
        .arg("Task with fuzzy c")
        .assert()
        .success()
        .stdout(contains("Created task:"));
}

#[test]
fn test_fuzzy_command_with_keywords() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Bug fix task")
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("tqs");
    cmd2.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Feature task")
        .assert()
        .success();

    let mut cmd3 = cargo_bin_cmd!("tqs");
    cmd3.arg("--root")
        .arg(temp.path())
        .arg("l")
        .arg("bug")
        .assert()
        .success()
        .stdout(contains("Bug fix task"))
        .stdout(contains("Feature task").not());
}

#[test]
fn test_fuzzy_command_inf() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Task for info")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("l")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut info_cmd = cargo_bin_cmd!("tqs");
    info_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("inf")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("ID:"))
        .stdout(contains("Summary: Task for info"));
}

#[test]
fn test_fuzzy_command_cmp() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Task to complete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("l")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut complete_cmd = cargo_bin_cmd!("tqs");
    complete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("cmp")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("Completed task:"));
}

#[test]
fn test_fuzzy_command_opn() {
    let temp = TempDir::new().expect("temp dir should be created");

    std::fs::write(
        temp.path().join("closed-task.md"),
        "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Task to reopen\n---\n",
    ).expect("closed task file should be written");

    let mut reopen_cmd = cargo_bin_cmd!("tqs");
    reopen_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("opn")
        .arg("closed-task")
        .assert()
        .success()
        .stdout(contains("Reopened task:"));
}

#[test]
fn test_fuzzy_command_del() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("cr")
        .arg("Task to delete")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("l")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut delete_cmd = cargo_bin_cmd!("tqs");
    delete_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("del")
        .arg(task_id)
        .assert()
        .success()
        .stdout(contains("Deleted task:"));
}

#[test]
fn move_without_args_in_non_tty_fails() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to move")
        .assert()
        .success();

    let mut move_cmd = cargo_bin_cmd!("tqs");
    move_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .assert()
        .failure()
        .stderr(contains("interactive input requires a TTY"));
}

#[test]
fn move_picker_with_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Task to move")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();

    let old_task_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut move_cmd = cargo_bin_cmd!("tqs");
    move_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg(old_task_id)
        .arg("new-task-id")
        .assert()
        .success()
        .stdout(contains("Moved task:"));
}

#[test]
fn move_picker_with_no_tasks() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut move_cmd = cargo_bin_cmd!("tqs");
    move_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .assert()
        .success()
        .stdout(contains("No tasks available"));
}

#[test]
fn move_with_both_ids() {
    let temp = TempDir::new().expect("temp dir should be created");

    let mut cmd1 = cargo_bin_cmd!("tqs");
    cmd1.arg("--root")
        .arg(temp.path())
        .arg("create")
        .arg("Original task")
        .assert()
        .success();

    let list_output = cargo_bin_cmd!("tqs")
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&list_output);
    let lines: Vec<&str> = stdout.lines().collect();
    let old_id = lines.get(2).unwrap().split_whitespace().next().unwrap();

    let mut move_cmd = cargo_bin_cmd!("tqs");
    move_cmd
        .arg("--root")
        .arg(temp.path())
        .arg("move")
        .arg(old_id)
        .arg("renamed-task")
        .assert()
        .success()
        .stdout(contains("Moved task:"));

    let mut list_cmd2 = cargo_bin_cmd!("tqs");
    list_cmd2
        .arg("--root")
        .arg(temp.path())
        .arg("list")
        .assert()
        .success()
        .stdout(contains("renamed-task"));
}
