use crate::audit;
use crate::auth;
use crate::config::Config;
use crate::error::{AuthyError, Result};
use crate::vault::{self, Vault};

pub fn run(passphrase: Option<String>, generate_keyfile: Option<String>) -> Result<()> {
    if vault::is_initialized() {
        return Err(AuthyError::VaultAlreadyExists(
            vault::vault_path().display().to_string(),
        ));
    }

    let key = auth::resolve_auth_for_init(passphrase, generate_keyfile)?;

    // Create empty vault
    let vault = Vault::new();
    vault::save_vault(&vault, &key)?;

    // Write default config
    let config = Config::default();
    config.save(&vault::config_path())?;

    // Log the init event
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "init",
        None,
        "master",
        "success",
        None,
        &audit_key,
    )?;

    eprintln!("Vault initialized at {}", vault::authy_dir().display());
    Ok(())
}
