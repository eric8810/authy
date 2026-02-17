use crate::audit;
use crate::auth;
use crate::cli::SessionCommands;
use crate::error::{AuthyError, Result};
use crate::session::{self, SessionRecord};
use crate::vault;

pub fn run(cmd: &SessionCommands) -> Result<()> {
    match cmd {
        SessionCommands::Create { scope, ttl, label } => {
            create(scope, ttl, label.as_deref())
        }
        SessionCommands::List => list(),
        SessionCommands::Revoke { id } => revoke(id),
        SessionCommands::RevokeAll => revoke_all(),
    }
}

fn create(scope: &str, ttl: &str, label: Option<&str>) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    // Verify the scope/policy exists
    if !vault.policies.contains_key(scope) {
        return Err(AuthyError::PolicyNotFound(scope.to_string()));
    }

    let duration = session::parse_ttl(ttl)?;
    let now = chrono::Utc::now();
    let expires_at = now + duration;

    // Derive the HMAC key for token generation
    let material = audit::key_material(&key);
    let hmac_key = crate::vault::crypto::derive_key(&material, b"session-hmac", 32);

    let (token, token_hmac) = session::generate_token(&hmac_key);
    let session_id = session::generate_session_id();

    let record = SessionRecord {
        id: session_id.clone(),
        scope: scope.to_string(),
        token_hmac,
        created_at: now,
        expires_at,
        revoked: false,
        label: label.map(|s| s.to_string()),
    };

    vault.sessions.push(record);
    vault.touch();
    vault::save_vault(&vault, &key)?;

    // Audit log
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "session.create",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("session={}, scope={}, ttl={}", session_id, scope, ttl)),
        &audit_key,
    )?;

    // Print the token to stdout (the only time it's ever shown)
    println!("{}", token);
    eprintln!("Session '{}' created (scope={}, expires={})", session_id, scope, expires_at);
    Ok(())
}

fn list() -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    if vault.sessions.is_empty() {
        eprintln!("No sessions.");
        return Ok(());
    }

    let now = chrono::Utc::now();
    for session in &vault.sessions {
        let status = if session.revoked {
            "revoked".to_string()
        } else if now > session.expires_at {
            "expired".to_string()
        } else {
            "active".to_string()
        };

        let label = session.label.as_deref().unwrap_or("-");
        println!(
            "{:<16} scope={:<16} status={:<8} label={} expires={}",
            session.id, session.scope, status, label, session.expires_at
        );
    }

    Ok(())
}

fn revoke(id: &str) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    let session = vault
        .sessions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| AuthyError::SessionNotFound(id.to_string()))?;

    session.revoked = true;
    vault.touch();
    vault::save_vault(&vault, &key)?;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "session.revoke",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("session={}", id)),
        &audit_key,
    )?;

    eprintln!("Session '{}' revoked.", id);
    Ok(())
}

fn revoke_all() -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    let count = vault
        .sessions
        .iter_mut()
        .filter(|s| !s.revoked)
        .map(|s| s.revoked = true)
        .count();

    vault.touch();
    vault::save_vault(&vault, &key)?;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "session.revoke_all",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("count={}", count)),
        &audit_key,
    )?;

    eprintln!("{} session(s) revoked.", count);
    Ok(())
}
