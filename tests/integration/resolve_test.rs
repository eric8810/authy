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

fn setup(home: &TempDir) {
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
        ("db-port", "5432"),
        ("api-key", "sk-test-123"),
    ] {
        authy_cmd(home)
            .args(["store", name])
            .write_stdin(val)
            .assert()
            .success();
    }

    authy_cmd(home)
        .args(["policy", "create", "deploy", "--allow", "db-*"])
        .assert()
        .success();

    authy_cmd(home)
        .args(["policy", "create", "all", "--allow", "*"])
        .assert()
        .success();
}

#[test]
fn test_resolve_yaml() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("config.yaml");
    fs::write(
        &src,
        "host: <authy:db-host>\nport: <authy:db-port>\n",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .success()
        .stdout("host: localhost\nport: 5432\n");
}

#[test]
fn test_resolve_json() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("config.json");
    fs::write(
        &src,
        r#"{"host": "<authy:db-host>", "port": "<authy:db-port>"}"#,
    )
    .unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .success()
        .stdout(r#"{"host": "localhost", "port": "5432"}"#);
}

#[test]
fn test_resolve_multiple_placeholders() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("multi.txt");
    fs::write(
        &src,
        "a=<authy:db-host> b=<authy:db-port> c=<authy:db-host>",
    )
    .unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .success()
        .stdout("a=localhost b=5432 c=localhost");
}

#[test]
fn test_resolve_missing_key() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("bad.txt");
    fs::write(&src, "val=<authy:nonexistent>").unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("Secret not found"));
}

#[test]
fn test_resolve_access_denied() {
    let home = TempDir::new().unwrap();
    setup(&home);

    // api-key is not allowed by "deploy" policy (only db-*)
    let src = home.path().join("denied.txt");
    fs::write(&src, "key=<authy:api-key>").unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .failure()
        .code(4)
        .stderr(predicate::str::contains("Access denied"));
}

#[test]
fn test_resolve_to_stdout() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("out.txt");
    fs::write(&src, "host=<authy:db-host>").unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .success()
        .stdout("host=localhost");
}

#[test]
fn test_resolve_to_file() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("src.txt");
    let dst = home.path().join("dst.txt");
    fs::write(&src, "host=<authy:db-host>").unwrap();

    authy_cmd(&home)
        .args([
            "resolve",
            src.to_str().unwrap(),
            "--scope",
            "deploy",
            "--output",
            dst.to_str().unwrap(),
        ])
        .assert()
        .success();

    let result = fs::read_to_string(&dst).unwrap();
    assert_eq!(result, "host=localhost");
}

#[test]
fn test_resolve_no_placeholders() {
    let home = TempDir::new().unwrap();
    setup(&home);

    let src = home.path().join("plain.txt");
    fs::write(&src, "no placeholders here").unwrap();

    authy_cmd(&home)
        .args(["resolve", src.to_str().unwrap(), "--scope", "deploy"])
        .assert()
        .success()
        .stdout("no placeholders here");
}

#[test]
fn test_resolve_run_only_allowed() {
    let home = TempDir::new().unwrap();

    // Setup with keyfile
    let keyfile = home.path().join("test.key");
    let keyfile_str = keyfile.to_str().unwrap().to_string();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_KEYFILE")
        .env_remove("AUTHY_TOKEN")
        .args(["init", "--generate-keyfile", &keyfile_str])
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", &keyfile_str)
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_TOKEN")
        .args(["store", "db-host"])
        .write_stdin("localhost")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", &keyfile_str)
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_TOKEN")
        .args(["policy", "create", "svc", "--allow", "*"])
        .assert()
        .success();

    // Create run-only token
    let output = Command::cargo_bin("authy")
        .unwrap()
        .env("HOME", home.path())
        .env("AUTHY_KEYFILE", &keyfile_str)
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_TOKEN")
        .args([
            "session", "create", "--scope", "svc", "--ttl", "1h", "--run-only",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let token = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // resolve should work with run-only token
    let src = home.path().join("config.txt");
    fs::write(&src, "host=<authy:db-host>").unwrap();

    let mut cmd = Command::cargo_bin("authy").unwrap();
    cmd.env("HOME", home.path())
        .env("AUTHY_KEYFILE", &keyfile_str)
        .env("AUTHY_TOKEN", &token)
        .env_remove("AUTHY_PASSPHRASE")
        .args(["resolve", src.to_str().unwrap(), "--scope", "svc"])
        .assert()
        .success()
        .stdout("host=localhost");
}
