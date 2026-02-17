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
fn test_import_basic() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "FOO=bar\nBAZ=qux\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("2 secret(s) imported"));

    // Default transform: UPPER_SNAKE -> lower-kebab
    authy_cmd(&home)
        .args(["get", "foo"])
        .assert()
        .success()
        .stdout("bar");

    authy_cmd(&home)
        .args(["get", "baz"])
        .assert()
        .success()
        .stdout("qux");
}

#[test]
fn test_import_name_transform() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "DATABASE_URL=postgres://localhost\nAPI_KEY=sk-123\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success();

    // UPPER_SNAKE_CASE -> lower-kebab-case
    authy_cmd(&home)
        .args(["get", "database-url"])
        .assert()
        .success()
        .stdout("postgres://localhost");

    authy_cmd(&home)
        .args(["get", "api-key"])
        .assert()
        .success()
        .stdout("sk-123");
}

#[test]
fn test_import_keep_names() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "MY_VAR=hello\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap(), "--keep-names"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "MY_VAR"])
        .assert()
        .success()
        .stdout("hello");
}

#[test]
fn test_import_force_overwrite() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Store initial secret
    authy_cmd(&home)
        .args(["store", "my-secret"])
        .write_stdin("old-value")
        .assert()
        .success();

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "MY_SECRET=new-value\n").unwrap();

    // Without --force: skipped
    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("Skipping"))
        .stderr(predicate::str::contains("0 secret(s) imported"));

    // With --force: overwritten
    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap(), "--force"])
        .assert()
        .success()
        .stderr(predicate::str::contains("1 secret(s) imported"));

    authy_cmd(&home)
        .args(["get", "my-secret"])
        .assert()
        .success()
        .stdout("new-value");
}

#[test]
fn test_import_dry_run() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "FOO=bar\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap(), "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"))
        .stderr(predicate::str::contains("dry run"));

    // Should not actually store anything
    authy_cmd(&home)
        .args(["get", "foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_import_quoted_values() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(
        &env_file,
        "SINGLE='hello world'\nDOUBLE=\"hello\\nworld\"\nPLAIN=simple\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "single"])
        .assert()
        .success()
        .stdout("hello world");

    authy_cmd(&home)
        .args(["get", "double"])
        .assert()
        .success()
        .stdout("hello\nworld");

    authy_cmd(&home)
        .args(["get", "plain"])
        .assert()
        .success()
        .stdout("simple");
}

#[test]
fn test_import_comments_and_blank_lines() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(
        &env_file,
        "# Comment\n\nFOO=bar\n# Another comment\nBAZ=qux\n\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("2 secret(s) imported"));
}

#[test]
fn test_import_export_prefix() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "export FOO=bar\nexport BAZ=qux\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("2 secret(s) imported"));

    authy_cmd(&home)
        .args(["get", "foo"])
        .assert()
        .success()
        .stdout("bar");
}

#[test]
fn test_import_stdin() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "-"])
        .write_stdin("STDIN_VAR=from_stdin\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("1 secret(s) imported"));

    authy_cmd(&home)
        .args(["get", "stdin-var"])
        .assert()
        .success()
        .stdout("from_stdin");
}

#[test]
fn test_import_with_prefix() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "FOO=bar\n").unwrap();

    authy_cmd(&home)
        .args(["import", env_file.to_str().unwrap(), "--prefix", "dev-"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "dev-foo"])
        .assert()
        .success()
        .stdout("bar");
}
