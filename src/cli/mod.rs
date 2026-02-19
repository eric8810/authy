pub mod admin;
pub mod alias;
pub mod audit;
pub mod common;
pub mod config;
pub mod env;
pub mod export;
pub mod get;
pub mod hook;
pub mod import;
pub mod init;
pub mod json_output;
pub mod list;
pub mod policy;
pub mod project_info;
pub mod rekey;
pub mod remove;
pub mod resolve;
pub mod rotate;
pub mod run;
pub mod session;
pub mod store;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "authy", version, about = "CLI secrets store & dispatch for agents")]
pub struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vault
    Init {
        /// Generate a keyfile at this path instead of using a passphrase
        #[arg(long)]
        generate_keyfile: Option<String>,
        /// Set vault passphrase non-interactively
        #[arg(long, env = "AUTHY_PASSPHRASE")]
        passphrase: Option<String>,
    },

    /// Store a secret (reads value from stdin)
    Store {
        /// Secret name
        name: String,
        /// Overwrite if exists
        #[arg(long)]
        force: bool,
    },

    /// Get a secret value
    Get {
        /// Secret name
        name: String,
        /// Scope to enforce policy against
        #[arg(long)]
        scope: Option<String>,
    },

    /// List secret names
    List {
        /// Scope to filter by policy
        #[arg(long)]
        scope: Option<String>,
    },

    /// Remove a secret
    Remove {
        /// Secret name
        name: String,
    },

    /// Rotate a secret (reads new value from stdin)
    Rotate {
        /// Secret name
        name: String,
    },

    /// Manage access policies
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },

    /// Manage session tokens
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Run a command with secrets injected as env vars
    Run {
        /// Scope for secret access (optional if .authy.toml exists)
        #[arg(long)]
        scope: Option<String>,
        /// Uppercase env var names
        #[arg(long)]
        uppercase: bool,
        /// Replace dashes with this character (e.g. '_')
        #[arg(long)]
        replace_dash: Option<char>,
        /// Prefix for env var names
        #[arg(long)]
        prefix: Option<String>,
        /// Command and arguments to run
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },

    /// Output secrets as environment variables
    Env {
        /// Scope (policy name) for secret access (optional if .authy.toml exists)
        #[arg(long)]
        scope: Option<String>,
        /// Uppercase env var names
        #[arg(long)]
        uppercase: bool,
        /// Replace dashes with this character (e.g. '_')
        #[arg(long)]
        replace_dash: Option<char>,
        /// Prefix for env var names
        #[arg(long)]
        prefix: Option<String>,
        /// Output format: shell, dotenv, json
        #[arg(long, default_value = "shell")]
        format: String,
        /// Omit 'export' keyword in shell format
        #[arg(long)]
        no_export: bool,
    },

    /// Import secrets from a .env file
    Import {
        /// Path to .env file (use '-' for stdin)
        file: String,
        /// Keep original names (don't transform to lower-kebab-case)
        #[arg(long)]
        keep_names: bool,
        /// Add prefix to secret names
        #[arg(long)]
        prefix: Option<String>,
        /// Overwrite existing secrets
        #[arg(long)]
        force: bool,
        /// Preview changes without storing
        #[arg(long)]
        dry_run: bool,
    },

    /// Export secrets as .env or JSON
    Export {
        /// Output format: env, json
        #[arg(long, default_value = "env")]
        format: String,
        /// Scope (policy name) to filter secrets
        #[arg(long)]
        scope: Option<String>,
        /// Uppercase env var names
        #[arg(long)]
        uppercase: bool,
        /// Replace dashes with this character (e.g. '_')
        #[arg(long)]
        replace_dash: Option<char>,
        /// Prefix for env var names
        #[arg(long)]
        prefix: Option<String>,
    },

    /// View and verify audit logs
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },

    /// Show or edit configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Show project config from .authy.toml
    ProjectInfo {
        /// Show a specific field (scope, keyfile, vault, uppercase, replace-dash, prefix, dir, aliases)
        #[arg(long)]
        field: Option<String>,
        /// Start directory for .authy.toml discovery
        #[arg(long)]
        dir: Option<String>,
    },

    /// Generate shell aliases for tools
    Alias {
        /// Scope (policy name) — optional if --from-project is used
        scope: Option<String>,
        /// Shell syntax to generate (bash, zsh, fish, powershell)
        #[arg(long, default_value = "bash")]
        shell: String,
        /// Read scope, naming, and aliases from .authy.toml
        #[arg(long)]
        from_project: bool,
        /// Output unalias commands for the project in AUTHY_PROJECT_DIR
        #[arg(long)]
        cleanup: bool,
        /// Tool names to alias
        #[arg(trailing_var_arg = true)]
        tools: Vec<String>,
    },

    /// Output shell hook code for auto-activation on cd
    Hook {
        /// Shell to generate hook for (bash, zsh, fish)
        shell: String,
    },

    /// Resolve <authy:key-name> placeholders in a file
    Resolve {
        /// Source file with <authy:key-name> placeholders
        file: String,
        /// Output path (default: stdout)
        #[arg(long, short)]
        output: Option<String>,
        /// Scope for secret access
        #[arg(long)]
        scope: Option<String>,
    },

    /// Re-encrypt the vault with new credentials
    Rekey {
        /// Generate a new keyfile at this path
        #[arg(long)]
        generate_keyfile: Option<String>,
        /// Switch to passphrase auth
        #[arg(long)]
        to_passphrase: bool,
        /// Re-encrypt with an existing keyfile
        #[arg(long)]
        new_keyfile: Option<String>,
    },

    /// Launch admin TUI (interactive vault management)
    Admin {
        /// Keyfile path (alternative to passphrase prompt in TUI)
        #[arg(long, env = "AUTHY_KEYFILE")]
        keyfile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PolicyCommands {
    /// Create a new policy
    Create {
        /// Policy / scope name
        name: String,
        /// Allow glob patterns
        #[arg(long, required = true, num_args = 1..)]
        allow: Vec<String>,
        /// Deny glob patterns
        #[arg(long, num_args = 1..)]
        deny: Vec<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Restrict to run-only mode (secrets can only be injected via `authy run`)
        #[arg(long)]
        run_only: bool,
    },
    /// Show a policy
    Show {
        name: String,
    },
    /// Update an existing policy
    Update {
        name: String,
        /// New allow glob patterns (replaces existing)
        #[arg(long, num_args = 1..)]
        allow: Option<Vec<String>>,
        /// New deny glob patterns (replaces existing)
        #[arg(long, num_args = 1..)]
        deny: Option<Vec<String>>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Enable run-only mode (secrets can only be injected via `authy run`)
        #[arg(long)]
        run_only: Option<bool>,
    },
    /// List all policies
    List,
    /// Remove a policy
    Remove {
        name: String,
    },
    /// Test a policy against a secret name
    Test {
        /// Policy name
        #[arg(long)]
        scope: String,
        /// Secret name to test
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// Create a new session token
    Create {
        /// Scope (policy name) for this session
        #[arg(long)]
        scope: String,
        /// Time to live (e.g. "1h", "30m", "7d")
        #[arg(long, default_value = "1h")]
        ttl: String,
        /// Optional label for this session
        #[arg(long)]
        label: Option<String>,
        /// Restrict to run-only mode (secrets can only be injected via `authy run`)
        #[arg(long)]
        run_only: bool,
    },
    /// List active sessions
    List,
    /// Revoke a session by ID
    Revoke {
        /// Session ID to revoke
        id: String,
    },
    /// Revoke all sessions
    RevokeAll,
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Show recent audit log entries
    Show {
        /// Number of entries to show (0 = all)
        #[arg(long, short, default_value = "20")]
        count: usize,
    },
    /// Verify audit log integrity
    Verify,
    /// Export audit log as JSON array
    Export,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
}
