use crate::error::Result;
use crate::tui;

pub fn run(keyfile: Option<String>) -> Result<()> {
    tui::run(keyfile)
}
