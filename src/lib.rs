//! Authy — encrypted secrets vault with policy-based ACL, session tokens, and audit logging.
//!
//! This library exposes the core vault, authentication, policy, session, and audit
//! modules for programmatic use. The CLI and TUI are gated behind the `cli` feature
//! and are private to the binary.
//!
//! # Quick start
//!
//! ```no_run
//! use authy::api::AuthyClient;
//!
//! let client = AuthyClient::with_passphrase("my-vault-passphrase")?;
//! client.store("api-key", "sk-secret-value", false)?;
//! let value = client.get("api-key")?;
//! # Ok::<(), authy::error::AuthyError>(())
//! ```

pub mod api;
pub mod audit;
pub mod auth;
pub mod config;
pub mod error;
pub mod policy;
pub mod session;
pub mod subprocess;
pub mod types;
pub mod vault;
