use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn authy_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path());
    cmd.env("AUTHY_PASSPHRASE", "testpass");
    cmd.env_remove("AUTHY_KEYFILE");
    cmd.env_remove("AUTHY_TOKEN");
    cmd
}

fn setup(home: &TempDir) {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();

    for (name, val) in [("db-host", "localhost"), ("db-port", "5432"), ("ssh-key", "mykey")] {
        authy_cmd(home)
            .args(["store", name])
            .write_stdin(val)
            .assert()
            .success();
    }

    authy_cmd(home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();
}

#[test]
fn test_run_injects_env_vars() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["run", "--scope", "deploy", "--", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("db-host=localhost"))
        .stdout(predicate::str::contains("db-port=5432"))
        .stdout(predicate::str::contains("ssh-key").not());
}

#[test]
fn test_run_uppercase() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["run", "--scope", "deploy", "--uppercase", "--", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DB-HOST=localhost"));
}

#[test]
fn test_run_replace_dash() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["run", "--scope", "deploy", "--replace-dash", "_", "--", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("db_host=localhost"));
}

#[test]
fn test_run_uppercase_and_replace_dash() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args([
            "run", "--scope", "deploy",
            "--uppercase", "--replace-dash", "_",
            "--", "env",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("DB_HOST=localhost"));
}

#[test]
fn test_run_with_prefix() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args([
            "run", "--scope", "deploy",
            "--prefix", "APP_",
            "--", "env",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("APP_db-host=localhost"));
}

#[test]
fn test_run_does_not_leak_authy_passphrase() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["run", "--scope", "deploy", "--", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AUTHY_PASSPHRASE").not());
}
