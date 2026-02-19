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
    let json = cli.json;

    let result = match &cli.command {
        Commands::Init {
            generate_keyfile,
            passphrase,
        } => cli::init::run(passphrase.clone(), generate_keyfile.clone()),

        Commands::Store { name, force } => cli::store::run(name, *force),

        Commands::Get { name, scope } => cli::get::run(name, scope.as_deref(), json),

        Commands::List { scope } => cli::list::run(scope.as_deref(), json),

        Commands::Remove { name } => cli::remove::run(name),

        Commands::Rotate { name } => cli::rotate::run(name),

        Commands::Policy { command } => cli::policy::run(command, json),

        Commands::Session { command } => cli::session::run(command, json),

        Commands::Run {
            scope,
            uppercase,
            replace_dash,
            prefix,
            command,
        } => cli::run::run(scope.as_deref(), *uppercase, *replace_dash, prefix.clone(), command),

        Commands::Env {
            scope,
            uppercase,
            replace_dash,
            prefix,
            format,
            no_export,
        } => cli::env::run(scope.as_deref(), *uppercase, *replace_dash, prefix.clone(), format, *no_export),

        Commands::Import {
            file,
            keep_names,
            prefix,
            force,
            dry_run,
        } => cli::import::run(file, *keep_names, prefix.as_deref(), *force, *dry_run),

        Commands::Export {
            format,
            scope,
            uppercase,
            replace_dash,
            prefix,
        } => cli::export::run(format, scope.as_deref(), *uppercase, *replace_dash, prefix.clone()),

        Commands::Audit { command } => cli::audit::run(command, json),

        Commands::Config { command } => cli::config::run(command),

        Commands::ProjectInfo { field, dir } => {
            cli::project_info::run(field.as_deref(), dir.as_deref(), json)
        }

        Commands::Alias {
            scope,
            shell,
            from_project,
            cleanup,
            tools,
        } => cli::alias::run(scope.as_deref(), shell, *from_project, *cleanup, tools),

        Commands::Hook { shell } => cli::hook::run(shell),

        Commands::Resolve {
            file,
            output,
            scope,
        } => cli::resolve::run(file, output.as_deref(), scope.as_deref()),

        Commands::Rekey {
            generate_keyfile,
            to_passphrase,
            new_keyfile,
        } => cli::rekey::run(
            generate_keyfile.as_deref(),
            *to_passphrase,
            new_keyfile.as_deref(),
        ),

        Commands::Admin { keyfile } => cli::admin::run(keyfile.clone()),
    };

    if let Err(e) = result {
        if json {
            let json_err = error::JsonError::from_error(&e);
            eprintln!(
                "{}",
                serde_json::to_string(&json_err).unwrap_or_else(|_| format!(
                    "{{\"error\":{{\"code\":\"{}\",\"message\":\"{}\",\"exit_code\":{}}}}}",
                    e.error_code(),
                    e,
                    e.exit_code()
                ))
            );
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(e.exit_code());
    }
}
