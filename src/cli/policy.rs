use crate::audit;
use crate::auth;
use crate::cli::PolicyCommands;
use crate::error::{AuthyError, Result};
use crate::policy::Policy;
use crate::vault;

pub fn run(cmd: &PolicyCommands) -> Result<()> {
    match cmd {
        PolicyCommands::Create {
            name,
            allow,
            deny,
            description,
        } => create(name, allow, deny, description.as_deref()),
        PolicyCommands::Show { name } => show(name),
        PolicyCommands::Update {
            name,
            allow,
            deny,
            description,
        } => update(name, allow.as_deref(), deny.as_deref(), description.as_deref()),
        PolicyCommands::List => list(),
        PolicyCommands::Remove { name } => remove(name),
        PolicyCommands::Test { scope, name } => test(scope, name),
    }
}

fn create(name: &str, allow: &[String], deny: &[String], description: Option<&str>) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if vault.policies.contains_key(name) {
        return Err(AuthyError::PolicyAlreadyExists(name.to_string()));
    }

    let mut policy = Policy::new(name.to_string(), allow.to_vec(), deny.to_vec());
    policy.description = description.map(|s| s.to_string());

    vault.policies.insert(name.to_string(), policy);
    vault.touch();
    vault::save_vault(&vault, &key)?;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "policy.create",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("policy={}", name)),
        &audit_key,
    )?;

    eprintln!("Policy '{}' created.", name);
    Ok(())
}

fn show(name: &str) -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    let policy = vault
        .policies
        .get(name)
        .ok_or_else(|| AuthyError::PolicyNotFound(name.to_string()))?;

    println!("Policy: {}", policy.name);
    if let Some(ref desc) = policy.description {
        println!("Description: {}", desc);
    }
    println!("Allow patterns:");
    for p in &policy.allow {
        println!("  + {}", p);
    }
    println!("Deny patterns:");
    if policy.deny.is_empty() {
        println!("  (none)");
    } else {
        for p in &policy.deny {
            println!("  - {}", p);
        }
    }
    println!("Created: {}", policy.created_at);
    println!("Modified: {}", policy.modified_at);

    Ok(())
}

fn update(
    name: &str,
    allow: Option<&[String]>,
    deny: Option<&[String]>,
    description: Option<&str>,
) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    let policy = vault
        .policies
        .get_mut(name)
        .ok_or_else(|| AuthyError::PolicyNotFound(name.to_string()))?;

    if let Some(allow) = allow {
        policy.allow = allow.to_vec();
    }
    if let Some(deny) = deny {
        policy.deny = deny.to_vec();
    }
    if let Some(desc) = description {
        policy.description = Some(desc.to_string());
    }
    policy.modified_at = chrono::Utc::now();
    vault.touch();
    vault::save_vault(&vault, &key)?;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "policy.update",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("policy={}", name)),
        &audit_key,
    )?;

    eprintln!("Policy '{}' updated.", name);
    Ok(())
}

fn list() -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    if vault.policies.is_empty() {
        eprintln!("No policies defined.");
        return Ok(());
    }

    for (name, policy) in &vault.policies {
        let desc = policy
            .description
            .as_deref()
            .unwrap_or("(no description)");
        println!(
            "{:<20} allow:{} deny:{} — {}",
            name,
            policy.allow.len(),
            policy.deny.len(),
            desc
        );
    }

    Ok(())
}

fn remove(name: &str) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if vault.policies.remove(name).is_none() {
        return Err(AuthyError::PolicyNotFound(name.to_string()));
    }

    vault.touch();
    vault::save_vault(&vault, &key)?;

    let material = audit::key_material(&key);
    let audit_key = audit::derive_audit_key(&material);
    audit::log_event(
        &vault::audit_path(),
        "policy.remove",
        None,
        &auth_ctx.actor_name(),
        "success",
        Some(&format!("policy={}", name)),
        &audit_key,
    )?;

    eprintln!("Policy '{}' removed.", name);
    Ok(())
}

fn test(scope: &str, secret_name: &str) -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    let policy = vault
        .policies
        .get(scope)
        .ok_or_else(|| AuthyError::PolicyNotFound(scope.to_string()))?;

    let allowed = policy.can_read(secret_name)?;
    if allowed {
        println!("ALLOWED: '{}' can read '{}'", scope, secret_name);
    } else {
        println!("DENIED: '{}' cannot read '{}'", scope, secret_name);
    }

    Ok(())
}
