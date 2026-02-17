pub mod audit;
pub mod config;
pub mod get;
pub mod init;
pub mod list;
pub mod policy;
pub mod remove;
pub mod rotate;
pub mod run;
pub mod session;
pub mod store;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "authy", version, about = "CLI secrets store & dispatch for agents")]
pub struct Cli {
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
        /// Scope for secret access
        #[arg(long)]
        scope: String,
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
