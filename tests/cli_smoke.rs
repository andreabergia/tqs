use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

#[test]
fn help_command_works() {
    let mut cmd = cargo_bin_cmd!("tqs");
    cmd.arg("--help").assert().success().stdout(contains("tqs"));
}
