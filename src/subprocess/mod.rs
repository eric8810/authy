use std::collections::HashMap;
use std::process::Command;

use crate::error::{AuthyError, Result};

/// Options for naming environment variables when injecting secrets.
#[derive(Debug, Clone, Default)]
pub struct NamingOptions {
    pub uppercase: bool,
    pub replace_dash: Option<char>,
    pub prefix: Option<String>,
}

/// Transform a secret name into an environment variable name.
pub fn transform_name(name: &str, opts: &NamingOptions) -> String {
    let mut result = name.to_string();

    if let Some(replacement) = opts.replace_dash {
        result = result.replace('-', &replacement.to_string());
    }

    if opts.uppercase {
        result = result.to_uppercase();
    }

    if let Some(ref prefix) = opts.prefix {
        result = format!("{}{}", prefix, result);
    }

    result
}

/// Run a subprocess with the given secrets injected as environment variables.
/// Returns the exit code of the subprocess.
pub fn run_with_secrets(
    command: &[String],
    secrets: &HashMap<String, String>,
    naming: &NamingOptions,
) -> Result<i32> {
    if command.is_empty() {
        return Err(AuthyError::Other("No command specified".into()));
    }

    let env_vars: HashMap<String, String> = secrets
        .iter()
        .map(|(name, value)| (transform_name(name, naming), value.clone()))
        .collect();

    let status = Command::new(&command[0])
        .args(&command[1..])
        .envs(&env_vars)
        .env_remove("AUTHY_PASSPHRASE")
        .env_remove("AUTHY_TOKEN")
        .status()
        .map_err(|e| AuthyError::Other(format!("Failed to run command '{}': {}", command[0], e)))?;

    Ok(status.code().unwrap_or(1))
}
