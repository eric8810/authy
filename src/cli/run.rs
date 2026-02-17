use std::collections::HashMap;

use crate::audit;
use crate::auth;
use crate::error::{AuthyError, Result};
use crate::subprocess::{self, NamingOptions};
use crate::vault;

pub fn run(
    scope: &str,
    uppercase: bool,
    replace_dash: Option<char>,
    prefix: Option<String>,
    command: &[String],
) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Look up the policy
    let policy = vault
        .policies
        .get(scope)
        .ok_or_else(|| AuthyError::PolicyNotFound(scope.to_string()))?;

    // Collect allowed secrets
    let names: Vec<&str> = vault.secrets.keys().map(|s| s.as_str()).collect();
    let allowed = policy.filter_secrets(&names)?;

    let mut secrets = HashMap::new();
    for name in &allowed {
        if let Some(entry) = vault.secrets.get(*name) {
            secrets.insert(name.to_string(), entry.value.clone());
        }
    }

    let naming = NamingOptions {
        uppercase,
        replace_dash,
        prefix,
    };

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "run",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!(
            "scope={}, secrets={}, cmd={}",
            scope,
            allowed.len(),
            command.first().map(|s| s.as_str()).unwrap_or("?")
        )),
        &audit_key,
    )?;

    let exit_code = subprocess::run_with_secrets(command, &secrets, &naming)?;
    std::process::exit(exit_code);
}
