use crate::audit;
use crate::auth;
use crate::cli::common;
use crate::config::project::ProjectConfig;
use crate::error::{AuthyError, Result};
use crate::subprocess::{transform_name, NamingOptions};
use crate::vault;

pub fn run(
    scope_arg: Option<&str>,
    uppercase_arg: bool,
    replace_dash_arg: Option<char>,
    prefix_arg: Option<String>,
    format: &str,
    no_export: bool,
) -> Result<()> {
    // Merge CLI args with project config
    let project = ProjectConfig::discover_from_cwd().ok().flatten();
    let project_config = project.as_ref().map(|(c, _)| c);

    let scope = scope_arg
        .map(|s| s.to_string())
        .or_else(|| project_config.map(|c| c.scope.clone()))
        .ok_or_else(|| {
            AuthyError::Other("No --scope provided and no .authy.toml found.".to_string())
        })?;

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

    let (key, auth_ctx) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    // Token-level run_only enforcement
    if auth_ctx.run_only {
        return Err(AuthyError::RunOnly);
    }

    // Policy-level run_only enforcement
    if let Some(policy) = vault.policies.get(&scope) {
        if policy.run_only {
            return Err(AuthyError::RunOnly);
        }
    }

    let secrets = common::resolve_scoped_secrets(&vault, &scope, &auth_ctx)?;

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
        Some(&format!(
            "scope={}, secrets={}, format={}",
            scope,
            secrets.len(),
            format
        )),
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
