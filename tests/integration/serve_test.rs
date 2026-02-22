use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_serve_without_mcp_flag_fails() {
    Command::cargo_bin("authy")
        .unwrap()
        .arg("serve")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --mcp"));
}

#[test]
fn test_serve_appears_in_help() {
    Command::cargo_bin("authy")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("serve"));
}
