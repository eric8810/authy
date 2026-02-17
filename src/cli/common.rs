use std::collections::HashMap;

use crate::error::{AuthyError, Result};
use crate::auth::context::AuthContext;
use crate::vault::Vault;

/// Resolve secrets accessible under a given scope (policy name).
/// Returns a HashMap of secret_name -> secret_value for all allowed secrets.
pub fn resolve_scoped_secrets(
    vault: &Vault,
    scope: &str,
    auth_ctx: &AuthContext,
) -> Result<HashMap<String, String>> {
    // Determine effective scope: explicit scope or from token
    let effective_scope = if !scope.is_empty() {
        scope.to_string()
    } else if let Some(ref token_scope) = auth_ctx.scope {
        token_scope.clone()
    } else {
        return Ok(HashMap::new());
    };

    let policy = vault
        .policies
        .get(&effective_scope)
        .ok_or_else(|| AuthyError::PolicyNotFound(effective_scope.clone()))?;

    let names: Vec<&str> = vault.secrets.keys().map(|s| s.as_str()).collect();
    let allowed = policy.filter_secrets(&names)?;

    let mut secrets = HashMap::new();
    for name in &allowed {
        if let Some(entry) = vault.secrets.get(*name) {
            secrets.insert(name.to_string(), entry.value.clone());
        }
    }

    Ok(secrets)
}
