pub mod crypto;
pub mod secret;

use std::fs;

use crate::error::{AuthyError, Result};
use crate::policy::Policy;
use crate::session::SessionRecord;
use crate::types::*;
use crate::vault::secret::SecretEntry;

/// The in-memory representation of the entire vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub secrets: BTreeMap<String, SecretEntry>,
    pub policies: BTreeMap<String, Policy>,
    pub sessions: Vec<SessionRecord>,
}

impl Vault {
    /// Create a new empty vault.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            version: 1,
            created_at: now,
            modified_at: now,
            secrets: BTreeMap::new(),
            policies: BTreeMap::new(),
            sessions: Vec::new(),
        }
    }

    /// Touch the modified timestamp.
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }
}

/// Encryption mode for the vault.
#[derive(Debug, Clone)]
pub enum VaultKey {
    Passphrase(String),
    Keyfile { identity: String, pubkey: String },
}

/// Get the default authy directory path (~/.authy).
pub fn authy_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".authy")
}

/// Get the vault file path.
pub fn vault_path() -> PathBuf {
    authy_dir().join("vault.age")
}

/// Get the config file path.
pub fn config_path() -> PathBuf {
    authy_dir().join("authy.toml")
}

/// Get the audit log path.
pub fn audit_path() -> PathBuf {
    authy_dir().join("audit.log")
}

/// Check if the vault is initialized.
pub fn is_initialized() -> bool {
    vault_path().exists()
}

/// Load and decrypt the vault from disk.
pub fn load_vault(key: &VaultKey) -> Result<Vault> {
    let path = vault_path();
    if !path.exists() {
        return Err(AuthyError::VaultNotInitialized);
    }

    let ciphertext = fs::read(&path)?;
    let plaintext = match key {
        VaultKey::Passphrase(pass) => crypto::decrypt_with_passphrase(&ciphertext, pass)?,
        VaultKey::Keyfile { identity, .. } => {
            crypto::decrypt_with_keyfile(&ciphertext, identity)?
        }
    };

    let vault: Vault =
        rmp_serde::from_slice(&plaintext).map_err(|e| AuthyError::Serialization(e.to_string()))?;

    Ok(vault)
}

/// Encrypt and save the vault to disk with atomic rename.
pub fn save_vault(vault: &Vault, key: &VaultKey) -> Result<()> {
    let path = vault_path();
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;

    let plaintext =
        rmp_serde::to_vec(vault).map_err(|e| AuthyError::Serialization(e.to_string()))?;

    let ciphertext = match key {
        VaultKey::Passphrase(pass) => crypto::encrypt_with_passphrase(&plaintext, pass)?,
        VaultKey::Keyfile { pubkey, .. } => crypto::encrypt_with_keyfile(&plaintext, pubkey)?,
    };

    // Atomic write: write to temp file, then rename
    let tmp_path = path.with_extension("age.tmp");
    fs::write(&tmp_path, &ciphertext)?;
    fs::rename(&tmp_path, &path)?;

    Ok(())
}
