use std::process::Command;

use authy::error::{AuthyError, Result};

use super::ImportAdapter;

pub struct HcVaultAdapter {
    pub path: String,
    pub mount: String,
}

impl ImportAdapter for HcVaultAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>> {
        check_vault_installed()?;

        let data = read_kv_secret(&self.path, &self.mount)?;

        let mut secrets = Vec::new();
        if let serde_json::Value::Object(map) = &data {
            for (key, val) in map {
                match val {
                    serde_json::Value::String(s) => {
                        secrets.push((key.clone(), s.clone()));
                    }
                    other => {
                        // Convert non-string values to their JSON representation
                        let s = other.to_string();
                        if s != "null" {
                            secrets.push((key.clone(), s));
                        }
                    }
                }
            }
        }

        Ok(secrets)
    }
}

fn check_vault_installed() -> Result<()> {
    match Command::new("vault").arg("version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AuthyError::Other(
            "HashiCorp Vault CLI not found. Install from https://www.vaultproject.io/downloads"
                .into(),
        )),
    }
}

fn read_kv_secret(path: &str, mount: &str) -> Result<serde_json::Value> {
    let output = Command::new("vault")
        .args(["kv", "get", "-format=json", &format!("-mount={}", mount), path])
        .output()
        .map_err(|e| AuthyError::Other(format!("Failed to run `vault kv get`: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("permission denied")
            || stderr.contains("missing client token")
            || stderr.contains("VAULT_TOKEN")
        {
            return Err(AuthyError::Other(
                "Not authenticated. Run `vault login` or set VAULT_TOKEN.".into(),
            ));
        }
        if stderr.contains("no secrets") || stderr.contains("Not Found") {
            return Err(AuthyError::Other(format!(
                "No secrets found at path '{}' (mount: {})",
                path, mount
            )));
        }
        return Err(AuthyError::Other(format!(
            "vault kv get failed: {}",
            stderr.trim()
        )));
    }

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            AuthyError::Other(format!("Failed to parse vault output: {}", e))
        })?;

    // KV v2 response: data is at .data.data
    // KV v1 response: data is at .data
    if let Some(data) = response.get("data").and_then(|d| d.get("data")) {
        Ok(data.clone())
    } else if let Some(data) = response.get("data") {
        Ok(data.clone())
    } else {
        Err(AuthyError::Other(
            "Unexpected vault response format: no data field".into(),
        ))
    }
}
