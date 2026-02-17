use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn init_vault(home: &TempDir) {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();
}

#[test]
fn test_noninteractive_fails_without_credentials() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Without any credentials + AUTHY_NON_INTERACTIVE=1, should fail immediately
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_NON_INTERACTIVE", "1")
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No credentials provided"))
        .stderr(predicate::str::contains("AUTHY_KEYFILE"));
}

#[test]
fn test_noninteractive_works_with_credentials() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // With credentials + AUTHY_NON_INTERACTIVE=1, should work fine
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_NON_INTERACTIVE", "1")
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["list"])
        .assert()
        .success();
}

#[test]
fn test_noninteractive_exit_code() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let output = Command::cargo_bin("authy")
        .unwrap()
        .env("HOME", home.path())
        .env("AUTHY_NON_INTERACTIVE", "1")
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["list"])
        .output()
        .unwrap();

    // AuthFailed maps to exit code 2
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_noninteractive_error_message_helpful() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_NON_INTERACTIVE", "1")
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["get", "my-secret"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("AUTHY_KEYFILE"))
        .stderr(predicate::str::contains("AUTHY_PASSPHRASE"))
        .stderr(predicate::str::contains("AUTHY_TOKEN"));
}

#[test]
fn test_store_works_with_piped_stdin() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Store should still work with piped stdin (reads value from stdin)
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["store", "test-secret"])
        .write_stdin("my-value")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["get", "test-secret"])
        .assert()
        .success()
        .stdout("my-value");
}

#[test]
fn test_rotate_works_with_piped_stdin() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Store initial
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["store", "rotatable"])
        .write_stdin("v1")
        .assert()
        .success();

    // Rotate should work with piped stdin
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["rotate", "rotatable"])
        .write_stdin("v2")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["get", "rotatable"])
        .assert()
        .success()
        .stdout("v2");
}
