//! Tests for the `AuthyClient` programmatic API (`src/api.rs`).
//!
//! These exercise the library surface directly — no CLI, no subprocess.
//! Each test creates an isolated vault in a temp HOME directory.
//!
//! All tests are `#[serial]` because they mutate the global HOME env var.

use serial_test::serial;
use tempfile::TempDir;

/// Set HOME to an isolated temp dir so vault operations don't collide.
fn with_isolated_home(f: impl FnOnce(&TempDir)) {
    let home = TempDir::new().unwrap();
    // AuthyClient reads HOME via dirs::home_dir, so override it.
    std::env::set_var("HOME", home.path());
    f(&home);
}

// ── init ─────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_init_vault() {
    with_isolated_home(|home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();

        assert!(!authy::api::AuthyClient::is_initialized());
        client.init_vault().unwrap();
        assert!(authy::api::AuthyClient::is_initialized());
        assert!(home.path().join(".authy/vault.age").exists());
    });
}

#[test]
#[serial]
fn test_api_init_twice_fails() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        let err = client.init_vault().unwrap_err();
        assert!(err.to_string().contains("already initialized"));
    });
}

// ── store / get ──────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_store_and_get() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("api-key", "sk-secret-123", false).unwrap();

        let val = client.get("api-key").unwrap();
        assert_eq!(val, Some("sk-secret-123".to_string()));
    });
}

#[test]
#[serial]
fn test_api_get_or_err() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("exists", "val", false).unwrap();
        assert_eq!(client.get_or_err("exists").unwrap(), "val");

        let err = client.get_or_err("nope").unwrap_err();
        assert!(err.to_string().contains("not found"));
    });
}

#[test]
#[serial]
fn test_api_get_nonexistent_returns_none() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        assert_eq!(client.get("nope").unwrap(), None);
    });
}

#[test]
#[serial]
fn test_api_store_duplicate_fails() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("dup", "v1", false).unwrap();
        let err = client.store("dup", "v2", false).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    });
}

#[test]
#[serial]
fn test_api_store_force_overwrite() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("key", "v1", false).unwrap();
        client.store("key", "v2", true).unwrap();

        assert_eq!(client.get("key").unwrap(), Some("v2".to_string()));
    });
}

// ── remove ───────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_remove() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("to-remove", "val", false).unwrap();
        assert!(client.remove("to-remove").unwrap());
        assert_eq!(client.get("to-remove").unwrap(), None);
    });
}

#[test]
#[serial]
fn test_api_remove_nonexistent_returns_false() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        assert!(!client.remove("nope").unwrap());
    });
}

// ── rotate ───────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_rotate() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("rotating", "v1", false).unwrap();
        let version = client.rotate("rotating", "v2").unwrap();
        assert_eq!(version, 2);

        assert_eq!(client.get("rotating").unwrap(), Some("v2".to_string()));
    });
}

#[test]
#[serial]
fn test_api_rotate_nonexistent_fails() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        let err = client.rotate("nope", "v1").unwrap_err();
        assert!(err.to_string().contains("not found"));
    });
}

// ── list ─────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_list() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("alpha", "a", false).unwrap();
        client.store("beta", "b", false).unwrap();
        client.store("gamma", "g", false).unwrap();

        let names = client.list(None).unwrap();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    });
}

#[test]
#[serial]
fn test_api_list_empty() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        let names = client.list(None).unwrap();
        assert!(names.is_empty());
    });
}

// ── audit ────────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_audit_entries() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("s1", "v1", false).unwrap();
        client.get("s1").unwrap();

        let entries = client.audit_entries().unwrap();
        // init + store + get = 3 entries minimum
        assert!(entries.len() >= 3);
    });
}

#[test]
#[serial]
fn test_api_verify_audit_chain() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        client.store("s1", "v1", false).unwrap();

        let (count, valid) = client.verify_audit_chain().unwrap();
        assert!(count >= 2); // init + store
        assert!(valid);
    });
}

// ── with_actor ───────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_custom_actor() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("test-pass")
            .unwrap()
            .with_actor("my-app");
        client.init_vault().unwrap();
        client.store("key", "val", false).unwrap();

        let entries = client.audit_entries().unwrap();
        let last = entries.last().unwrap();
        assert_eq!(last.actor, "my-app");
    });
}

// ── from_env ─────────────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_from_env_passphrase() {
    with_isolated_home(|_home| {
        std::env::set_var("AUTHY_PASSPHRASE", "env-pass");
        std::env::remove_var("AUTHY_KEYFILE");

        let client = authy::api::AuthyClient::from_env().unwrap();
        client.init_vault().unwrap();
        client.store("env-test", "val", false).unwrap();
        assert_eq!(client.get("env-test").unwrap(), Some("val".to_string()));

        std::env::remove_var("AUTHY_PASSPHRASE");
    });
}

#[test]
#[serial]
fn test_api_from_env_no_credentials_fails() {
    with_isolated_home(|_home| {
        std::env::remove_var("AUTHY_PASSPHRASE");
        std::env::remove_var("AUTHY_KEYFILE");

        let result = authy::api::AuthyClient::from_env();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("No credentials found"));
    });
}

// ── wrong passphrase ─────────────────────────────────────────────────

#[test]
#[serial]
fn test_api_wrong_passphrase_fails() {
    with_isolated_home(|_home| {
        let client = authy::api::AuthyClient::with_passphrase("right-pass").unwrap();
        client.init_vault().unwrap();
        client.store("key", "val", false).unwrap();

        let wrong = authy::api::AuthyClient::with_passphrase("wrong-pass").unwrap();
        assert!(wrong.get("key").is_err());
    });
}
