use crate::audit as audit_mod;
use crate::auth;
use crate::cli::json_output::{AuditEntryItem, AuditShowResponse};
use crate::cli::AuditCommands;
use crate::error::Result;
use crate::vault;

pub fn run(cmd: &AuditCommands, json: bool) -> Result<()> {
    match cmd {
        AuditCommands::Show { count } => show(*count, json),
        AuditCommands::Verify => verify(),
        AuditCommands::Export => export(),
    }
}

fn show(count: usize, json: bool) -> Result<()> {
    let entries = audit_mod::read_entries(&vault::audit_path())?;

    if entries.is_empty() {
        if json {
            let response = AuditShowResponse {
                entries: vec![],
                shown: 0,
                total: 0,
            };
            println!(
                "{}",
                serde_json::to_string(&response)
                    .map_err(|e| crate::error::AuthyError::Serialization(e.to_string()))?
            );
        } else {
            eprintln!("No audit log entries.");
        }
        return Ok(());
    }

    let display = if count == 0 {
        &entries[..]
    } else {
        let start = entries.len().saturating_sub(count);
        &entries[start..]
    };

    if json {
        let items: Vec<AuditEntryItem> = display
            .iter()
            .map(|e| AuditEntryItem {
                timestamp: e.timestamp.to_rfc3339(),
                operation: e.operation.clone(),
                secret: e.secret.clone(),
                actor: e.actor.clone(),
                outcome: e.outcome.clone(),
                detail: e.detail.clone(),
            })
            .collect();
        let response = AuditShowResponse {
            shown: items.len(),
            total: entries.len(),
            entries: items,
        };
        println!(
            "{}",
            serde_json::to_string(&response)
                .map_err(|e| crate::error::AuthyError::Serialization(e.to_string()))?
        );
    } else {
        for entry in display {
            let secret_str = entry.secret.as_deref().unwrap_or("-");
            let detail_str = entry.detail.as_deref().unwrap_or("");
            println!(
                "{} | {:<16} | {:<12} | {:<24} | {} {}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.operation,
                entry.outcome,
                entry.actor,
                secret_str,
                detail_str,
            );
        }

        eprintln!("\n({} entries shown of {} total)", display.len(), entries.len());
    }

    Ok(())
}

fn verify() -> Result<()> {
    let (key, _) = auth::resolve_auth(false)?;
    let material = audit_mod::key_material(&key);
    let audit_key = audit_mod::derive_audit_key(&material);

    match audit_mod::verify_chain(&vault::audit_path(), &audit_key) {
        Ok((count, true)) => {
            println!("Audit log integrity verified. {} entries, chain intact.", count);
            Ok(())
        }
        Ok(_) => {
            println!("Audit log verification returned unexpected result.");
            Ok(())
        }
        Err(e) => {
            eprintln!("INTEGRITY FAILURE: {}", e);
            Err(e)
        }
    }
}

fn export() -> Result<()> {
    let entries = audit_mod::read_entries(&vault::audit_path())?;
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| crate::error::AuthyError::Serialization(e.to_string()))?;
    println!("{}", json);
    Ok(())
}
