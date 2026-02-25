use std::process::Command;

use authy::error::{AuthyError, Result};

use super::ImportAdapter;

pub struct SopsAdapter {
    pub file: String,
}

impl ImportAdapter for SopsAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>> {
        check_sops_installed()?;

        let plaintext = decrypt_file(&self.file)?;

        // Detect format from file extension
        let is_json = self.file.ends_with(".json");
        let is_yaml = self.file.ends_with(".yaml")
            || self.file.ends_with(".yml");

        let value: serde_json::Value = if is_json {
            serde_json::from_str(&plaintext).map_err(|e| {
                AuthyError::Other(format!("Failed to parse decrypted JSON: {}", e))
            })?
        } else if is_yaml {
            let yaml_val: serde_yaml::Value =
                serde_yaml::from_str(&plaintext).map_err(|e| {
                    AuthyError::Other(format!("Failed to parse decrypted YAML: {}", e))
                })?;
            // Convert YAML value to JSON value for uniform flattening
            yaml_to_json(yaml_val)
        } else {
            return Err(AuthyError::Other(format!(
                "Unsupported SOPS file extension for '{}'. Expected .yaml, .yml, or .json",
                self.file
            )));
        };

        let mut secrets = Vec::new();
        flatten_json(&value, String::new(), &mut secrets);

        Ok(secrets)
    }
}

fn check_sops_installed() -> Result<()> {
    match Command::new("sops").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AuthyError::Other(
            "SOPS CLI not found. Install from https://github.com/getsops/sops"
                .into(),
        )),
    }
}

fn decrypt_file(file: &str) -> Result<String> {
    let output = Command::new("sops")
        .args(["--decrypt", file])
        .output()
        .map_err(|e| AuthyError::Other(format!("Failed to run sops: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AuthyError::Other(format!(
            "SOPS decryption failed: {}",
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| AuthyError::Other(format!("SOPS output is not valid UTF-8: {}", e)))
}

/// Flatten a JSON value into dot-separated key-value pairs.
/// Nested objects produce keys like "parent.child".
fn flatten_json(
    value: &serde_json::Value,
    prefix: String,
    out: &mut Vec<(String, String)>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(val, new_prefix, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = format!("{}.{}", prefix, i);
                flatten_json(val, new_prefix, out);
            }
        }
        serde_json::Value::String(s) => {
            if !prefix.is_empty() {
                out.push((prefix, s.clone()));
            }
        }
        serde_json::Value::Number(n) => {
            if !prefix.is_empty() {
                out.push((prefix, n.to_string()));
            }
        }
        serde_json::Value::Bool(b) => {
            if !prefix.is_empty() {
                out.push((prefix, b.to_string()));
            }
        }
        serde_json::Value::Null => {}
    }
}

/// Convert a serde_yaml::Value to a serde_json::Value for uniform processing.
fn yaml_to_json(val: serde_yaml::Value) -> serde_json::Value {
    match val {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                serde_json::Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter_map(|(k, v)| {
                    let key = match k {
                        serde_yaml::Value::String(s) => s,
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        _ => return None,
                    };
                    Some((key, yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}
