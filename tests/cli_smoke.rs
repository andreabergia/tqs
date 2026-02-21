use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
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
