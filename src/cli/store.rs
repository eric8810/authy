use std::io::{self, Read};

use authy::audit;
use authy::auth;
use authy::error::{AuthyError, Result};
use authy::vault::{self, secret::SecretEntry};

pub fn run(name: &str, force: bool) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if vault.secrets.contains_key(name) && !force {
        return Err(AuthyError::SecretAlreadyExists(name.to_string()));
    }

    // Read secret value from stdin
    let mut value = String::new();
    io::stdin()
        .read_to_string(&mut value)
        .map_err(|e| AuthyError::Other(format!("Failed to read from stdin: {}", e)))?;

    // Trim trailing newline (common when piping echo)
    let value = value.trim_end_matches('\n').to_string();

    let is_update = vault.secrets.contains_key(name);
    vault.secrets.insert(name.to_string(), SecretEntry::new(value));
    vault.touch();

    vault::save_vault(&vault, &key)?;

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    let op = if is_update { "update" } else { "store" };
    audit::log_event(
        &vault::audit_path(),
        op,
        Some(name),
        &auth_ctx.actor_name(),
        "success",
        None,
        &audit_key,
    )?;

    eprintln!(
        "Secret '{}' {}.",
        name,
        if is_update { "updated" } else { "stored" }
    );
    Ok(())
}
