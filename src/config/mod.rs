pub mod project;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::Result;

/// Configuration file format (~/.authy/authy.toml).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub vault: VaultConfig,
    #[serde(default)]
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    /// Default auth method: "passphrase" or "keyfile"
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
    /// Path to the keyfile (if auth_method is "keyfile")
    pub keyfile: Option<String>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            auth_method: default_auth_method(),
            keyfile: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn default_auth_method() -> String {
    "passphrase".to_string()
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load config from a path. Returns default config if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::AuthyError::Other(format!("Invalid config: {}", e)))?;
        Ok(config)
    }

    /// Save config to a path.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::AuthyError::Other(format!("Config serialize error: {}", e)))?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}
