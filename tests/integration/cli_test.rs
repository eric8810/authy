use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn authy_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path());
    cmd.env_remove("AUTHY_PASSPHRASE");
    cmd.env_remove("AUTHY_KEYFILE");
    cmd.env_remove("AUTHY_TOKEN");
    cmd
}

#[test]
fn test_help() {
    let home = TempDir::new().unwrap();
    authy_cmd(&home)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("secrets store"));
}

#[test]
fn test_version() {
    let home = TempDir::new().unwrap();
    authy_cmd(&home)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("authy"));
}

#[test]
fn test_commands_without_init_fail() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["list"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized"));
}

#[test]
fn test_init_with_keyfile() {
    let home = TempDir::new().unwrap();
    let keyfile = home.path().join("test.key");

    authy_cmd(&home)
        .args(["init", "--generate-keyfile", keyfile.to_str().unwrap()])
        .assert()
        .success();

    assert!(keyfile.exists());
    assert!(home.path().join(format!("{}.pub", keyfile.to_str().unwrap())).exists());
    assert!(home.path().join(".authy/vault.age").exists());

    // Should be able to use the keyfile
    authy_cmd(&home)
        .args(["store", "test"])
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .write_stdin("value")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "test"])
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .assert()
        .success()
        .stdout("value");
}

#[test]
fn test_config_show() {
    let home = TempDir::new().unwrap();
    authy_cmd(&home)
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth_method"));
}
