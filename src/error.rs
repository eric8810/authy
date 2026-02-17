use serde::Serialize;
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

    #[error("Run-only mode: secret values cannot be read directly. Use `authy run` to inject secrets into a subprocess.")]
    RunOnly,

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

impl AuthyError {
    /// Return a typed exit code for this error category.
    pub fn exit_code(&self) -> i32 {
        match self {
            AuthyError::VaultNotInitialized => 7,
            AuthyError::VaultAlreadyExists(_) => 5,
            AuthyError::SecretNotFound(_) => 3,
            AuthyError::SecretAlreadyExists(_) => 5,
            AuthyError::PolicyNotFound(_) => 3,
            AuthyError::PolicyAlreadyExists(_) => 5,
            AuthyError::AccessDenied { .. } => 4,
            AuthyError::AuthFailed(_) => 2,
            AuthyError::InvalidToken => 6,
            AuthyError::TokenExpired => 6,
            AuthyError::TokenRevoked => 6,
            AuthyError::SessionNotFound(_) => 3,
            AuthyError::TokenReadOnly => 4,
            AuthyError::RunOnly => 4,
            AuthyError::Encryption(_) => 1,
            AuthyError::Decryption(_) => 2,
            AuthyError::Serialization(_) => 1,
            AuthyError::AuditChainBroken(_) => 1,
            AuthyError::InvalidKeyfile(_) => 2,
            AuthyError::Io(_) => 1,
            AuthyError::Other(_) => 1,
        }
    }

    /// Return a string error code identifier.
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthyError::VaultNotInitialized => "vault_not_initialized",
            AuthyError::VaultAlreadyExists(_) => "already_exists",
            AuthyError::SecretNotFound(_) => "not_found",
            AuthyError::SecretAlreadyExists(_) => "already_exists",
            AuthyError::PolicyNotFound(_) => "not_found",
            AuthyError::PolicyAlreadyExists(_) => "already_exists",
            AuthyError::AccessDenied { .. } => "access_denied",
            AuthyError::AuthFailed(_) => "auth_failed",
            AuthyError::InvalidToken => "invalid_token",
            AuthyError::TokenExpired => "token_expired",
            AuthyError::TokenRevoked => "token_revoked",
            AuthyError::SessionNotFound(_) => "not_found",
            AuthyError::TokenReadOnly => "token_read_only",
            AuthyError::RunOnly => "run_only",
            AuthyError::Encryption(_) => "encryption_error",
            AuthyError::Decryption(_) => "decryption_error",
            AuthyError::Serialization(_) => "serialization_error",
            AuthyError::AuditChainBroken(_) => "audit_chain_broken",
            AuthyError::InvalidKeyfile(_) => "invalid_keyfile",
            AuthyError::Io(_) => "io_error",
            AuthyError::Other(_) => "error",
        }
    }
}

/// JSON error response for --json mode.
#[derive(Serialize)]
pub struct JsonError {
    pub error: JsonErrorDetail,
}

#[derive(Serialize)]
pub struct JsonErrorDetail {
    pub code: String,
    pub message: String,
    pub exit_code: i32,
}

impl JsonError {
    pub fn from_error(e: &AuthyError) -> Self {
        Self {
            error: JsonErrorDetail {
                code: e.error_code().to_string(),
                message: e.to_string(),
                exit_code: e.exit_code(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, AuthyError>;
