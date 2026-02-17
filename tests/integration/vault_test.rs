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

fn init_vault(home: &TempDir) {
    authy_cmd(home)
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();
}

#[test]
fn test_init_creates_vault() {
    let home = TempDir::new().unwrap();
    init_vault(&home);
    assert!(home.path().join(".authy/vault.age").exists());
    assert!(home.path().join(".authy/authy.toml").exists());
}

#[test]
fn test_init_twice_fails() {
    let home = TempDir::new().unwrap();
    init_vault(&home);
    authy_cmd(&home)
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already initialized"));
}

#[test]
fn test_store_and_get() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "my-secret"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("secret123")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "my-secret"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .success()
        .stdout("secret123");
}

#[test]
fn test_list_secrets() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "alpha"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("val1")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["store", "beta"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("val2")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["list"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn test_remove_secret() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "to-remove"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("val")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["remove", "to-remove"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "to-remove"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_rotate_secret() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "rotating"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v1")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["rotate", "rotating"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v2")
        .assert()
        .success()
        .stderr(predicate::str::contains("version 2"));

    authy_cmd(&home)
        .args(["get", "rotating"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .success()
        .stdout("v2");
}

#[test]
fn test_store_duplicate_fails() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "dup"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v1")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["store", "dup"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v2")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_store_force_overwrite() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "dup"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v1")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["store", "dup", "--force"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .write_stdin("v2")
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "dup"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .success()
        .stdout("v2");
}

#[test]
fn test_get_nonexistent_fails() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["get", "nope"])
        .env("AUTHY_PASSPHRASE", "testpass")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wrong_passphrase_fails() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["list"])
        .env("AUTHY_PASSPHRASE", "wrongpass")
        .assert()
        .failure();
}
