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
fn test_alias_explicit_scope() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["alias", "my-scope", "claude", "aider"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "alias claude='authy run --scope my-scope --uppercase --replace-dash _ -- claude'",
        ))
        .stdout(predicate::str::contains(
            "alias aider='authy run --scope my-scope --uppercase --replace-dash _ -- aider'",
        ));
}

#[test]
fn test_alias_from_project() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        r#"[authy]
scope = "test-proj"
uppercase = true
replace_dash = "_"
aliases = ["claude", "aider"]
"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["alias", "--from-project"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "alias claude='authy run --scope test-proj --uppercase --replace-dash _ -- claude'",
        ))
        .stdout(predicate::str::contains(
            "alias aider='authy run --scope test-proj --uppercase --replace-dash _ -- aider'",
        ));
}

#[test]
fn test_alias_from_project_with_prefix() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        r#"[authy]
scope = "my-proj"
prefix = "APP_"
aliases = ["claude"]
"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["alias", "--from-project"])
        .current_dir(project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("--prefix APP_"));
}

#[test]
fn test_alias_cleanup() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        r#"[authy]
scope = "test"
aliases = ["claude", "aider"]
"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["alias", "--cleanup"])
        .env("AUTHY_PROJECT_DIR", project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("unalias claude 2>/dev/null"))
        .stdout(predicate::str::contains("unalias aider 2>/dev/null"));
}

#[test]
fn test_alias_fish_syntax() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["alias", "--shell", "fish", "my-scope", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "alias claude 'authy run --scope my-scope --uppercase --replace-dash _ -- claude'",
        ));
}

#[test]
fn test_alias_powershell_syntax() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["alias", "--shell", "powershell", "my-scope", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains("function claude {"));
}

#[test]
fn test_alias_error_no_scope() {
    let home = TempDir::new().unwrap();
    let empty = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["alias", "--from-project"])
        .current_dir(empty.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No .authy.toml found"));
}

#[test]
fn test_alias_error_no_tools() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["alias", "my-scope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No tools specified"));
}

#[test]
fn test_alias_cleanup_fish() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join(".authy.toml"),
        r#"[authy]
scope = "test"
aliases = ["claude"]
"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["alias", "--cleanup", "--shell", "fish"])
        .env("AUTHY_PROJECT_DIR", project.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("functions --erase claude"));
}
