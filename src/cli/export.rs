use serde::Serialize;

use crate::audit;
use crate::auth;
use crate::cli::common;
use crate::error::{AuthyError, Result};
use crate::subprocess::{transform_name, NamingOptions};
use crate::vault;

#[derive(Serialize)]
struct ExportJsonEntry {
    name: String,
    value: String,
    version: u32,
    created: String,
    modified: String,
}

pub fn run(
    format: &str,
    scope: Option<&str>,
    uppercase: bool,
    replace_dash: Option<char>,
    prefix: Option<String>,
) -> Result<()> {
    // Without scope: require master auth (reject tokens)
    let require_write = scope.is_none();
    let (key, auth_ctx) = auth::resolve_auth(require_write)?;
    let vault_data = vault::load_vault(&key)?;

    // Token-level run_only enforcement
    if auth_ctx.run_only {
        return Err(AuthyError::RunOnly);
    }

    // Policy-level run_only enforcement
    if let Some(scope_name) = scope {
        if let Some(policy) = vault_data.policies.get(scope_name) {
            if policy.run_only {
                return Err(AuthyError::RunOnly);
            }
        }
    }

    let naming = NamingOptions {
        uppercase,
        replace_dash,
        prefix,
    };

    match format {
        "env" => {
            if let Some(scope) = scope {
                let secrets = common::resolve_scoped_secrets(&vault_data, scope, &auth_ctx)?;
                let mut pairs: Vec<(String, String)> = secrets
                    .iter()
                    .map(|(name, value)| (transform_name(name, &naming), value.clone()))
                    .collect();
                pairs.sort_by(|a, b| a.0.cmp(&b.0));

                for (key, value) in &pairs {
                    println!("{}={}", key, dotenv_quote(value));
                }
            } else {
                // Export all secrets (master auth required)
                let mut pairs: Vec<(String, &str)> = vault_data
                    .secrets
                    .iter()
                    .map(|(name, entry)| (transform_name(name, &naming), entry.value.as_str()))
                    .collect();
                pairs.sort_by(|a, b| a.0.cmp(&b.0));

                for (key, value) in &pairs {
                    println!("{}={}", key, dotenv_quote(value));
                }
            }
        }
        "json" => {
            if let Some(scope) = scope {
                let secrets = common::resolve_scoped_secrets(&vault_data, scope, &auth_ctx)?;
                let mut entries: Vec<ExportJsonEntry> = secrets
                    .keys()
                    .filter_map(|name| {
                        vault_data.secrets.get(name).map(|entry| ExportJsonEntry {
                            name: transform_name(name, &naming),
                            value: entry.value.clone(),
                            version: entry.metadata.version,
                            created: entry.metadata.created_at.to_rfc3339(),
                            modified: entry.metadata.modified_at.to_rfc3339(),
                        })
                    })
                    .collect();
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entries)
                        .map_err(|e| AuthyError::Serialization(e.to_string()))?
                );
            } else {
                let mut entries: Vec<ExportJsonEntry> = vault_data
                    .secrets
                    .iter()
                    .map(|(name, entry)| ExportJsonEntry {
                        name: transform_name(name, &naming),
                        value: entry.value.clone(),
                        version: entry.metadata.version,
                        created: entry.metadata.created_at.to_rfc3339(),
                        modified: entry.metadata.modified_at.to_rfc3339(),
                    })
                    .collect();
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entries)
                        .map_err(|e| AuthyError::Serialization(e.to_string()))?
                );
            }
        }
        other => {
            return Err(AuthyError::Other(format!(
                "Unknown format '{}'. Use 'env' or 'json'.",
                other
            )));
        }
    }

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    let detail = match scope {
        Some(s) => format!("format={}, scope={}", format, s),
        None => format!("format={}, scope=all", format),
    };
    audit::log_event(
        &vault::audit_path(),
        "export",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&detail),
        &audit_key,
    )?;

    Ok(())
}

/// Quote a value for dotenv format.
fn dotenv_quote(value: &str) -> String {
    if value.is_empty() {
        return "\"\"".to_string();
    }

    let needs_quoting = value.contains(|c: char| {
        c == ' '
            || c == '#'
            || c == '"'
            || c == '\''
            || c == '\\'
            || c == '\n'
            || c == '\r'
            || c == '\t'
            || c == '$'
            || c == '`'
    });

    if needs_quoting {
        let escaped = value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}
