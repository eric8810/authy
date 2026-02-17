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

fn setup_vault_with_secrets(home: &TempDir) {
    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--passphrase", "testpass"])
        .assert()
        .success();

    for (name, val) in [
        ("db-host", "localhost"),
        ("db-password", "dbpass123"),
        ("db-port", "5432"),
        ("ssh-key", "my-ssh-key"),
        ("api-token", "tok123"),
    ] {
        authy_cmd(home)
            .args(["store", name])
            .write_stdin(val)
            .assert()
            .success();
    }
}

#[test]
fn test_policy_create_and_show() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*", "--description", "DB access"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "show", "deploy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deploy"))
        .stdout(predicate::str::contains("db-*"))
        .stdout(predicate::str::contains("DB access"));
}

#[test]
fn test_policy_enforced_on_get() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    // Allowed
    authy_cmd(&home)
        .args(["get", "db-password", "--scope", "deploy"])
        .assert()
        .success()
        .stdout("dbpass123");

    // Denied
    authy_cmd(&home)
        .args(["get", "ssh-key", "--scope", "deploy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("denied"));
}

#[test]
fn test_policy_deny_overrides_allow() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args([
            "policy", "create", "limited",
            "--allow", "db-*",
            "--deny", "db-password",
        ])
        .assert()
        .success();

    // db-host allowed
    authy_cmd(&home)
        .args(["get", "db-host", "--scope", "limited"])
        .assert()
        .success()
        .stdout("localhost");

    // db-password denied (deny overrides)
    authy_cmd(&home)
        .args(["get", "db-password", "--scope", "limited"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("denied"));
}

#[test]
fn test_policy_list_filters() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["list", "--scope", "deploy"])
        .assert()
        .success()
        .stdout(predicate::str::contains("db-host"))
        .stdout(predicate::str::contains("db-password"))
        .stdout(predicate::str::contains("db-port"))
        .stdout(predicate::str::contains("ssh-key").not())
        .stdout(predicate::str::contains("api-token").not());
}

#[test]
fn test_policy_test_command() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "test", "--scope", "deploy", "db-host"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ALLOWED"));

    authy_cmd(&home)
        .args(["policy", "test", "--scope", "deploy", "ssh-key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("DENIED"));
}

#[test]
fn test_policy_update() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    // Update to also allow api-*
    authy_cmd(&home)
        .args(["policy", "update", "deploy", "--allow", "db-*", "api-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["get", "api-token", "--scope", "deploy"])
        .assert()
        .success()
        .stdout("tok123");
}

#[test]
fn test_policy_remove() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "remove", "deploy"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "show", "deploy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_policy_list_command() {
    let home = TempDir::new().unwrap();
    setup_vault_with_secrets(&home);

    authy_cmd(&home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "create", "ci", "--allow", "api-*"])
        .assert()
        .success();

    authy_cmd(&home)
        .args(["policy", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("deploy"))
        .stdout(predicate::str::contains("ci"));
}
