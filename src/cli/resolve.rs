use std::fs;

use authy::audit;
use authy::auth;
use crate::cli::common;
use authy::config::project::ProjectConfig;
use authy::error::{AuthyError, Result};
use authy::vault;

pub fn run(file: &str, output: Option<&str>, scope_arg: Option<&str>) -> Result<()> {
    // Merge scope from CLI arg / .authy.toml / token scope
    let project = ProjectConfig::discover_from_cwd().ok().flatten();
    let project_config = project.as_ref().map(|(c, _)| c);

    let scope = scope_arg
        .map(|s| s.to_string())
        .or_else(|| project_config.map(|c| c.scope.clone()));

    // If project has keyfile and AUTHY_KEYFILE not set, set it
    if std::env::var("AUTHY_KEYFILE").is_err() {
        if let Some(kf) = project_config.and_then(|c| c.expanded_keyfile()) {
            std::env::set_var("AUTHY_KEYFILE", &kf);
        }
    }

    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Do NOT check auth_ctx.run_only — resolve is a safe command (like run)

    let scope = scope
        .or_else(|| auth_ctx.scope.clone())
        .ok_or_else(|| {
            AuthyError::Other("No --scope provided and no .authy.toml found.".to_string())
        })?;

    let secrets = common::resolve_scoped_secrets(&vault, &scope, &auth_ctx)?;

    // Read source file
    let content = fs::read_to_string(file)
        .map_err(|e| AuthyError::Other(format!("Cannot read file '{}': {}", file, e)))?;

    // Find and replace all <authy:KEY> placeholders
    let mut result = String::with_capacity(content.len());
    let mut rest = content.as_str();
    let mut keys_resolved = 0u32;

    while let Some(start) = rest.find("<authy:") {
        result.push_str(&rest[..start]);
        let after_prefix = &rest[start + 7..]; // skip "<authy:"

        if let Some(end) = after_prefix.find('>') {
            let key_name = &after_prefix[..end];

            // Validate key name: [a-z0-9][a-z0-9-]*
            if key_name.is_empty() || !is_valid_key_name(key_name) {
                // Not a valid placeholder, pass through literally
                result.push_str(&rest[start..start + 7 + end + 1]);
                rest = &after_prefix[end + 1..];
                continue;
            }

            let value = secrets.get(key_name).ok_or_else(|| {
                if vault.secrets.contains_key(key_name) {
                    AuthyError::AccessDenied {
                        secret: key_name.to_string(),
                        scope: scope.clone(),
                    }
                } else {
                    AuthyError::SecretNotFound(key_name.to_string())
                }
            })?;

            result.push_str(value);
            keys_resolved += 1;
            rest = &after_prefix[end + 1..];
        } else {
            // No closing '>', pass through rest
            result.push_str(&rest[start..]);
            rest = "";
        }
    }
    result.push_str(rest);

    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, &result)
            .map_err(|e| AuthyError::Other(format!("Cannot write to '{}': {}", output_path, e)))?;
        eprintln!("Resolved {} placeholder(s) → {}", keys_resolved, output_path);
    } else {
        print!("{}", result);
    }

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "resolve",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!(
            "scope={}, file={}, keys={}",
            scope, file, keys_resolved
        )),
        &audit_key,
    )?;

    Ok(())
}

/// Check if a key name matches [a-z0-9][a-z0-9-]*
fn is_valid_key_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
