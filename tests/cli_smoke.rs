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
fn create_without_summary_fails() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--root")
        .arg(temp.path())
        .arg("create")
        .assert()
        .failure()
        .stderr(contains("summary is required"));
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
    let task_id = stdout.lines().next().unwrap().split('\t').next().unwrap();

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
