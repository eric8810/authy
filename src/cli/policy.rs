use authy::audit;
use authy::auth;
use crate::cli::json_output::{
    PolicyListItem, PolicyListResponse, PolicyShowResponse, PolicyTestResponse,
};
use crate::cli::PolicyCommands;
use authy::error::{AuthyError, Result};
use authy::policy::Policy;
use authy::vault;

pub fn run(cmd: &PolicyCommands, json: bool) -> Result<()> {
    match cmd {
        PolicyCommands::Create {
            name,
            allow,
            deny,
            description,
            run_only,
        } => create(name, allow, deny, description.as_deref(), *run_only),
        PolicyCommands::Show { name } => show(name, json),
        PolicyCommands::Update {
            name,
            allow,
            deny,
            description,
            run_only,
        } => update(name, allow.as_deref(), deny.as_deref(), description.as_deref(), *run_only),
        PolicyCommands::List => list(json),
        PolicyCommands::Remove { name } => remove(name),
        PolicyCommands::Test { scope, name } => test(scope, name, json),
    }
}

fn create(name: &str, allow: &[String], deny: &[String], description: Option<&str>, run_only: bool) -> Result<()> {
    let (key, auth_ctx) = auth::resolve_auth(true)?;
    let mut vault = vault::load_vault(&key)?;

    if vault.policies.contains_key(name) {
        return Err(AuthyError::PolicyAlreadyExists(name.to_string()));
    }

    let mut policy = Policy::new(name.to_string(), allow.to_vec(), deny.to_vec());
    policy.description = description.map(|s| s.to_string());
    policy.run_only = run_only;

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

fn show(name: &str, json: bool) -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    let policy = vault
        .policies
        .get(name)
        .ok_or_else(|| AuthyError::PolicyNotFound(name.to_string()))?;

    if json {
        let response = PolicyShowResponse {
            name: policy.name.clone(),
            description: policy.description.clone(),
            allow: policy.allow.clone(),
            deny: policy.deny.clone(),
            run_only: policy.run_only,
            created: policy.created_at.to_rfc3339(),
            modified: policy.modified_at.to_rfc3339(),
        };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| AuthyError::Serialization(e.to_string()))?
        );
    } else {
        println!("Policy: {}", policy.name);
        if let Some(ref desc) = policy.description {
            println!("Description: {}", desc);
        }
        if policy.run_only {
            println!("Mode: run-only (secrets can only be injected via `authy run`)");
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
    }

    Ok(())
}

fn update(
    name: &str,
    allow: Option<&[String]>,
    deny: Option<&[String]>,
    description: Option<&str>,
    run_only: Option<bool>,
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
    if let Some(run_only) = run_only {
        policy.run_only = run_only;
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

fn list(json: bool) -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    if json {
        let policies: Vec<PolicyListItem> = vault
            .policies
            .values()
            .map(|p| PolicyListItem {
                name: p.name.clone(),
                description: p.description.clone(),
                allow_count: p.allow.len(),
                deny_count: p.deny.len(),
            })
            .collect();
        let response = PolicyListResponse { policies };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| AuthyError::Serialization(e.to_string()))?
        );
    } else {
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

fn test(scope: &str, secret_name: &str, json: bool) -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let vault = vault::load_vault(&key)?;

    let policy = vault
        .policies
        .get(scope)
        .ok_or_else(|| AuthyError::PolicyNotFound(scope.to_string()))?;

    let allowed = policy.can_read(secret_name)?;

    if json {
        let response = PolicyTestResponse {
            scope: scope.to_string(),
            secret: secret_name.to_string(),
            allowed,
        };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| AuthyError::Serialization(e.to_string()))?
        );
    } else if allowed {
        println!("ALLOWED: '{}' can read '{}'", scope, secret_name);
    } else {
        println!("DENIED: '{}' cannot read '{}'", scope, secret_name);
    }

    Ok(())
}
