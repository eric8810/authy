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

    for (name, val) in [("db-host", "localhost"), ("api-key", "sk-123")] {
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
fn test_export_env_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["export", "--format", "env"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api-key=sk-123"))
        .stdout(predicate::str::contains("db-host=localhost"));
}

#[test]
fn test_export_json_format() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let output = authy_cmd(&home)
        .args(["export", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let entries = json.as_array().unwrap();
    assert_eq!(entries.len(), 2);

    // Should contain full metadata
    for entry in entries {
        assert!(entry["name"].is_string());
        assert!(entry["value"].is_string());
        assert!(entry["version"].is_number());
        assert!(entry["created"].is_string());
        assert!(entry["modified"].is_string());
    }
}

#[test]
fn test_export_with_scope() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // Create a more restrictive policy
    authy_cmd(&home)
        .args(["policy", "create", "db-only", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["export", "--format", "env", "--scope", "db-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("db-host=localhost"))
        .stdout(predicate::str::contains("api-key").not());
}

#[test]
fn test_export_with_naming_transforms() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args([
            "export", "--format", "env",
            "--uppercase", "--replace-dash", "_",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("API_KEY=sk-123"))
        .stdout(predicate::str::contains("DB_HOST=localhost"));
}

#[test]
fn test_export_token_rejected_without_scope() {
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

    // Store secret
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env_remove("AUTHY_TOKEN")
        .env_remove("AUTHY_PASSPHRASE")
        .args(["store", "my-secret"])
        .write_stdin("val")
        .assert()
        .success();

    // Create policy and session
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env_remove("AUTHY_TOKEN")
        .env_remove("AUTHY_PASSPHRASE")
        .args(["policy", "create", "test-scope", "--allow", "*"])
        .assert()
        .success();

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

    // Export without scope using token should fail (tokens are read-only, export all requires write)
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", keyfile.to_str().unwrap())
        .env("AUTHY_TOKEN", &token)
        .env_remove("AUTHY_PASSPHRASE")
        .args(["export", "--format", "env"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("read-only"));
}

#[test]
fn test_import_export_roundtrip() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // Export as env
    let output = authy_cmd(&home)
        .args(["export", "--format", "env", "--scope", "agent"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let exported = String::from_utf8_lossy(&output.stdout);

    // Write exported to a file
    let env_file = home.path().join("exported.env");
    std::fs::write(&env_file, exported.as_ref()).unwrap();

    // Create a new vault and import
    let home2 = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home2.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home2.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["import", env_file.to_str().unwrap(), "--keep-names"])
        .assert()
        .success()
        .stderr(predicate::str::contains("2 secret(s) imported"));

    // Verify imported values
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home2.path())
        .env("AUTHY_PASSPHRASE", "testpass")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["get", "api-key"])
        .assert()
        .success()
        .stdout("sk-123");
}
