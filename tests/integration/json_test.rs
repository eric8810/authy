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

    authy_cmd(home)
        .args(["store", "api-key"])
        .write_stdin("sk-test-123")
        .assert()
        .success();

    authy_cmd(home)
        .args(["store", "db-url"])
        .write_stdin("postgres://localhost/mydb")
        .assert()
        .success();

    authy_cmd(home)
        .args(["policy", "create", "agent", "--allow", "*"])
        .assert()
        .success();
}

#[test]
fn test_get_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["get", "api-key", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["name"], "api-key");
    assert_eq!(json["value"], "sk-test-123");
    assert_eq!(json["version"], 1);
    assert!(json["created"].is_string());
    assert!(json["modified"].is_string());
}

#[test]
fn test_list_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let secrets = json["secrets"].as_array().unwrap();
    assert_eq!(secrets.len(), 2);

    // Should not contain values
    for s in secrets {
        assert!(s.get("value").is_none());
        assert!(s["name"].is_string());
        assert!(s["version"].is_number());
    }
}

#[test]
fn test_list_json_with_scope() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["list", "--scope", "agent", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let secrets = json["secrets"].as_array().unwrap();
    assert_eq!(secrets.len(), 2);
}

#[test]
fn test_policy_show_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["policy", "show", "agent", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["name"], "agent");
    assert!(json["allow"].as_array().unwrap().contains(&serde_json::Value::String("*".into())));
}

#[test]
fn test_policy_list_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["policy", "list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let policies = json["policies"].as_array().unwrap();
    assert!(!policies.is_empty());
}

#[test]
fn test_policy_test_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["policy", "test", "--scope", "agent", "api-key", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["allowed"], true);
    assert_eq!(json["scope"], "agent");
    assert_eq!(json["secret"], "api-key");
}

#[test]
fn test_session_create_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["session", "create", "--scope", "agent", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["token"].as_str().unwrap().starts_with("authy_v1."));
    assert!(json["session_id"].is_string());
    assert_eq!(json["scope"], "agent");
    assert!(json["expires"].is_string());
}

#[test]
fn test_session_list_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // Create a session first
    authy_cmd(&home)
        .args(["session", "create", "--scope", "agent"])
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["session", "list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let sessions = json["sessions"].as_array().unwrap();
    assert!(!sessions.is_empty());
    assert_eq!(sessions[0]["status"], "active");
}

#[test]
fn test_audit_show_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["audit", "show", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["entries"].is_array());
    assert!(json["total"].as_u64().unwrap() > 0);
    assert!(json["shown"].as_u64().unwrap() > 0);
}

#[test]
fn test_json_error_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["get", "nonexistent", "--json"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(stderr.trim()).unwrap();
    assert_eq!(json["error"]["code"], "not_found");
    assert!(json["error"]["exit_code"].as_i64().unwrap() == 3);
}

#[test]
fn test_run_stdout_unaffected_by_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // --json on run should not interfere with child process stdout
    authy_cmd(&home)
        .args(["run", "--scope", "agent", "--json", "--", "echo", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_list_json_empty_vault() {
    let home = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();

    let output = authy_cmd(&home)
        .args(["list", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let secrets = json["secrets"].as_array().unwrap();
    assert!(secrets.is_empty());
}
