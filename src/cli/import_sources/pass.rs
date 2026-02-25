use std::path::{Path, PathBuf};
use std::process::Command;

use authy::error::{AuthyError, Result};

use super::ImportAdapter;

pub struct PassAdapter {
    pub store_path: Option<String>,
}

impl ImportAdapter for PassAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>> {
        let store_dir = resolve_store_dir(&self.store_path)?;

        if !store_dir.is_dir() {
            return Err(AuthyError::Other(format!(
                "Password store directory not found: {}",
                store_dir.display()
            )));
        }

        // Check that gpg is available
        check_gpg_installed()?;

        // Walk the directory for .gpg files
        let gpg_files = find_gpg_files(&store_dir)?;

        if gpg_files.is_empty() {
            return Ok(Vec::new());
        }

        let mut secrets = Vec::new();
        for gpg_path in &gpg_files {
            let rel_path = gpg_path
                .strip_prefix(&store_dir)
                .unwrap_or(gpg_path)
                .to_string_lossy();

            // Strip the .gpg extension to get the secret name
            let name = rel_path.trim_end_matches(".gpg").to_string();
            // Replace path separators with dashes for the name
            let name = name.replace(['/', '\\'], "-");

            match decrypt_gpg_file(gpg_path) {
                Ok(value) => {
                    // pass convention: only the first line is the password
                    let first_line = value.lines().next().unwrap_or("").to_string();
                    if !first_line.is_empty() {
                        secrets.push((name, first_line));
                    }
                }
                Err(e) => {
                    eprintln!("Warning: skipping '{}': {}", name, e);
                }
            }
        }

        Ok(secrets)
    }
}

fn resolve_store_dir(explicit_path: &Option<String>) -> Result<PathBuf> {
    if let Some(p) = explicit_path {
        return Ok(PathBuf::from(p));
    }

    // Check $PASSWORD_STORE_DIR
    if let Ok(dir) = std::env::var("PASSWORD_STORE_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // Default: ~/.password-store
    let home = dirs::home_dir()
        .ok_or_else(|| AuthyError::Other("Cannot determine home directory".into()))?;
    Ok(home.join(".password-store"))
}

fn check_gpg_installed() -> Result<()> {
    match Command::new("gpg").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(AuthyError::Other(
            "GPG not found. Install gnupg.".into(),
        )),
    }
}

fn find_gpg_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_dir(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // Skip hidden directories (like .git, .gpg-id)
        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            walk_dir(&path, files)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("gpg") {
            files.push(path);
        }
    }
    Ok(())
}

fn decrypt_gpg_file(path: &Path) -> Result<String> {
    let output = Command::new("gpg")
        .args(["--quiet", "--yes", "--batch", "--decrypt"])
        .arg(path)
        .output()
        .map_err(|e| AuthyError::Other(format!("Failed to run gpg: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AuthyError::Other(format!(
            "GPG decryption failed: {}",
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|_| AuthyError::Other("Decrypted value is not valid UTF-8 (binary value, skipping)".into()))
}
