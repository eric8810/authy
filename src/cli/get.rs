use authy::audit;
use authy::auth;
use crate::cli::json_output::GetResponse;
use authy::error::{AuthyError, Result};
use authy::vault;

pub fn run(name: &str, scope: Option<&str>, json: bool) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Token-level run_only enforcement
    if auth_ctx.run_only {
        return Err(AuthyError::RunOnly);
    }

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

        // Policy-level run_only enforcement
        if policy.run_only {
            return Err(AuthyError::RunOnly);
        }

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

    if json {
        let response = GetResponse {
            name: name.to_string(),
            value: entry.value.clone(),
            version: entry.metadata.version,
            created: entry.metadata.created_at.to_rfc3339(),
            modified: entry.metadata.modified_at.to_rfc3339(),
        };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| authy::error::AuthyError::Serialization(e.to_string()))?
        );
    } else {
        print!("{}", entry.value);
    }

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
