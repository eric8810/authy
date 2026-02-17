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

fn setup_vault_with_keyfile(home: &TempDir) -> String {
    let keyfile = home.path().join("test.key");
    let keyfile_str = keyfile.to_str().unwrap().to_string();

    authy_cmd(home)
        .args(["init", "--generate-keyfile", &keyfile_str])
        .assert()
        .success();

    // Store some secrets
    authy_cmd(home)
        .args(["store", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .write_stdin("localhost")
        .assert()
        .success();

    authy_cmd(home)
        .args(["store", "db-password"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .write_stdin("secret123")
        .assert()
        .success();

    authy_cmd(home)
        .args(["store", "ssh-key"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .write_stdin("my-ssh-key")
        .assert()
        .success();

    // Create a policy
    authy_cmd(home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .assert()
        .success();

    keyfile_str
}

#[test]
fn test_session_create_and_use() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault_with_keyfile(&home);

    // Create a session token
    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "deploy", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();

    assert!(output.status.success());
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(token.starts_with("authy_v1."));

    // Use the token to get an allowed secret
    authy_cmd(&home)
        .args(["get", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .success()
        .stdout("localhost");

    // Token should enforce scope — denied secret
    authy_cmd(&home)
        .args(["get", "ssh-key"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .stderr(predicate::str::contains("denied"));
}

#[test]
fn test_session_token_is_read_only() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault_with_keyfile(&home);

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "deploy", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();

    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Try to store with token — should fail
    authy_cmd(&home)
        .args(["store", "new-secret"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .write_stdin("val")
        .assert()
        .failure()
        .stderr(predicate::str::contains("read-only"));
}

#[test]
fn test_session_list() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault_with_keyfile(&home);

    authy_cmd(&home)
        .args(["session", "create", "--scope", "deploy", "--ttl", "1h", "--label", "ci-runner"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    authy_cmd(&home)
        .args(["session", "list"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("deploy"))
        .stdout(predicate::str::contains("active"))
        .stdout(predicate::str::contains("ci-runner"));
}

#[test]
fn test_session_revoke() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault_with_keyfile(&home);

    // Create token
    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "deploy", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();

    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Get the session ID from the list
    let list_output = authy_cmd(&home)
        .args(["session", "list"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();

    let list_str = String::from_utf8(list_output.stdout).unwrap();
    let session_id = list_str.split_whitespace().next().unwrap().to_string();

    // Revoke it
    authy_cmd(&home)
        .args(["session", "revoke", &session_id])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Token should no longer work
    authy_cmd(&home)
        .args(["get", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure();
}

#[test]
fn test_invalid_token_rejected() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault_with_keyfile(&home);

    authy_cmd(&home)
        .args(["get", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", "authy_v1.invalid_token_data")
        .assert()
        .failure();
}
