use crate::audit;
use crate::auth;
use crate::error::{AuthyError, Result};
use crate::vault;

pub fn run(name: &str, scope: Option<&str>) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Determine the effective scope
    let effective_scope = scope
        .map(|s| s.to_string())
        .or_else(|| auth_ctx.scope.clone());

    // If a scope is active, enforce policy
    if let Some(ref scope_name) = effective_scope {
        let policy = vault
            .policies
            .get(scope_name)
            .ok_or_else(|| AuthyError::PolicyNotFound(scope_name.clone()))?;

        if !policy.can_read(name)? {
            // Audit the denial
            let material = audit::key_material(&key);
            let audit_key = audit::derive_audit_key(&material);
            audit::log_event(
                &vault::audit_path(),
                "get",
                Some(name),
                &auth_ctx.actor_name(),
                "denied",
                Some(&format!("scope={}", scope_name)),
                &audit_key,
            )?;

            return Err(AuthyError::AccessDenied {
                secret: name.to_string(),
                scope: scope_name.clone(),
            });
        }
    }

    let entry = vault
        .secrets
        .get(name)
        .ok_or_else(|| AuthyError::SecretNotFound(name.to_string()))?;

    print!("{}", entry.value);

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    let detail = effective_scope.as_deref().map(|s| format!("scope={}", s));
    audit::log_event(
        &vault::audit_path(),
        "get",
        Some(name),
        &auth_ctx.actor_name(),
        "success",
        detail.as_deref(),
        &audit_key,
    )?;

    Ok(())
}
