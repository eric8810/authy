pub mod context;

use std::env;
use std::fs;
use std::io::IsTerminal;

use crate::error::{AuthyError, Result};
use crate::session;
use crate::vault::{self, VaultKey};
use context::AuthContext;

const AUTHY_PASSPHRASE_ENV: &str = "AUTHY_PASSPHRASE";
const AUTHY_KEYFILE_ENV: &str = "AUTHY_KEYFILE";
const AUTHY_TOKEN_ENV: &str = "AUTHY_TOKEN";
const AUTHY_NON_INTERACTIVE_ENV: &str = "AUTHY_NON_INTERACTIVE";

/// Check if we are in non-interactive mode.
/// Returns true if stdin is not a TTY or AUTHY_NON_INTERACTIVE=1 is set.
pub fn is_non_interactive() -> bool {
    if env::var(AUTHY_NON_INTERACTIVE_ENV)
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }
    !std::io::stdin().is_terminal()
}

/// Resolve authentication. Tries in order:
/// 1. AUTHY_TOKEN env var (session token, requires AUTHY_KEYFILE for vault decryption)
/// 2. AUTHY_KEYFILE env var (master keyfile)
/// 3. AUTHY_PASSPHRASE env var (master passphrase)
/// 4. Interactive passphrase prompt (only if TTY is available)
pub fn resolve_auth(require_write: bool) -> Result<(VaultKey, AuthContext)> {
    // Check for token-based auth first
    if let Ok(token) = env::var(AUTHY_TOKEN_ENV) {
        if require_write {
            return Err(AuthyError::TokenReadOnly);
        }

        // Token auth requires a keyfile to decrypt the vault
        let keyfile_path = env::var(AUTHY_KEYFILE_ENV)
            .map_err(|_| AuthyError::AuthFailed(
                "AUTHY_TOKEN requires AUTHY_KEYFILE to be set".into(),
            ))?;

        let (identity, pubkey) = read_keyfile(&keyfile_path)?;
        let vault_key = VaultKey::Keyfile {
            identity: identity.clone(),
            pubkey,
        };

        // Load the vault to validate the token
        let vault = vault::load_vault(&vault_key)?;
        let hmac_key = vault::crypto::derive_key(identity.as_bytes(), b"session-hmac", 32);
        let session_record = session::validate_token(&token, &vault.sessions, &hmac_key)?;

        let auth_ctx = AuthContext::from_token(
            session_record.id.clone(),
            session_record.scope.clone(),
            session_record.run_only,
        );

        return Ok((vault_key, auth_ctx));
    }

    // Check for keyfile auth
    if let Ok(keyfile_path) = env::var(AUTHY_KEYFILE_ENV) {
        let (identity, pubkey) = read_keyfile(&keyfile_path)?;
        let vault_key = VaultKey::Keyfile { identity, pubkey };
        let auth_ctx = AuthContext::master_keyfile();
        return Ok((vault_key, auth_ctx));
    }

    // Check for passphrase env var
    if let Ok(passphrase) = env::var(AUTHY_PASSPHRASE_ENV) {
        let vault_key = VaultKey::Passphrase(passphrase);
        let auth_ctx = AuthContext::master_passphrase();
        return Ok((vault_key, auth_ctx));
    }

    // Non-interactive mode: fail immediately without prompting
    if is_non_interactive() {
        return Err(AuthyError::AuthFailed(
            "No credentials provided. Set AUTHY_KEYFILE, AUTHY_PASSPHRASE, or AUTHY_TOKEN environment variable.".into(),
        ));
    }

    // Interactive passphrase prompt
    let passphrase = dialoguer::Password::new()
        .with_prompt("Enter vault passphrase")
        .interact()
        .map_err(|e| AuthyError::AuthFailed(format!("Failed to read passphrase: {}", e)))?;

    let vault_key = VaultKey::Passphrase(passphrase);
    let auth_ctx = AuthContext::master_passphrase();
    Ok((vault_key, auth_ctx))
}

/// Resolve auth specifically for init (no vault exists yet, just get the key).
pub fn resolve_auth_for_init(
    passphrase: Option<String>,
    generate_keyfile: Option<String>,
) -> Result<VaultKey> {
    if let Some(keyfile_path) = generate_keyfile {
        let (secret_key, public_key) = vault::crypto::generate_keypair();
        // Write the keyfile (secret key)
        fs::write(&keyfile_path, &secret_key)?;
        // Restrict permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&keyfile_path, fs::Permissions::from_mode(0o600))?;
        }
        // Write the public key alongside
        let pubkey_path = format!("{}.pub", keyfile_path);
        fs::write(&pubkey_path, &public_key)?;

        eprintln!("Generated keyfile: {}", keyfile_path);
        eprintln!("Public key: {}", pubkey_path);

        return Ok(VaultKey::Keyfile {
            identity: secret_key,
            pubkey: public_key,
        });
    }

    if let Some(pass) = passphrase {
        return Ok(VaultKey::Passphrase(pass));
    }

    // Check env
    if let Ok(pass) = env::var(AUTHY_PASSPHRASE_ENV) {
        return Ok(VaultKey::Passphrase(pass));
    }

    // Interactive
    let pass = dialoguer::Password::new()
        .with_prompt("Create vault passphrase")
        .with_confirmation("Confirm passphrase", "Passphrases don't match")
        .interact()
        .map_err(|e| AuthyError::AuthFailed(format!("Failed to read passphrase: {}", e)))?;

    Ok(VaultKey::Passphrase(pass))
}

/// Read an age keyfile from disk. Returns (identity_string, public_key_string).
fn read_keyfile(path: &str) -> Result<(String, String)> {
    let content = fs::read_to_string(path)
        .map_err(|e| AuthyError::InvalidKeyfile(format!("Cannot read {}: {}", path, e)))?;

    let identity: age::x25519::Identity = content
        .trim()
        .parse()
        .map_err(|e: &str| AuthyError::InvalidKeyfile(e.to_string()))?;

    let pubkey = identity.to_public().to_string();
    Ok((content.trim().to_string(), pubkey))
}
