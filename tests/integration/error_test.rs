use assert_cmd::Command;
use tempfile::TempDir;

fn authy_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path());
    cmd.env("AUTHY_PASSPHRASE", "testpass");
    cmd.env_remove("AUTHY_KEYFILE");
    cmd.env_remove("AUTHY_TOKEN");
    cmd
}

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
fn test_exit_code_vault_not_initialized() {
    let home = TempDir::new().unwrap();
    let output = authy_cmd(&home)
        .args(["list"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7)); // VaultNotInitialized
}

#[test]
fn test_exit_code_secret_not_found() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let output = authy_cmd(&home)
        .args(["get", "nonexistent"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3)); // SecretNotFound
}

#[test]
fn test_exit_code_already_exists() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "dup"])
        .write_stdin("v1")
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["store", "dup"])
        .write_stdin("v2")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5)); // SecretAlreadyExists
}

#[test]
fn test_exit_code_policy_not_found() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let output = authy_cmd(&home)
        .args(["policy", "show", "nonexistent"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3)); // PolicyNotFound
}

#[test]
fn test_exit_code_access_denied() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "secret-a"])
        .write_stdin("val")
        .assert()
        .success();

    // Create restrictive policy
    authy_cmd(&home)
        .args(["policy", "create", "restricted", "--allow", "other-*"])
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["get", "secret-a", "--scope", "restricted"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4)); // AccessDenied
}

#[test]
fn test_exit_code_success() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["store", "test-secret"])
        .write_stdin("val")
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["get", "test-secret"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exit_code_vault_already_exists() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let output = authy_cmd(&home)
        .args(["init", "--passphrase", "testpass"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(5)); // VaultAlreadyExists
}

#[test]
fn test_run_forwards_child_exit_code() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "test-scope", "--allow", "*"])
        .assert()
        .success();

    // Run a command that exits with code 42
    let output = authy_cmd(&home)
        .args(["run", "--scope", "test-scope", "--", "bash", "-c", "exit 42"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(42));
}
