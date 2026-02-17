use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::{AuthyError, Result};
use crate::types::*;

type HmacSha256 = Hmac<Sha256>;

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub secret: Option<String>,
    pub actor: String,
    pub outcome: String,
    pub detail: Option<String>,
    pub chain_hmac: String,
}

/// Append an audit entry to the log file.
pub fn log_event(
    audit_path: &Path,
    operation: &str,
    secret: Option<&str>,
    actor: &str,
    outcome: &str,
    detail: Option<&str>,
    hmac_key: &[u8],
) -> Result<()> {
    let prev_hmac = read_last_hmac(audit_path);

    let entry = AuditEntry {
        timestamp: Utc::now(),
        operation: operation.to_string(),
        secret: secret.map(|s| s.to_string()),
        actor: actor.to_string(),
        outcome: outcome.to_string(),
        detail: detail.map(|s| s.to_string()),
        chain_hmac: String::new(), // Will be filled below
    };

    // Compute HMAC chain: HMAC(prev_hmac || serialized_entry_without_chain)
    let chain_data = format!(
        "{}|{}|{}|{:?}|{}|{}|{:?}",
        prev_hmac,
        entry.timestamp.to_rfc3339(),
        entry.operation,
        entry.secret,
        entry.actor,
        entry.outcome,
        entry.detail,
    );

    let chain_hmac = compute_chain_hmac(&chain_data, hmac_key);

    let final_entry = AuditEntry {
        chain_hmac,
        ..entry
    };

    let json_line =
        serde_json::to_string(&final_entry).map_err(|e| AuthyError::Serialization(e.to_string()))?;

    if let Some(dir) = audit_path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_path)?;
    writeln!(file, "{}", json_line)?;

    Ok(())
}

/// Read all audit entries from the log file.
pub fn read_entries(audit_path: &Path) -> Result<Vec<AuditEntry>> {
    if !audit_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(audit_path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: AuditEntry =
            serde_json::from_str(&line).map_err(|e| AuthyError::Serialization(e.to_string()))?;
        entries.push(entry);
    }

    Ok(entries)
}

/// Verify the HMAC chain integrity of the audit log.
pub fn verify_chain(audit_path: &Path, hmac_key: &[u8]) -> Result<(usize, bool)> {
    let entries = read_entries(audit_path)?;
    let mut prev_hmac = String::new();

    for (i, entry) in entries.iter().enumerate() {
        let chain_data = format!(
            "{}|{}|{}|{:?}|{}|{}|{:?}",
            prev_hmac,
            entry.timestamp.to_rfc3339(),
            entry.operation,
            entry.secret,
            entry.actor,
            entry.outcome,
            entry.detail,
        );

        let expected_hmac = compute_chain_hmac(&chain_data, hmac_key);
        if expected_hmac != entry.chain_hmac {
            return Err(AuthyError::AuditChainBroken(i));
        }
        prev_hmac = entry.chain_hmac.clone();
    }

    Ok((entries.len(), true))
}

fn read_last_hmac(audit_path: &Path) -> String {
    if !audit_path.exists() {
        return String::new();
    }

    // Read the file and get the last non-empty line
    if let Ok(content) = fs::read_to_string(audit_path) {
        for line in content.lines().rev() {
            if !line.trim().is_empty() {
                if let Ok(entry) = serde_json::from_str::<AuditEntry>(line) {
                    return entry.chain_hmac;
                }
            }
        }
    }

    String::new()
}

fn compute_chain_hmac(data: &str, hmac_key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(hmac_key).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Derive the audit HMAC key from the master key material.
pub fn derive_audit_key(master_material: &[u8]) -> Vec<u8> {
    crate::vault::crypto::derive_key(master_material, b"audit-hmac", 32)
}

/// Get the master material from a VaultKey (for HKDF derivation).
pub fn key_material(key: &crate::vault::VaultKey) -> Vec<u8> {
    match key {
        crate::vault::VaultKey::Passphrase(p) => p.as_bytes().to_vec(),
        crate::vault::VaultKey::Keyfile { identity, .. } => identity.as_bytes().to_vec(),
    }
}
