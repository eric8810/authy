//! High-level programmatic API for the Authy vault.
//!
//! [`AuthyClient`] provides a simple facade over the vault, handling
//! load → operate → save → audit in every method call.

use crate::audit;
use crate::auth;
use crate::error::{AuthyError, Result};
use crate::vault::{self, Vault, VaultKey};
use crate::vault::secret::SecretEntry;

/// High-level client for programmatic vault access.
///
/// Each operation loads the vault, performs the mutation, saves it back,
/// and appends an audit entry — mirroring the CLI handler pattern.
pub struct AuthyClient {
    key: VaultKey,
    /// HMAC key derived from the master material, used for audit chain.
    audit_key: Vec<u8>,
    /// Human-readable actor label for audit entries.
    actor: String,
}

impl AuthyClient {
    /// Authenticate with a passphrase.
    pub fn with_passphrase(passphrase: &str) -> Result<Self> {
        let key = VaultKey::Passphrase(passphrase.to_string());
        let material = audit::key_material(&key);
        let audit_key = audit::derive_audit_key(&material);
        Ok(Self {
            key,
            audit_key,
            actor: "api(passphrase)".to_string(),
        })
    }

    /// Authenticate with an age keyfile on disk.
    pub fn with_keyfile(keyfile_path: &str) -> Result<Self> {
        let (identity, pubkey) = auth::read_keyfile(keyfile_path)?;
        let key = VaultKey::Keyfile { identity, pubkey };
        let material = audit::key_material(&key);
        let audit_key = audit::derive_audit_key(&material);
        Ok(Self {
            key,
            audit_key,
            actor: "api(keyfile)".to_string(),
        })
    }

    /// Authenticate from environment variables (`AUTHY_KEYFILE` or `AUTHY_PASSPHRASE`).
    ///
    /// This does **not** fall through to interactive prompts — it only reads env vars.
    pub fn from_env() -> Result<Self> {
        if let Ok(keyfile_path) = std::env::var("AUTHY_KEYFILE") {
            return Self::with_keyfile(&keyfile_path);
        }
        if let Ok(passphrase) = std::env::var("AUTHY_PASSPHRASE") {
            return Self::with_passphrase(&passphrase);
        }
        Err(AuthyError::AuthFailed(
            "No credentials found. Set AUTHY_KEYFILE or AUTHY_PASSPHRASE.".into(),
        ))
    }

    /// Override the actor label used in audit entries.
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = actor.into();
        self
    }

    /// Check whether the vault has been initialized.
    pub fn is_initialized() -> bool {
        vault::is_initialized()
    }

    /// Retrieve a secret by name. Returns `None` if not found.
    pub fn get(&self, name: &str) -> Result<Option<String>> {
        let v = vault::load_vault(&self.key)?;

        let result = v.secrets.get(name).map(|e| e.value.clone());
        let outcome = if result.is_some() { "success" } else { "not_found" };

        self.audit("get", Some(name), outcome, None);
        Ok(result)
    }

    /// Retrieve a secret by name, returning an error if it does not exist.
    pub fn get_or_err(&self, name: &str) -> Result<String> {
        self.get(name)?
            .ok_or_else(|| AuthyError::SecretNotFound(name.to_string()))
    }

    /// Store a secret. If `force` is false and the secret already exists,
    /// returns [`AuthyError::SecretAlreadyExists`].
    pub fn store(&self, name: &str, value: &str, force: bool) -> Result<()> {
        let mut v = vault::load_vault(&self.key)?;

        if !force && v.secrets.contains_key(name) {
            self.audit("store", Some(name), "denied", Some("already exists"));
            return Err(AuthyError::SecretAlreadyExists(name.to_string()));
        }

        let is_update = v.secrets.contains_key(name);
        v.secrets
            .insert(name.to_string(), SecretEntry::new(value.to_string()));
        v.touch();
        vault::save_vault(&v, &self.key)?;

        let op = if is_update { "update" } else { "store" };
        self.audit(op, Some(name), "success", None);
        Ok(())
    }

    /// Remove a secret. Returns `true` if the secret existed.
    pub fn remove(&self, name: &str) -> Result<bool> {
        let mut v = vault::load_vault(&self.key)?;

        let existed = v.secrets.remove(name).is_some();
        if existed {
            v.touch();
            vault::save_vault(&v, &self.key)?;
            self.audit("remove", Some(name), "success", None);
        } else {
            self.audit("remove", Some(name), "not_found", None);
        }

        Ok(existed)
    }

    /// Rotate a secret to a new value. Returns the new version number.
    /// The secret must already exist.
    pub fn rotate(&self, name: &str, new_value: &str) -> Result<u32> {
        let mut v = vault::load_vault(&self.key)?;

        let entry = v
            .secrets
            .get_mut(name)
            .ok_or_else(|| AuthyError::SecretNotFound(name.to_string()))?;

        entry.value = new_value.to_string();
        entry.metadata.bump_version();
        let version = entry.metadata.version;

        v.touch();
        vault::save_vault(&v, &self.key)?;

        self.audit(
            "rotate",
            Some(name),
            "success",
            Some(&format!("v{version}")),
        );
        Ok(version)
    }

    /// List secret names, optionally filtered by a policy scope.
    pub fn list(&self, scope: Option<&str>) -> Result<Vec<String>> {
        let v = vault::load_vault(&self.key)?;

        let names: Vec<String> = if let Some(scope_name) = scope {
            let policy = v
                .policies
                .get(scope_name)
                .ok_or_else(|| AuthyError::PolicyNotFound(scope_name.to_string()))?;
            let all_names: Vec<&str> = v.secrets.keys().map(String::as_str).collect();
            policy
                .filter_secrets(&all_names)?
                .into_iter()
                .map(String::from)
                .collect()
        } else {
            v.secrets.keys().cloned().collect()
        };

        self.audit("list", None, "success", None);
        Ok(names)
    }

    /// Initialize a new vault. The vault must not already exist.
    pub fn init_vault(&self) -> Result<()> {
        if vault::is_initialized() {
            return Err(AuthyError::VaultAlreadyExists(
                vault::vault_path().display().to_string(),
            ));
        }
        let v = Vault::new();
        vault::save_vault(&v, &self.key)?;

        // Write default config
        let config = crate::config::Config::default();
        config.save(&vault::config_path())?;

        self.audit("init", None, "success", None);
        Ok(())
    }

    /// Read all audit entries from the log.
    pub fn audit_entries(&self) -> Result<Vec<audit::AuditEntry>> {
        audit::read_entries(&vault::audit_path())
    }

    /// Verify the integrity of the audit chain.
    /// Returns `(entry_count, valid)`.
    pub fn verify_audit_chain(&self) -> Result<(usize, bool)> {
        audit::verify_chain(&vault::audit_path(), &self.audit_key)
    }

    /// Test whether a policy allows access to a secret.
    /// Returns `true` if allowed, `false` if denied.
    pub fn test_policy(&self, scope: &str, secret_name: &str) -> Result<bool> {
        let v = vault::load_vault(&self.key)?;

        let policy = v
            .policies
            .get(scope)
            .ok_or_else(|| AuthyError::PolicyNotFound(scope.to_string()))?;

        let allowed = policy.can_read(secret_name)?;
        let outcome = if allowed { "allowed" } else { "denied" };

        self.audit(
            "policy.test",
            Some(secret_name),
            outcome,
            Some(&format!("scope={}", scope)),
        );
        Ok(allowed)
    }

    /// Create a new policy in the vault.
    pub fn create_policy(
        &self,
        name: &str,
        allow: Vec<String>,
        deny: Vec<String>,
        description: Option<&str>,
        run_only: bool,
    ) -> Result<()> {
        use crate::policy::Policy;

        let mut v = vault::load_vault(&self.key)?;

        if v.policies.contains_key(name) {
            return Err(AuthyError::PolicyAlreadyExists(name.to_string()));
        }

        let mut policy = Policy::new(name.to_string(), allow, deny);
        policy.description = description.map(String::from);
        policy.run_only = run_only;
        v.policies.insert(name.to_string(), policy);
        v.touch();
        vault::save_vault(&v, &self.key)?;

        self.audit(
            "policy.create",
            None,
            "success",
            Some(&format!("policy={}", name)),
        );
        Ok(())
    }

    // ── internal helpers ─────────────────────────────────────────

    fn audit(&self, operation: &str, secret: Option<&str>, outcome: &str, detail: Option<&str>) {
        let _ = audit::log_event(
            &vault::audit_path(),
            operation,
            secret,
            &self.actor,
            outcome,
            detail,
            &self.audit_key,
        );
    }
}
