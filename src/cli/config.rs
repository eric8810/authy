use crate::cli::ConfigCommands;
use crate::config::Config;
use crate::error::Result;
use crate::vault;

pub fn run(cmd: &ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => show(),
    }
}

fn show() -> Result<()> {
    let config = Config::load(&vault::config_path())?;
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| crate::error::AuthyError::Other(format!("Config serialize error: {}", e)))?;
    println!("{}", toml_str);
    Ok(())
}
