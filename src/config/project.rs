use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::{AuthyError, Result};

/// Project-level configuration from `.authy.toml`.
///
/// Example:
/// ```toml
/// [authy]
/// scope = "my-project"
/// keyfile = "~/.authy/keys/my-project.key"
/// uppercase = true
/// replace_dash = "_"
/// aliases = ["claude", "aider"]
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfigFile {
    pub authy: ProjectConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    /// Scope (policy name) for secret access (required)
    pub scope: String,
    /// Path to keyfile (supports ~ expansion)
    pub keyfile: Option<String>,
    /// Override vault path
    pub vault: Option<String>,
    /// Uppercase env var names (default false)
    #[serde(default)]
    pub uppercase: bool,
    /// Replace dashes with this character, validated to single char
    pub replace_dash: Option<String>,
    /// Prefix for env var names
    pub prefix: Option<String>,
    /// Tool names to alias (e.g. ["claude", "aider"])
    #[serde(default)]
    pub aliases: Vec<String>,
}

const CONFIG_FILENAME: &str = ".authy.toml";

impl ProjectConfig {
    /// Load project config from a specific file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AuthyError::Other(format!("Failed to read {}: {}", path.display(), e)))?;
        let file: ProjectConfigFile = toml::from_str(&content)
            .map_err(|e| AuthyError::Other(format!("Invalid .authy.toml: {}", e)))?;

        let config = file.authy;

        // Validate replace_dash is a single character
        if let Some(ref rd) = config.replace_dash {
            if rd.chars().count() != 1 {
                return Err(AuthyError::Other(format!(
                    "replace_dash must be a single character, got '{}'",
                    rd
                )));
            }
        }

        // Validate scope is not empty
        if config.scope.is_empty() {
            return Err(AuthyError::Other(
                "scope must not be empty in .authy.toml".to_string(),
            ));
        }

        Ok(config)
    }

    /// Walk up from `start_dir` looking for `.authy.toml`.
    /// Returns the config and the directory containing the file.
    pub fn discover(start_dir: &Path) -> Result<Option<(Self, PathBuf)>> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(CONFIG_FILENAME);
            if candidate.is_file() {
                let config = Self::load(&candidate)?;
                return Ok(Some((config, dir)));
            }
            if !dir.pop() {
                break;
            }
        }
        Ok(None)
    }

    /// Convenience: discover from current working directory.
    pub fn discover_from_cwd() -> Result<Option<(Self, PathBuf)>> {
        let cwd = std::env::current_dir()
            .map_err(|e| AuthyError::Other(format!("Cannot determine cwd: {}", e)))?;
        Self::discover(&cwd)
    }

    /// Get replace_dash as a char.
    pub fn replace_dash_char(&self) -> Option<char> {
        self.replace_dash.as_ref().and_then(|s| s.chars().next())
    }

    /// Expand ~ in keyfile path.
    pub fn expanded_keyfile(&self) -> Option<String> {
        self.keyfile.as_ref().map(|kf| expand_tilde(kf))
    }

    /// Expand ~ in vault path.
    pub fn expanded_vault(&self) -> Option<String> {
        self.vault.as_ref().map(|v| expand_tilde(v))
    }
}

/// Expand leading `~` to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_valid_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".authy.toml");
        fs::write(
            &config_path,
            r#"
[authy]
scope = "my-project"
keyfile = "~/.authy/keys/test.key"
uppercase = true
replace_dash = "_"
prefix = "APP_"
aliases = ["claude", "aider"]
"#,
        )
        .unwrap();

        let config = ProjectConfig::load(&config_path).unwrap();
        assert_eq!(config.scope, "my-project");
        assert_eq!(config.keyfile.as_deref(), Some("~/.authy/keys/test.key"));
        assert!(config.uppercase);
        assert_eq!(config.replace_dash.as_deref(), Some("_"));
        assert_eq!(config.prefix.as_deref(), Some("APP_"));
        assert_eq!(config.aliases, vec!["claude", "aider"]);
        assert_eq!(config.replace_dash_char(), Some('_'));
    }

    #[test]
    fn test_load_minimal_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".authy.toml");
        fs::write(
            &config_path,
            r#"
[authy]
scope = "test"
"#,
        )
        .unwrap();

        let config = ProjectConfig::load(&config_path).unwrap();
        assert_eq!(config.scope, "test");
        assert!(!config.uppercase);
        assert!(config.replace_dash.is_none());
        assert!(config.prefix.is_none());
        assert!(config.aliases.is_empty());
        assert!(config.keyfile.is_none());
        assert!(config.vault.is_none());
    }

    #[test]
    fn test_invalid_replace_dash() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".authy.toml");
        fs::write(
            &config_path,
            r#"
[authy]
scope = "test"
replace_dash = "abc"
"#,
        )
        .unwrap();

        let err = ProjectConfig::load(&config_path).unwrap_err();
        assert!(err.to_string().contains("single character"));
    }

    #[test]
    fn test_empty_scope_rejected() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(".authy.toml");
        fs::write(
            &config_path,
            r#"
[authy]
scope = ""
"#,
        )
        .unwrap();

        let err = ProjectConfig::load(&config_path).unwrap_err();
        assert!(err.to_string().contains("scope must not be empty"));
    }

    #[test]
    fn test_discover_walks_up() {
        let root = TempDir::new().unwrap();
        let nested = root.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            root.path().join(".authy.toml"),
            "[authy]\nscope = \"root-project\"\n",
        )
        .unwrap();

        let result = ProjectConfig::discover(&nested).unwrap();
        assert!(result.is_some());
        let (config, dir) = result.unwrap();
        assert_eq!(config.scope, "root-project");
        assert_eq!(dir, root.path());
    }

    #[test]
    fn test_discover_finds_closest() {
        let root = TempDir::new().unwrap();
        let sub = root.path().join("sub");
        fs::create_dir_all(&sub).unwrap();

        fs::write(
            root.path().join(".authy.toml"),
            "[authy]\nscope = \"root\"\n",
        )
        .unwrap();
        fs::write(sub.join(".authy.toml"), "[authy]\nscope = \"sub\"\n").unwrap();

        let result = ProjectConfig::discover(&sub).unwrap();
        let (config, dir) = result.unwrap();
        assert_eq!(config.scope, "sub");
        assert_eq!(dir, sub);
    }

    #[test]
    fn test_discover_none_when_not_found() {
        let dir = TempDir::new().unwrap();
        let result = ProjectConfig::discover(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/foo/bar");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.ends_with("/foo/bar"));

        // Absolute path unchanged
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    }
}
