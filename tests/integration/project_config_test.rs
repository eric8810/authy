use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn authy_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path());
    cmd.env("AUTHY_PASSPHRASE", "testpass");
    cmd.env_remove("AUTHY_KEYFILE");
    cmd.env_remove("AUTHY_TOKEN");
    cmd.env_remove("AUTHY_PROJECT_DIR");
    cmd
}

#[test]
fn test_project_info_shows_scope() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        "[authy]\nscope = \"my-project\"\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["project-info", "--field", "scope", "--dir"])
        .arg(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("my-project"));
}

#[test]
fn test_project_info_shows_all() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        r#"[authy]
scope = "test-proj"
uppercase = true
replace_dash = "_"
prefix = "APP_"
aliases = ["claude", "aider"]
"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["project-info", "--dir"])
        .arg(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("scope: test-proj"))
        .stdout(predicate::str::contains("uppercase: true"))
        .stdout(predicate::str::contains("replace-dash: _"))
        .stdout(predicate::str::contains("prefix: APP_"))
        .stdout(predicate::str::contains("aliases: claude, aider"));
}

#[test]
fn test_project_info_json() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        "[authy]\nscope = \"json-test\"\nuppercase = true\n",
    )
    .unwrap();

    let output = authy_cmd(&home)
        .args(["--json", "project-info", "--dir"])
        .arg(project.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["scope"], "json-test");
    assert_eq!(json["uppercase"], true);
}

#[test]
fn test_project_info_keyfile_tilde_expansion() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        "[authy]\nscope = \"test\"\nkeyfile = \"~/.authy/keys/test.key\"\n",
    )
    .unwrap();

    let output = authy_cmd(&home)
        .args(["project-info", "--field", "keyfile", "--dir"])
        .arg(project.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should not start with ~ after expansion
    assert!(!stdout.trim().starts_with('~'));
    assert!(stdout.trim().ends_with("/.authy/keys/test.key"));
}

#[test]
fn test_project_info_error_when_no_config() {
    let home = TempDir::new().unwrap();
    let empty_dir = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["project-info", "--dir"])
        .arg(empty_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No .authy.toml found"));
}

#[test]
fn test_project_info_unknown_field() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        "[authy]\nscope = \"test\"\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["project-info", "--field", "nonexistent", "--dir"])
        .arg(project.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown field"));
}

#[test]
fn test_project_info_discovers_parent() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();
    let nested = project.path().join("a").join("b");
    fs::create_dir_all(&nested).unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        "[authy]\nscope = \"parent-scope\"\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["project-info", "--field", "scope", "--dir"])
        .arg(&nested)
        .assert()
        .success()
        .stdout(predicate::str::contains("parent-scope"));
}
