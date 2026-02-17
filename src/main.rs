mod audit;
mod auth;
mod cli;
mod config;
mod error;
mod policy;
mod session;
mod subprocess;
mod tui;
mod types;
mod vault;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init {
            generate_keyfile,
            passphrase,
        } => cli::init::run(passphrase.clone(), generate_keyfile.clone()),

        Commands::Store { name, force } => cli::store::run(name, *force),

        Commands::Get { name, scope } => cli::get::run(name, scope.as_deref()),

        Commands::List { scope } => cli::list::run(scope.as_deref()),

        Commands::Remove { name } => cli::remove::run(name),

        Commands::Rotate { name } => cli::rotate::run(name),

        Commands::Policy { command } => cli::policy::run(command),

        Commands::Session { command } => cli::session::run(command),

        Commands::Run {
            scope,
            uppercase,
            replace_dash,
            prefix,
            command,
        } => cli::run::run(scope, *uppercase, *replace_dash, prefix.clone(), command),

        Commands::Audit { command } => cli::audit::run(command),

        Commands::Config { command } => cli::config::run(command),

        Commands::Admin { keyfile } => cli::admin::run(keyfile.clone()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
