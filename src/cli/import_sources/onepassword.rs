use std::process::Command;

use authy::error::{AuthyError, Result};

use super::ImportAdapter;

pub struct OnePasswordAdapter {
    pub vault: Option<String>,
    pub tag: Option<String>,
}

impl ImportAdapter for OnePasswordAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>> {
        // Check that `op` CLI is installed
        check_op_installed()?;

        // List items
        let items = list_items(&self.vault, &self.tag)?;

        let mut secrets = Vec::new();
        for item in &items {
            let id = item["id"].as_str().unwrap_or_default();
            let title = item["title"].as_str().unwrap_or_default();

            if id.is_empty() || title.is_empty() {
                continue;
            }

            match get_item_password(id) {
                Ok(value) => {
                    if !value.is_empty() {
                        secrets.push((title.to_string(), value));
                    }
                }
                Err(e) => {
                    eprintln!("Warning: skipping '{}': {}", title, e);
                }
            }
        }

        Ok(secrets)
    }
}

fn check_op_installed() -> Result<()> {
    match Command::new("op").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AuthyError::Other(
            "1Password CLI (`op`) not found. Install from https://1password.com/downloads/command-line/"
                .into(),
        )),
    }
}

fn list_items(
    vault: &Option<String>,
    tag: &Option<String>,
) -> Result<Vec<serde_json::Value>> {
    let mut cmd = Command::new("op");
    cmd.args(["item", "list", "--format", "json"]);

    if let Some(v) = vault {
        cmd.args(["--vault", v]);
    }

    if let Some(t) = tag {
        cmd.args(["--tags", t]);
    }

    let output = cmd.output().map_err(|e| {
        AuthyError::Other(format!("Failed to run `op item list`: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not currently signed in")
            || stderr.contains("sign in")
            || stderr.contains("session expired")
        {
            return Err(AuthyError::Other(
                "Not signed in to 1Password. Run `op signin` first.".into(),
            ));
        }
        return Err(AuthyError::Other(format!(
            "op item list failed: {}",
            stderr.trim()
        )));
    }

    let items: Vec<serde_json::Value> =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            AuthyError::Other(format!("Failed to parse op output: {}", e))
        })?;

    Ok(items)
}

fn get_item_password(id: &str) -> Result<String> {
    let output = Command::new("op")
        .args(["item", "get", id, "--fields", "label=password", "--format", "json"])
        .output()
        .map_err(|e| AuthyError::Other(format!("Failed to run `op item get`: {}", e)))?;

    if !output.status.success() {
        // Try credential field as fallback
        let output2 = Command::new("op")
            .args(["item", "get", id, "--fields", "label=credential", "--format", "json"])
            .output()
            .map_err(|e| AuthyError::Other(format!("Failed to run `op item get`: {}", e)))?;

        if !output2.status.success() {
            return Err(AuthyError::Other(
                "No password or credential field found".into(),
            ));
        }

        let field: serde_json::Value =
            serde_json::from_slice(&output2.stdout).map_err(|e| {
                AuthyError::Other(format!("Failed to parse op field output: {}", e))
            })?;

        return Ok(field["value"].as_str().unwrap_or_default().to_string());
    }

    let field: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| {
            AuthyError::Other(format!("Failed to parse op field output: {}", e))
        })?;

    Ok(field["value"].as_str().unwrap_or_default().to_string())
}
