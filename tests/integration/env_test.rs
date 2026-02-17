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

    for (name, val) in [("db-host", "localhost"), ("db-port", "5432"), ("api-key", "sk-test")] {
        authy_cmd(home)
            .args(["store", name])
            .write_stdin(val)
            .assert()
            .success();
    }

    authy_cmd(home)
        .args(["policy", "create", "agent", "--allow", "*"])
        .assert()
        .success();
}

#[test]
fn test_env_shell_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["env", "--scope", "agent", "--format", "shell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("export api-key='sk-test'"))
        .stdout(predicate::str::contains("export db-host='localhost'"))
        .stdout(predicate::str::contains("export db-port='5432'"));
}

#[test]
fn test_env_shell_no_export() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["env", "--scope", "agent", "--format", "shell", "--no-export"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("export "));
    assert!(stdout.contains("db-host='localhost'"));
}

#[test]
fn test_env_dotenv_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["env", "--scope", "agent", "--format", "dotenv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api-key=sk-test"))
        .stdout(predicate::str::contains("db-host=localhost"));
}

#[test]
fn test_env_json_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["env", "--scope", "agent", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["api-key"], "sk-test");
    assert_eq!(json["db-host"], "localhost");
    assert_eq!(json["db-port"], "5432");
}

#[test]
fn test_env_uppercase_and_replace_dash() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args([
            "env", "--scope", "agent",
            "--format", "shell",
            "--uppercase", "--replace-dash", "_",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("export DB_HOST='localhost'"))
        .stdout(predicate::str::contains("export DB_PORT='5432'"))
        .stdout(predicate::str::contains("export API_KEY='sk-test'"));
}

#[test]
fn test_env_with_prefix() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args([
            "env", "--scope", "agent",
            "--format", "shell",
            "--prefix", "APP_",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("export APP_db-host='localhost'"));
}

#[test]
fn test_env_with_token_auth() {
    let home = TempDir::new().unwrap();
    let keyfile = home.path().join("test.key");

    // Init with keyfile
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--generate-keyfile", keyfile.to_str().unwrap()])
        .assert()
        .success();

    // Store a secret
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env_remove("AUTHY_TOKEN")
        .env_remove("AUTHY_PASSPHRASE")
        .args(["store", "my-secret"])
        .write_stdin("myval")
        .assert()
        .success();

    // Create policy
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env_remove("AUTHY_TOKEN")
        .env_remove("AUTHY_PASSPHRASE")
        .args(["policy", "create", "test-scope", "--allow", "*"])
        .assert()
        .success();

    // Create session token
    let output = Command::cargo_bin("authy")
        .unwrap()
        .env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env_remove("AUTHY_TOKEN")
        .env_remove("AUTHY_PASSPHRASE")
        .args(["session", "create", "--scope", "test-scope"])
        .output()
        .unwrap();
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Use token to env
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env("AUTHY_TOKEN", &token)
        .env_remove("AUTHY_PASSPHRASE")
        .args(["env", "--scope", "test-scope", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-secret"));
}
