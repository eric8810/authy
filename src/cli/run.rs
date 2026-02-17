use crate::audit;
use crate::auth;
use crate::cli::common;
use crate::error::Result;
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

    let secrets = common::resolve_scoped_secrets(&vault, scope, &auth_ctx)?;

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
