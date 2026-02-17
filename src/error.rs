use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthyError {
    #[error("Vault not initialized. Run `authy init` first.")]
    VaultNotInitialized,

    #[error("Vault already initialized at {0}")]
    VaultAlreadyExists(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Secret already exists: {0} (use --force to overwrite)")]
    SecretAlreadyExists(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(String),

    #[error("Policy already exists: {0}")]
    PolicyAlreadyExists(String),

    #[error("Access denied: secret '{secret}' not allowed by scope '{scope}'")]
    AccessDenied { secret: String, scope: String },

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Invalid session token")]
    InvalidToken,

    #[error("Session token expired")]
    TokenExpired,

    #[error("Session token revoked")]
    #[allow(dead_code)]
    TokenRevoked,

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Write operations require master key authentication (tokens are read-only)")]
    TokenReadOnly,

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Audit chain integrity violation at entry {0}")]
    AuditChainBroken(usize),

    #[error("Invalid keyfile: {0}")]
    InvalidKeyfile(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AuthyError>;
