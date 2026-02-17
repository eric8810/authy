use crate::types::*;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A single secret entry in the vault.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecretEntry {
    /// The secret value (plaintext once vault is decrypted).
    pub value: String,
    /// Metadata about this secret.
    #[zeroize(skip)]
    pub metadata: SecretMetadata,
}

/// Metadata associated with a secret (non-sensitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub version: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl SecretMetadata {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            modified_at: now,
            version: 1,
            tags: Vec::new(),
            description: None,
        }
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
        self.modified_at = Utc::now();
    }
}

impl SecretEntry {
    pub fn new(value: String) -> Self {
        Self {
            value,
            metadata: SecretMetadata::new(),
        }
    }
}
