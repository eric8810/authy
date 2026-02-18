use serde::Serialize;
use std::path::PathBuf;

use crate::config::project::ProjectConfig;
use crate::error::{AuthyError, Result};

#[derive(Serialize)]
struct ProjectInfoJson {
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyfile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vault: Option<String>,
    uppercase: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace_dash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prefix: Option<String>,
    aliases: Vec<String>,
    dir: String,
}

pub fn run(field: Option<&str>, dir: Option<&str>, json: bool) -> Result<()> {
    let start_dir = match dir {
        Some(d) => PathBuf::from(d),
        None => std::env::current_dir()
            .map_err(|e| AuthyError::Other(format!("Cannot determine cwd: {}", e)))?,
    };

    let (config, project_dir) = ProjectConfig::discover(&start_dir)?
        .ok_or_else(|| AuthyError::Other("No .authy.toml found".to_string()))?;

    if json && field.is_none() {
        let info = ProjectInfoJson {
            scope: config.scope.clone(),
            keyfile: config.expanded_keyfile(),
            vault: config.expanded_vault(),
            uppercase: config.uppercase,
            replace_dash: config.replace_dash.clone(),
            prefix: config.prefix.clone(),
            aliases: config.aliases.clone(),
            dir: project_dir.to_string_lossy().to_string(),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&info)
                .map_err(|e| AuthyError::Serialization(e.to_string()))?
        );
        return Ok(());
    }

    match field {
        Some("scope") => println!("{}", config.scope),
        Some("keyfile") => {
            if let Some(kf) = config.expanded_keyfile() {
                println!("{}", kf);
            }
        }
        Some("vault") => {
            if let Some(v) = config.expanded_vault() {
                println!("{}", v);
            }
        }
        Some("uppercase") => println!("{}", config.uppercase),
        Some("replace-dash") => {
            if let Some(ref rd) = config.replace_dash {
                println!("{}", rd);
            }
        }
        Some("prefix") => {
            if let Some(ref p) = config.prefix {
                println!("{}", p);
            }
        }
        Some("dir") => println!("{}", project_dir.display()),
        Some("aliases") => {
            for alias in &config.aliases {
                println!("{}", alias);
            }
        }
        Some(other) => {
            return Err(AuthyError::Other(format!(
                "Unknown field '{}'. Valid fields: scope, keyfile, vault, uppercase, replace-dash, prefix, dir, aliases",
                other
            )));
        }
        None => {
            // Default: show all fields
            println!("scope: {}", config.scope);
            if let Some(kf) = config.expanded_keyfile() {
                println!("keyfile: {}", kf);
            }
            if let Some(v) = config.expanded_vault() {
                println!("vault: {}", v);
            }
            println!("uppercase: {}", config.uppercase);
            if let Some(ref rd) = config.replace_dash {
                println!("replace-dash: {}", rd);
            }
            if let Some(ref p) = config.prefix {
                println!("prefix: {}", p);
            }
            if !config.aliases.is_empty() {
                println!("aliases: {}", config.aliases.join(", "));
            }
            println!("dir: {}", project_dir.display());
        }
    }

    Ok(())
}
