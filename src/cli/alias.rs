use authy::config::project::ProjectConfig;
use authy::error::{AuthyError, Result};

pub fn run(
    scope: Option<&str>,
    shell: &str,
    from_project: bool,
    cleanup: bool,
    tools: &[String],
) -> Result<()> {
    // Validate shell
    let shell = match shell {
        "bash" | "zsh" | "fish" | "powershell" => shell,
        other => {
            return Err(AuthyError::Other(format!(
                "Unsupported shell '{}'. Use bash, zsh, fish, or powershell.",
                other
            )));
        }
    };

    if cleanup {
        return run_cleanup(shell);
    }

    if from_project {
        return run_from_project(shell);
    }

    // Explicit scope mode
    let scope = scope
        .ok_or_else(|| AuthyError::Other("No scope provided. Use --scope or --from-project.".to_string()))?;

    if tools.is_empty() {
        return Err(AuthyError::Other(
            "No tools specified. Provide tool names to alias.".to_string(),
        ));
    }

    // Default naming for explicit scope: --uppercase --replace-dash _
    let run_flags = build_run_flags(scope, true, Some('_'), None);

    for tool in tools {
        print_alias(shell, tool, &run_flags, tool);
    }

    Ok(())
}

fn run_from_project(shell: &str) -> Result<()> {
    let (config, _dir) = ProjectConfig::discover_from_cwd()?
        .ok_or_else(|| AuthyError::Other("No .authy.toml found".to_string()))?;

    if config.aliases.is_empty() {
        return Err(AuthyError::Other(
            "No aliases defined in .authy.toml".to_string(),
        ));
    }

    let run_flags = build_run_flags(
        &config.scope,
        config.uppercase,
        config.replace_dash_char(),
        config.prefix.as_deref(),
    );

    for tool in &config.aliases {
        print_alias(shell, tool, &run_flags, tool);
    }

    Ok(())
}

fn run_cleanup(shell: &str) -> Result<()> {
    // Read AUTHY_PROJECT_DIR to find the project config to clean up
    let project_dir = std::env::var("AUTHY_PROJECT_DIR")
        .map_err(|_| AuthyError::Other("AUTHY_PROJECT_DIR not set — nothing to clean up.".to_string()))?;

    let config_path = std::path::PathBuf::from(&project_dir).join(".authy.toml");
    if !config_path.is_file() {
        return Err(AuthyError::Other(format!(
            "No .authy.toml in {}",
            project_dir
        )));
    }

    let config = ProjectConfig::load(&config_path)?;

    for tool in &config.aliases {
        print_unalias(shell, tool);
    }

    Ok(())
}

fn build_run_flags(scope: &str, uppercase: bool, replace_dash: Option<char>, prefix: Option<&str>) -> String {
    let mut flags = format!("--scope {}", shell_quote(scope));
    if uppercase {
        flags.push_str(" --uppercase");
    }
    if let Some(c) = replace_dash {
        flags.push_str(&format!(" --replace-dash {}", c));
    }
    if let Some(p) = prefix {
        flags.push_str(&format!(" --prefix {}", shell_quote(p)));
    }
    flags
}

fn print_alias(shell: &str, name: &str, run_flags: &str, tool: &str) {
    match shell {
        "fish" => {
            println!(
                "alias {} 'authy run {} -- {}'",
                name, run_flags, tool
            );
        }
        "powershell" => {
            println!(
                "function {} {{ authy run {} -- {} @args }}",
                name, run_flags, tool
            );
        }
        // bash, zsh
        _ => {
            println!(
                "alias {}='authy run {} -- {}'",
                name, run_flags, tool
            );
        }
    }
}

fn print_unalias(shell: &str, name: &str) {
    match shell {
        "fish" => {
            println!("functions --erase {}", name);
        }
        "powershell" => {
            println!("Remove-Item -Path Function:\\{}", name);
        }
        // bash, zsh
        _ => {
            println!("unalias {} 2>/dev/null", name);
        }
    }
}

/// Simple shell quoting: wrap in single quotes if it contains spaces or special chars.
fn shell_quote(s: &str) -> String {
    if s.contains(|c: char| c.is_whitespace() || c == '\'' || c == '"' || c == '$' || c == '`') {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}
