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

fn setup_vault(home: &TempDir) -> String {
    let keyfile = home.path().join("test.key");
    let keyfile_str = keyfile.to_str().unwrap().to_string();

    authy_cmd(home)
        .args(["init", "--generate-keyfile", &keyfile_str])
        .assert()
        .success();

    // Store secrets
    authy_cmd(home)
        .args(["store", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .write_stdin("localhost")
        .assert()
        .success();

    authy_cmd(home)
        .args(["store", "api-key"])
        .env("AUTHY_KEYFILE", &keyfile_str)
        .write_stdin("sk-test-123")
        .assert()
        .success();

    keyfile_str
}

// --- Token-level run_only ---

#[test]
fn test_run_only_token_blocks_get() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    // Create normal policy
    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Create run-only token
    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    assert!(output.status.success());
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // get should be blocked
    authy_cmd(&home)
        .args(["get", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

#[test]
fn test_run_only_token_blocks_env() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // env should be blocked
    authy_cmd(&home)
        .args(["env", "--scope", "svc"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

#[test]
fn test_run_only_token_blocks_export() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // export should be blocked
    authy_cmd(&home)
        .args(["export", "--format", "env", "--scope", "svc"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

#[test]
fn test_run_only_token_allows_run() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // run should still work
    authy_cmd(&home)
        .args(["run", "--scope", "svc", "--uppercase", "--replace-dash", "_", "--", "echo", "ok"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn test_run_only_token_allows_list() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // list should still work (names only, no values)
    authy_cmd(&home)
        .args(["list", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .success()
        .stdout(predicate::str::contains("db-host"));
}

// --- Policy-level run_only ---

#[test]
fn test_run_only_policy_blocks_get() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    // Create run-only policy
    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Create normal (non-run-only) token
    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "agent", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // get should be blocked by policy
    authy_cmd(&home)
        .args(["get", "db-host"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

#[test]
fn test_run_only_policy_blocks_env() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "agent", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    authy_cmd(&home)
        .args(["env", "--scope", "agent"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

#[test]
fn test_run_only_policy_allows_run() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "agent", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    authy_cmd(&home)
        .args(["run", "--scope", "agent", "--uppercase", "--replace-dash", "_", "--", "echo", "ok"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

// --- Policy show/update ---

#[test]
fn test_policy_show_run_only() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Text output
    authy_cmd(&home)
        .args(["policy", "show", "agent"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("run-only"));

    // JSON output
    authy_cmd(&home)
        .args(["policy", "show", "agent", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"run_only\":true"));
}

#[test]
fn test_policy_update_run_only() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    // Create normal policy
    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Verify not run-only
    authy_cmd(&home)
        .args(["policy", "show", "svc", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"run_only\":false"));

    // Update to run-only
    authy_cmd(&home)
        .args(["policy", "update", "svc", "--run-only", "true"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Verify now run-only
    authy_cmd(&home)
        .args(["policy", "show", "svc", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"run_only\":true"));
}

// --- Session create JSON includes run_only ---

#[test]
fn test_session_create_json_includes_run_only() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "svc", "--allow", "*"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Normal token
    authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"run_only\":false"));

    // Run-only token
    authy_cmd(&home)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h", "--run-only", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"run_only\":true"));
}

// --- Master key is not restricted ---

#[test]
fn test_master_key_not_restricted_by_run_only_policy() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    // Create run-only policy
    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    // Master key (keyfile without token) can still get with --scope
    authy_cmd(&home)
        .args(["get", "db-host", "--scope", "agent"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Run-only mode"));
}

// --- JSON error output ---

#[test]
fn test_run_only_json_error() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_vault(&home);

    authy_cmd(&home)
        .args(["policy", "create", "agent", "--allow", "*", "--run-only"])
        .env("AUTHY_KEYFILE", &keyfile)
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "agent", "--ttl", "1h"])
        .env("AUTHY_KEYFILE", &keyfile)
        .output()
        .unwrap();
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    authy_cmd(&home)
        .args(["get", "db-host", "--json"])
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("\"code\":\"run_only\""));
}
