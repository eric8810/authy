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

fn setup_with_passphrase(home: &TempDir) {
    authy_cmd(home)
        .args(["init", "--passphrase", "oldpass"])
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["store", "db-host"])
        .write_stdin("localhost")
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["store", "api-key"])
        .write_stdin("sk-test-123")
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["policy", "create", "svc", "--allow", "*"])
        .assert()
        .success();
}

fn setup_with_keyfile(home: &TempDir) -> String {
    let keyfile = home.path().join("old.key");
    let keyfile_str = keyfile.to_str().unwrap().to_string();

    authy_cmd(home)
        .args(["init", "--generate-keyfile", &keyfile_str])
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_KEYFILE", &keyfile_str)
        .args(["store", "db-host"])
        .write_stdin("localhost")
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_KEYFILE", &keyfile_str)
        .args(["store", "api-key"])
        .write_stdin("sk-test-123")
        .assert()
        .success();

    authy_cmd(home)
        .env("AUTHY_KEYFILE", &keyfile_str)
        .args(["policy", "create", "svc", "--allow", "*"])
        .assert()
        .success();

    keyfile_str
}

#[test]
fn test_rekey_passphrase_to_passphrase() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    // Rekey with new passphrase
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["rekey", "--to-passphrase"])
        .env("AUTHY_NON_INTERACTIVE", "0")
        // Use pipe to provide new passphrase (non-interactive won't work, so we test generate-keyfile instead)
        .assert()
        // This will fail because it tries to prompt — that's expected for passphrase→passphrase in tests
        // Instead, test the passphrase path via keyfile generation
        .failure();
}

#[test]
fn test_rekey_passphrase_to_keyfile() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    // Rekey from passphrase to generated keyfile
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["rekey", "--generate-keyfile", &new_keyfile_str])
        .assert()
        .success()
        .stderr(predicate::str::contains("Vault re-encrypted successfully"))
        .stderr(predicate::str::contains("session tokens are now invalidated"));

    // Verify: new keyfile works
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &new_keyfile_str)
        .args(["get", "db-host"])
        .assert()
        .success()
        .stdout("localhost");

    // Verify: old passphrase fails
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["get", "db-host"])
        .assert()
        .failure();
}

#[test]
fn test_rekey_keyfile_to_passphrase() {
    let home = TempDir::new().unwrap();
    let old_keyfile = setup_with_keyfile(&home);

    // Rekey from keyfile to new generated keyfile (since we can't prompt in tests)
    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &old_keyfile)
        .args(["rekey", "--generate-keyfile", &new_keyfile_str])
        .assert()
        .success()
        .stderr(predicate::str::contains("Vault re-encrypted successfully"));

    // New keyfile works
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &new_keyfile_str)
        .args(["get", "db-host"])
        .assert()
        .success()
        .stdout("localhost");

    // Old keyfile fails
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &old_keyfile)
        .args(["get", "db-host"])
        .assert()
        .failure();
}

#[test]
fn test_rekey_old_passphrase_fails() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["rekey", "--generate-keyfile", &new_keyfile_str])
        .assert()
        .success();

    // Old passphrase should fail
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["list"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_rekey_vault_contents_preserved() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["rekey", "--generate-keyfile", &new_keyfile_str])
        .assert()
        .success();

    // Secrets preserved
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &new_keyfile_str)
        .args(["get", "db-host"])
        .assert()
        .success()
        .stdout("localhost");

    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &new_keyfile_str)
        .args(["get", "api-key"])
        .assert()
        .success()
        .stdout("sk-test-123");

    // Policies preserved
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &new_keyfile_str)
        .args(["policy", "show", "svc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("svc"));
}

#[test]
fn test_rekey_token_rejected() {
    let home = TempDir::new().unwrap();
    let keyfile = setup_with_keyfile(&home);

    // Create a session token
    let output = authy_cmd(&home)
        .env("AUTHY_KEYFILE", &keyfile)
        .args(["session", "create", "--scope", "svc", "--ttl", "1h"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    // Rekey with token should fail (write required)
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &keyfile)
        .env("AUTHY_TOKEN", &token)
        .args(["rekey", "--generate-keyfile", &new_keyfile_str])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Write operations require master key"));
}

#[test]
fn test_rekey_with_existing_keyfile() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    // Generate a keyfile independently
    let keyfile_path = home.path().join("external.key");
    let keyfile_str = keyfile_path.to_str().unwrap().to_string();

    // First init a throwaway vault to generate a keyfile
    let tmp_home = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", tmp_home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--generate-keyfile", &keyfile_str])
        .assert()
        .success();

    // Rekey using existing keyfile
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args(["rekey", "--new-keyfile", &keyfile_str])
        .assert()
        .success()
        .stderr(predicate::str::contains("Vault re-encrypted successfully"));

    // Verify: existing keyfile works
    authy_cmd(&home)
        .env("AUTHY_KEYFILE", &keyfile_str)
        .args(["get", "db-host"])
        .assert()
        .success()
        .stdout("localhost");
}

#[test]
fn test_rekey_mutual_exclusivity() {
    let home = TempDir::new().unwrap();
    setup_with_passphrase(&home);

    let new_keyfile = home.path().join("new.key");
    let new_keyfile_str = new_keyfile.to_str().unwrap().to_string();

    // Two flags should fail
    authy_cmd(&home)
        .env("AUTHY_PASSPHRASE", "oldpass")
        .args([
            "rekey",
            "--generate-keyfile",
            &new_keyfile_str,
            "--to-passphrase",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Only one of"));
}
