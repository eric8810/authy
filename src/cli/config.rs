use crate::cli::ConfigCommands;
use authy::config::Config;
use authy::error::Result;
use authy::vault;

pub fn run(cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => show(),
    }
}

fn show() -> Result<()> {
    let config = Config::load(&vault::config_path())?;
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| authy::error::AuthyError::Other(format!("Config serialize error: {}", e)))?;
    println!("{}", toml_str);
    Ok(())
}
