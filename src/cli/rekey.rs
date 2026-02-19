use std::fs;

use crate::audit;
use crate::auth;
use crate::error::{AuthyError, Result};
use crate::vault;

pub fn run(
    generate_keyfile: Option<&str>,
    to_passphrase: bool,
    new_keyfile: Option<&str>,
) -> Result<()> {
    // Validate mutual exclusivity
    let flag_count =
        generate_keyfile.is_some() as u8 + to_passphrase as u8 + new_keyfile.is_some() as u8;
    if flag_count > 1 {
        return Err(AuthyError::Other(
            "Only one of --generate-keyfile, --to-passphrase, or --new-keyfile can be specified."
                .to_string(),
        ));
    }

    // Auth with old credentials (require write access — no tokens)
    let (old_key, auth_ctx) = auth::resolve_auth(true)?;
    let vault = vault::load_vault(&old_key)?;

    // Determine new key
    let new_key = if let Some(keyfile_path) = generate_keyfile {
        // Generate a new keyfile
        let (secret_key, public_key) = vault::crypto::generate_keypair();
        fs::write(keyfile_path, &secret_key)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(keyfile_path, fs::Permissions::from_mode(0o600))?;
        }
        let pubkey_path = format!("{}.pub", keyfile_path);
        fs::write(&pubkey_path, &public_key)?;
        eprintln!("Generated new keyfile: {}", keyfile_path);
        eprintln!("Public key: {}", pubkey_path);
        vault::VaultKey::Keyfile {
            identity: secret_key,
            pubkey: public_key,
        }
    } else if let Some(keyfile_path) = new_keyfile {
        // Read existing keyfile
        let (identity, pubkey) = auth::read_keyfile(keyfile_path)?;
        vault::VaultKey::Keyfile { identity, pubkey }
    } else {
        // Prompt for new passphrase (default behavior, also handles --to-passphrase)
        if auth::is_non_interactive() {
            return Err(AuthyError::AuthFailed(
                "Cannot prompt for new passphrase in non-interactive mode.".to_string(),
            ));
        }
        let passphrase = dialoguer::Password::new()
            .with_prompt("Enter new vault passphrase")
            .with_confirmation("Confirm new passphrase", "Passphrases don't match")
            .interact()
            .map_err(|e| AuthyError::AuthFailed(format!("Failed to read passphrase: {}", e)))?;
        vault::VaultKey::Passphrase(passphrase)
    };

    // Save vault with new key
    vault::save_vault(&vault, &new_key)?;

    // Audit log with NEW key material (so the chain continues with new key)
    let material = audit::key_material(&new_key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "rekey",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some("vault re-encrypted with new credentials"),
        &audit_key,
    )?;

    eprintln!("Vault re-encrypted successfully.");
    eprintln!("Warning: all existing session tokens are now invalidated.");

    Ok(())
}
