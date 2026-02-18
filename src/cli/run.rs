use crate::audit;
use crate::auth;
use crate::cli::common;
use crate::config::project::ProjectConfig;
use crate::error::{AuthyError, Result};
use crate::subprocess::{self, NamingOptions};
use crate::vault;

pub fn run(
    scope_arg: Option<&str>,
    uppercase_arg: bool,
    replace_dash_arg: Option<char>,
    prefix_arg: Option<String>,
    command: &[String],
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

    let secrets = common::resolve_scoped_secrets(&vault, &scope, &auth_ctx)?;

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
            secrets.len(),
            command.first().map(|s| s.as_str()).unwrap_or("?")
        )),
        &audit_key,
    )?;

    let exit_code = subprocess::run_with_secrets(command, &secrets, &naming)?;
    std::process::exit(exit_code);
}
