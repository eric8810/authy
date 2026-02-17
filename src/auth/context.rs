/// The resolved authentication context after verifying credentials.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    /// The authentication method used.
    pub method: AuthMethod,
    /// Optional scope for policy-based access (set when using tokens).
    pub scope: Option<String>,
    /// Whether this context allows write operations.
    pub can_write: bool,
    /// When true, secrets can only be injected via `run` — `get`, `env`, `export` are blocked.
    pub run_only: bool,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Passphrase,
    Keyfile,
    SessionToken { session_id: String },
}

impl AuthContext {
    pub fn master_passphrase() -> Self {
        Self {
            method: AuthMethod::Passphrase,
            scope: None,
            can_write: true,
            run_only: false,
        }
    }

    pub fn master_keyfile() -> Self {
        Self {
            method: AuthMethod::Keyfile,
            scope: None,
            can_write: true,
            run_only: false,
        }
    }

    pub fn from_token(session_id: String, scope: String, run_only: bool) -> Self {
        Self {
            method: AuthMethod::SessionToken { session_id },
            scope: Some(scope),
            can_write: false,
            run_only,
        }
    }

    pub fn actor_name(&self) -> String {
        match &self.method {
            AuthMethod::Passphrase => "master(passphrase)".to_string(),
            AuthMethod::Keyfile => "master(keyfile)".to_string(),
            AuthMethod::SessionToken { session_id } => format!("token({})", session_id),
        }
    }
}
