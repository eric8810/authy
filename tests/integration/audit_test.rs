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
        .args(["store", "my-secret"])
        .write_stdin("val")
        .assert()
        .success();

    authy_cmd(home)
        .args(["get", "my-secret"])
        .assert()
        .success();
}

#[test]
fn test_audit_show() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["audit", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("store"))
        .stdout(predicate::str::contains("get"));
}

#[test]
fn test_audit_verify() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["audit", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("verified"));
}

#[test]
fn test_audit_export() {
    let home = TempDir::new().unwrap();
    setup(&home);

    authy_cmd(&home)
        .args(["audit", "export"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\""))
        .stdout(predicate::str::contains("\"chain_hmac\""));
}

#[test]
fn test_audit_tamper_detection() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // Tamper with the audit log
    let audit_path = home.path().join(".authy/audit.log");
    let content = std::fs::read_to_string(&audit_path).unwrap();
    let tampered = content.replacen("success", "tampered", 1);
    std::fs::write(&audit_path, tampered).unwrap();

    authy_cmd(&home)
        .args(["audit", "verify"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("integrity")
            .or(predicate::str::contains("INTEGRITY"))
            .or(predicate::str::contains("violation")));
}
