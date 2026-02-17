use crate::auth;
use crate::error::{AuthyError, Result};
use crate::tui;

pub fn run(keyfile: Option<String>) -> Result<()> {
    if auth::is_non_interactive() {
        return Err(AuthyError::Other(
            "authy admin requires an interactive terminal.".into(),
        ));
    }
    tui::run(keyfile)
}
