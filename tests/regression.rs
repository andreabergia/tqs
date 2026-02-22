use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

// ============================================================================
// Exit Code Tests
// ============================================================================

mod exit_code {
    use super::*;

    // Exit code 0: Success cases
    #[test]
    fn create_task_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Test task")
            .assert()
            .code(0);
    }

    #[test]
    fn list_tasks_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .code(0);
    }

    #[test]
    fn complete_task_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
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
            .code(0);
    }

    #[test]
    fn complete_already_closed_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("closed-task.md"),
            "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Already closed\n---\n",
        ).expect("closed task file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("closed-task")
            .assert()
            .code(0);
    }

    #[test]
    fn reopen_task_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("closed-task.md"),
            "---\nid: closed-task\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Closed task\n---\n",
        ).expect("closed task file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .arg("closed-task")
            .assert()
            .code(0);
    }

    #[test]
    fn reopen_already_open_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Open task")
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

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .arg(task_id)
            .assert()
            .code(0);
    }

    #[test]
    fn info_task_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
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

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg(task_id)
            .assert()
            .code(0);
    }

    #[test]
    fn delete_task_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
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

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("delete")
            .arg(task_id)
            .assert()
            .code(0);
    }

    #[test]
    fn list_no_matches_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
            .assert()
            .success();

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .arg("nonexistent")
            .assert()
            .code(0);
    }

    #[test]
    fn list_empty_repo_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .code(0);
    }

    #[test]
    fn complete_without_id_no_tasks_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .assert()
            .code(0);
    }

    #[test]
    fn reopen_without_id_no_tasks_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .assert()
            .code(0);
    }

    #[test]
    fn info_without_id_no_tasks_exits_with_0() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .assert()
            .code(0);
    }

    // Exit code 1: Operational errors
    #[test]
    fn complete_nonexistent_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("nonexistent")
            .assert()
            .code(1);
    }

    #[test]
    fn reopen_nonexistent_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .arg("nonexistent")
            .assert()
            .code(1);
    }

    #[test]
    fn info_nonexistent_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg("nonexistent")
            .assert()
            .code(1);
    }

    #[test]
    fn delete_nonexistent_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("delete")
            .arg("nonexistent")
            .assert()
            .code(1);
    }

    #[test]
    fn complete_without_id_non_tty_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
            .assert()
            .success();

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .assert()
            .code(1);
    }

    #[test]
    fn malformed_file_does_not_affect_info_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg("good")
            .assert()
            .success()
            .stdout(contains("ID: good"))
            .stdout(contains("Summary: Good task"));
    }

    #[test]
    fn info_without_id_non_tty_exits_with_1() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
            .assert()
            .success();

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .assert()
            .code(1);
    }

    // Exit code 2: Usage errors
    #[test]
    fn create_without_summary_exits_with_2() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("create")
            .assert()
            .code(2);
    }

    #[test]
    fn delete_without_id_exits_with_2() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("delete")
            .assert()
            .code(2);
    }

    #[test]
    fn invalid_global_flag_exits_with_2() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--invalid-flag")
            .arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .code(2);
    }

    #[test]
    fn invalid_command_exits_with_2() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("invalid-command")
            .assert()
            .code(2);
    }
}

// ============================================================================
// Stream Consistency Tests
// ============================================================================

mod stream_consistency {
    use super::*;

    #[test]
    fn success_messages_go_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Test task")
            .assert()
            .success()
            .stdout(contains("Created task:"))
            .stderr(contains("Created task:").not());
    }

    #[test]
    fn task_list_goes_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Test task")
            .assert()
            .success();

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stdout(contains("Test task"))
            .stderr(contains("Test task").not());
    }

    #[test]
    fn task_detail_goes_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Test task")
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

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg(task_id)
            .assert()
            .success()
            .stdout(contains("ID:"))
            .stdout(contains("Status:"))
            .stdout(contains("Summary:"))
            .stderr(contains("ID:").not());
    }

    #[test]
    fn error_messages_go_to_stderr() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg("nonexistent")
            .assert()
            .failure()
            .stderr(contains("task not found"))
            .stdout(contains("task not found").not());
    }

    #[test]
    fn not_found_errors_go_to_stderr() {
        let temp = TempDir::new().expect("temp dir should be created");

        let mut complete_cmd = cargo_bin_cmd!("tqs");
        complete_cmd
            .arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("nonexistent")
            .assert()
            .failure()
            .stderr(contains("task not found"))
            .stdout(contains("task not found").not());

        let mut reopen_cmd = cargo_bin_cmd!("tqs");
        reopen_cmd
            .arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .arg("nonexistent")
            .assert()
            .failure()
            .stderr(contains("task not found"))
            .stdout(contains("task not found").not());

        let mut delete_cmd = cargo_bin_cmd!("tqs");
        delete_cmd
            .arg("--root")
            .arg(temp.path())
            .arg("delete")
            .arg("nonexistent")
            .assert()
            .failure()
            .stderr(contains("task not found"))
            .stdout(contains("task not found").not());
    }

    #[test]
    fn usage_errors_go_to_stderr() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("create")
            .assert()
            .failure()
            .stderr(contains("summary is required"))
            .stdout(contains("summary is required").not());
    }

    #[test]
    fn no_tty_errors_go_to_stderr() {
        let temp = TempDir::new().expect("temp dir should be created");

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Task")
            .assert()
            .success();

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .assert()
            .failure()
            .stderr(contains("interactive selection requires a TTY"))
            .stdout(contains("interactive selection requires a TTY").not());
    }

    #[test]
    fn malformed_file_does_not_affect_complete_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("good")
            .assert()
            .success()
            .stdout(contains("Completed task:"));
    }

    #[test]
    fn idempotent_success_messages_go_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("closed.md"),
            "---\nid: closed\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Closed task\n---\n",
        ).expect("closed task file should be written");

        let mut complete_cmd = cargo_bin_cmd!("tqs");
        complete_cmd
            .arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("closed")
            .assert()
            .success()
            .stdout(contains("already closed"))
            .stderr(contains("already closed").not());

        cargo_bin_cmd!("tqs")
            .arg("--root")
            .arg(temp.path())
            .arg("create")
            .arg("Open task")
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
            .stdout(contains("already open"))
            .stderr(contains("already open").not());
    }

    #[test]
    fn empty_list_message_goes_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stdout(contains("No tasks found"))
            .stderr(contains("No tasks found").not());
    }

    #[test]
    fn no_tasks_available_message_goes_to_stdout() {
        let temp = TempDir::new().expect("temp dir should be created");
        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .assert()
            .success()
            .stdout(contains("No open tasks available"))
            .stderr(contains("No open tasks available").not());
    }
}

// ============================================================================
// Malformed File Tests
// ============================================================================

mod malformed_files {
    use super::*;

    #[test]
    fn missing_required_field_id() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("missing-id.md"),
            "---\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Missing ID\n---\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Missing ID").not());
    }

    #[test]
    fn missing_required_field_created_at() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("missing-time.md"),
            "---\nid: task-id\nstatus: open\nsummary: Missing timestamp\n---\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Missing timestamp").not());
    }

    #[test]
    fn missing_required_field_status() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("missing-status.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nsummary: Missing status\n---\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Missing status").not());
    }

    #[test]
    fn missing_required_field_summary() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("missing-summary.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\n---\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("task-id").not());
    }

    #[test]
    fn malformed_file_does_not_affect_delete_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("delete")
            .arg("good")
            .assert()
            .success()
            .stdout(contains("Deleted task:"));
    }

    #[test]
    fn invalid_timestamp_format() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("bad-time.md"),
            "---\nid: task-id\ncreated_at: not-a-date\nstatus: open\nsummary: Bad timestamp\n---\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Bad timestamp").not());
    }

    #[test]
    fn invalid_yaml_syntax() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("bad-yaml.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Task\ninvalid yaml: [unclosed\n---\n",
        ).expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Task").not());
    }

    #[test]
    fn invalid_status_value() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("bad-status.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: invalid-status\nsummary: Bad status\n---\n",
        ).expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("Bad status").not());
    }

    #[test]
    fn missing_frontmatter_separator() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("no-separator.md"),
            "id: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: No separator\n",
        )
        .expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"))
            .stdout(contains("No separator").not());
    }

    #[test]
    fn empty_file() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("empty.md"), "").expect("empty file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stderr(contains("Warning"));
    }

    #[test]
    fn only_frontmatter_no_body() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("no-body.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: No body\n---\n",
        ).expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stdout(contains("No body"));
    }

    #[test]
    fn malformed_file_does_not_affect_reopen_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("closed.md"),
            "---\nid: closed\ncreated_at: 2026-02-21T00:00:00Z\nstatus: closed\nsummary: Closed task\n---\n",
        ).expect("closed file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("reopen")
            .arg("closed")
            .assert()
            .success()
            .stdout(contains("Reopened task:"));
    }

    #[test]
    fn extra_unknown_fields_are_ignored() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(
            temp.path().join("extra-fields.md"),
            "---\nid: task-id\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Extra fields\nunknown1: value1\nunknown2: value2\npriority: high\ntags: [urgent, important]\n---\n",
        ).expect("file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stdout(contains("Extra fields"));
    }

    #[test]
    fn malformed_file_with_valid_file_list_shows_only_valid() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("list")
            .assert()
            .success()
            .stdout(contains("Good task"))
            .stdout(contains("not valid markdown").not())
            .stderr(contains("Warning"));
    }

    #[test]
    fn malformed_file_does_not_affect_complete_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("complete")
            .arg("good")
            .assert()
            .success()
            .stdout(contains("Completed task:"));
    }

    #[test]
    fn malformed_file_does_not_affect_info_command() {
        let temp = TempDir::new().expect("temp dir should be created");

        std::fs::write(temp.path().join("bad.md"), "not valid markdown")
            .expect("bad file should be written");

        std::fs::write(
            temp.path().join("good.md"),
            "---\nid: good\ncreated_at: 2026-02-21T00:00:00Z\nstatus: open\nsummary: Good task\n---\n",
        ).expect("good file should be written");

        let mut cmd = cargo_bin_cmd!("tqs");
        cmd.arg("--root")
            .arg(temp.path())
            .arg("info")
            .arg("good")
            .assert()
            .success()
            .stdout(contains("ID: good"))
            .stdout(contains("Summary: Good task"));
    }
}
