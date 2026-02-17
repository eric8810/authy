use crate::audit;
use crate::auth;
use crate::cli::json_output::{ListResponse, SecretListItem};
use crate::error::{AuthyError, Result};
use crate::vault;

pub fn run(scope: Option<&str>, json: bool) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    let effective_scope = scope
        .map(|s| s.to_string())
        .or_else(|| auth_ctx.scope.clone());

    let names: Vec<&str> = vault.secrets.keys().map(|s| s.as_str()).collect();

    let filtered = if let Some(ref scope_name) = effective_scope {
        let policy = vault
            .policies
            .get(scope_name)
            .ok_or_else(|| AuthyError::PolicyNotFound(scope_name.clone()))?;
        policy.filter_secrets(&names)?
    } else {
        names
    };

    if json {
        let secrets: Vec<SecretListItem> = filtered
            .iter()
            .filter_map(|name| {
                vault.secrets.get(*name).map(|entry| SecretListItem {
                    name: name.to_string(),
                    version: entry.metadata.version,
                    created: entry.metadata.created_at.to_rfc3339(),
                    modified: entry.metadata.modified_at.to_rfc3339(),
                })
            })
            .collect();
        let response = ListResponse { secrets };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| crate::error::AuthyError::Serialization(e.to_string()))?
        );
    } else {
        for name in &filtered {
            println!("{}", name);
        }
    }

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    let detail = effective_scope.as_deref().map(|s| format!("scope={}", s));
    audit::log_event(
        &vault::audit_path(),
        "list",
        None,
        &auth_ctx.actor_name(),
        "success",
        detail.as_deref(),
        &audit_key,
    )?;

    Ok(())
}
