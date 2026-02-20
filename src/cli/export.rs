use serde::Serialize;

use authy::audit;
use authy::auth;
use crate::cli::common;
use authy::config::project::ProjectConfig;
use authy::error::{AuthyError, Result};
use authy::subprocess::{transform_name, NamingOptions};
use authy::vault;

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
    scope_arg: Option<&str>,
    uppercase_arg: bool,
    replace_dash_arg: Option<char>,
    prefix_arg: Option<String>,
) -> Result<()> {
    // Merge CLI args with project config (scope remains optional for export)
    let project = ProjectConfig::discover_from_cwd().ok().flatten();
    let project_config = project.as_ref().map(|(c, _)| c);

    let scope = scope_arg
        .map(|s| s.to_string())
        .or_else(|| project_config.map(|c| c.scope.clone()));

    let uppercase = uppercase_arg || project_config.is_some_and(|c| c.uppercase);
    let replace_dash =
        replace_dash_arg.or_else(|| project_config.and_then(|c| c.replace_dash_char()));
    let prefix = prefix_arg.or_else(|| project_config.and_then(|c| c.prefix.clone()));

    // If project has keyfile and AUTHY_KEYFILE not set, set it
    if std::env::var("AUTHY_KEYFILE").is_err() {
        if let Some(kf) = project_config.and_then(|c| c.expanded_keyfile()) {
            std::env::set_var("AUTHY_KEYFILE", &kf);
        }
    }

    // Without scope: require master auth (reject tokens)
    let require_write = scope.is_none();
    let (key, auth_ctx) = auth::resolve_auth(require_write)?;
    let vault_data = vault::load_vault(&key)?;

    // Token-level run_only enforcement
    if auth_ctx.run_only {
        return Err(AuthyError::RunOnly);
    }

    // Policy-level run_only enforcement
    if let Some(ref scope_name) = scope {
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
            if let Some(ref scope) = scope {
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
            if let Some(ref scope) = scope {
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
        Some(ref s) => format!("format={}, scope={}", format, s),
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
