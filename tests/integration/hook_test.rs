use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn authy_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path());
    cmd.env_remove("AUTHY_KEYFILE");
    cmd.env_remove("AUTHY_TOKEN");
    cmd.env_remove("AUTHY_PROJECT_DIR");
    cmd
}

#[test]
fn test_hook_bash_contains_authy_hook() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_authy_hook"))
        .stdout(predicate::str::contains("AUTHY_PROJECT_DIR"));
}

#[test]
fn test_hook_bash_overrides_cd() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd()"))
        .stdout(predicate::str::contains("builtin cd"));
}

#[test]
fn test_hook_zsh_uses_chpwd() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("chpwd"))
        .stdout(predicate::str::contains("_authy_hook"));
}

#[test]
fn test_hook_fish_uses_on_variable_pwd() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--on-variable PWD"))
        .stdout(predicate::str::contains("_authy_hook"));
}

#[test]
fn test_hook_invalid_shell() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unsupported shell"));
}

#[test]
fn test_hook_bash_syntax_check() {
    let home = TempDir::new().unwrap();

    let output = authy_cmd(&home)
        .args(["hook", "bash"])
        .output()
        .unwrap();

    assert!(output.status.success());

    // Pipe through bash -n to check syntax
    let bash_check = std::process::Command::new("bash")
        .arg("-n")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    if let Ok(mut child) = bash_check {
        use std::io::Write;
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(&output.stdout).unwrap();
        }
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "bash -n failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn test_hook_bash_has_find_config() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_authy_find_config"));
}

#[test]
fn test_hook_bash_handles_cleanup() {
    let home = TempDir::new().unwrap();

    authy_cmd(&home)
        .args(["hook", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("authy alias --cleanup"));
}
