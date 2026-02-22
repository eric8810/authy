use std::io;

use authy::api::AuthyClient;
use authy::error::{AuthyError, Result};
use authy::mcp::McpServer;

pub fn run(mcp: bool) -> Result<()> {
    if !mcp {
        eprintln!("authy serve requires --mcp");
        return Err(AuthyError::Other("authy serve requires --mcp".into()));
    }

    let client = AuthyClient::from_env().ok();
    let server = McpServer::new(client);

    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();
    server.run(stdin, stdout)?;

    Ok(())
}
