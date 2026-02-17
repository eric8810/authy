use crate::audit as audit_mod;
use crate::auth;
use crate::cli::AuditCommands;
use crate::error::Result;
use crate::vault;

pub fn run(cmd: &AuditCommands) -> Result<()> {
    match cmd {
        AuditCommands::Show { count } => show(*count),
        AuditCommands::Verify => verify(),
        AuditCommands::Export => export(),
    }
}

fn show(count: usize) -> Result<()> {
    let entries = audit_mod::read_entries(&vault::audit_path())?;

    if entries.is_empty() {
        eprintln!("No audit log entries.");
        return Ok(());
    }

    let display = if count == 0 {
        &entries[..]
    } else {
        let start = entries.len().saturating_sub(count);
        &entries[start..]
    };

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
            // Shouldn't reach here, but handle it
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
