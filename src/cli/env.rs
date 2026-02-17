use crate::audit;
use crate::auth;
use crate::cli::common;
use crate::error::{AuthyError, Result};
use crate::subprocess::{transform_name, NamingOptions};
use crate::vault;

pub fn run(
    scope: &str,
    uppercase: bool,
    replace_dash: Option<char>,
    prefix: Option<String>,
    format: &str,
    no_export: bool,
) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Token-level run_only enforcement
    if auth_ctx.run_only {
        return Err(AuthyError::RunOnly);
    }

    // Policy-level run_only enforcement
    if let Some(policy) = vault.policies.get(scope) {
        if policy.run_only {
            return Err(AuthyError::RunOnly);
        }
    }

    let secrets = common::resolve_scoped_secrets(&vault, scope, &auth_ctx)?;

    let naming = NamingOptions {
        uppercase,
        replace_dash,
        prefix,
    };

    // Sort keys for deterministic output
    let mut pairs: Vec<(String, String)> = secrets
        .iter()
        .map(|(name, value)| (transform_name(name, &naming), value.clone()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    match format {
        "shell" => {
            for (key, value) in &pairs {
                let escaped = shell_escape(value);
                if no_export {
                    println!("{}='{}'", key, escaped);
                } else {
                    println!("export {}='{}'", key, escaped);
                }
            }
        }
        "dotenv" => {
            for (key, value) in &pairs {
                let quoted = dotenv_quote(value);
                println!("{}={}", key, quoted);
            }
        }
        "json" => {
            let map: serde_json::Map<String, serde_json::Value> = pairs
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(map))
                    .map_err(|e| AuthyError::Serialization(e.to_string()))?
            );
        }
        other => {
            return Err(AuthyError::Other(format!(
                "Unknown format '{}'. Use 'shell', 'dotenv', or 'json'.",
                other
            )));
        }
    }

    // Audit log
    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "env_export",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("scope={}, secrets={}, format={}", scope, secrets.len(), format)),
        &audit_key,
    )?;

    Ok(())
}

/// Shell-escape a value for single-quoted POSIX shell strings.
/// Replaces `'` with `'\''`.
fn shell_escape(value: &str) -> String {
    value.replace('\'', "'\\''")
}

/// Quote a value for dotenv format.
/// If it contains special chars, wrap in double quotes and escape.
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
