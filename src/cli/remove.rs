use authy::audit;
use authy::auth;
use authy::error::{AuthyError, Result};
use authy::vault;

pub fn run(name: &str) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if vault.secrets.remove(name).is_none() {
        return Err(AuthyError::SecretNotFound(name.to_string()));
    }

    vault.touch();
    vault::save_vault(&vault, &key)?;

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "remove",
        Some(name),
        &auth_ctx.actor_name(),
        "success",
        None,
        &audit_key,
    )?;

    eprintln!("Secret '{}' removed.", name);
    Ok(())
}
