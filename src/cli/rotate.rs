use std::io::{self, Read};

use authy::audit;
use authy::auth;
use authy::error::{AuthyError, Result};
use authy::vault;

pub fn run(name: &str) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if !vault.secrets.contains_key(name) {
        return Err(AuthyError::SecretNotFound(name.to_string()));
    }

    // Read new value from stdin
    let mut value = String::new();
    io::stdin()
        .read_to_string(&mut value)
        .map_err(|e| AuthyError::Other(format!("Failed to read from stdin: {}", e)))?;
    let value = value.trim_end_matches('\n').to_string();

    let entry = vault.secrets.get_mut(name).unwrap();
    entry.value = value;
    entry.metadata.bump_version();
    let version = entry.metadata.version;

    vault.touch();

    vault::save_vault(&vault, &key)?;

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "rotate",
        Some(name),
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("version={}", version)),
        &audit_key,
    )?;

    eprintln!("Secret '{}' rotated to version {}.", name, version);
    Ok(())
}
