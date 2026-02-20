use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_command_works() {
    let mut cmd = Command::cargo_bin("tqs").expect("binary should build");
    cmd.arg("--help").assert().success().stdout(contains("tqs"));
}
