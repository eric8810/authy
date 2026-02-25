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

// --- External import adapter tests ---

#[test]
fn test_import_from_dotenv_explicit() {
    // --from dotenv should behave exactly like the default
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "FOO=bar\nBAZ=qux\n").unwrap();

    authy_cmd(&home)
        .args(["import", "--from", "dotenv", env_file.to_str().unwrap()])
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
fn test_import_from_1password_missing_cli() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "--from", "1password"])
        .env("PATH", "/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("1Password CLI (`op`) not found"))
        .stderr(predicate::str::contains("https://1password.com/downloads/command-line/"));
}

#[test]
fn test_import_from_pass_missing_gpg() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Create a fake password-store with a .gpg file
    let store = home.path().join(".password-store");
    fs::create_dir_all(&store).unwrap();
    fs::write(store.join("test-secret.gpg"), b"fake-encrypted").unwrap();

    authy_cmd(&home)
        .args(["import", "--from", "pass"])
        .env("PATH", "/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("GPG not found"));
}

#[test]
fn test_import_from_pass_missing_store_dir() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "--from", "pass", "--path", "/nonexistent/store"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Password store directory not found"));
}

#[test]
fn test_import_from_sops_missing_cli() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let sops_file = home.path().join("secrets.enc.yaml");
    fs::write(&sops_file, "dummy: value").unwrap();

    authy_cmd(&home)
        .args(["import", "--from", "sops", sops_file.to_str().unwrap()])
        .env("PATH", "/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOPS CLI not found"))
        .stderr(predicate::str::contains("https://github.com/getsops/sops"));
}

#[test]
fn test_import_from_sops_requires_file() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "--from", "sops"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOPS import requires a file argument"));
}

#[test]
fn test_import_from_vault_missing_cli() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "--from", "vault", "--path", "secret/myapp"])
        .env("PATH", "/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("HashiCorp Vault CLI not found"))
        .stderr(predicate::str::contains("https://www.vaultproject.io/downloads"));
}

#[test]
fn test_import_from_vault_requires_path() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import", "--from", "vault"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("HashiCorp Vault import requires --path"));
}

#[test]
fn test_import_no_file_no_from_errors() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args(["import"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Import requires a file argument"));
}

#[test]
fn test_import_from_dotenv_with_all_flags() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    let env_file = home.path().join("test.env");
    fs::write(&env_file, "MY_KEY=my_value\n").unwrap();

    // Test with --from dotenv, --prefix, --keep-names, --dry-run
    authy_cmd(&home)
        .args([
            "import",
            "--from", "dotenv",
            env_file.to_str().unwrap(),
            "--prefix", "staging-",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"))
        .stdout(predicate::str::contains("staging-my-key"))
        .stderr(predicate::str::contains("1 secret(s) imported"));
}

#[test]
fn test_import_from_1password_with_vault_and_tag() {
    // Verify that --op-vault and --tag args are parsed correctly
    // (will fail because `op` is not installed, but we check the error message)
    let home = TempDir::new().unwrap();
    init_vault(&home);

    authy_cmd(&home)
        .args([
            "import",
            "--from", "1password",
            "--op-vault", "Engineering",
            "--tag", "api-keys",
        ])
        .env("PATH", "/nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("1Password CLI (`op`) not found"));
}

#[test]
fn test_import_from_pass_empty_store() {
    let home = TempDir::new().unwrap();
    init_vault(&home);

    // Create empty password-store directory (no .gpg files)
    let store = home.path().join(".password-store");
    fs::create_dir_all(&store).unwrap();

    authy_cmd(&home)
        .args(["import", "--from", "pass"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No secrets found in input"));
}
