# 03 — CLI `serve` Command

## Summary

Add `authy serve --mcp` subcommand that launches the MCP server on stdin/stdout. This is the CLI entry point for the MCP protocol module.

## Motivation

MCP clients (Claude Desktop, Cursor, etc.) launch MCP servers as subprocesses and communicate over stdio. `authy serve --mcp` is the command they'll invoke.

## CLI Definition

```rust
/// Start a server (MCP, etc.)
Serve {
    /// Run as MCP server (JSON-RPC over stdio)
    #[arg(long)]
    mcp: bool,
},
```

## Behavior

1. If `--mcp` is not set, print error "authy serve requires --mcp" and exit with code 1
2. Try `AuthyClient::from_env()` to authenticate from env vars
   - If credentials are available, create `McpServer::new(Some(client))`
   - If no credentials, create `McpServer::new(None)` — tools will return credential errors
3. Call `server.run(stdin, stdout)` — blocks until EOF
4. Return success on clean EOF

**Important:** The server does NOT fail on missing credentials. It starts up and returns errors on individual tool calls. This allows the MCP client to connect, discover tools, and show the user what's available before auth is configured.

## File Changes

| File | Change |
|------|--------|
| `src/cli/mod.rs` | Add `pub mod serve;` and `Serve` variant to `Commands` enum |
| `src/cli/serve.rs` | **Create** — serve command handler (~30 lines) |
| `src/main.rs` | Add `Commands::Serve { mcp } => cli::serve::run(*mcp)` match arm |

## `src/cli/serve.rs`

```rust
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
```

## Integration Tests

### `test_serve_without_mcp_flag_fails`
- Run `authy serve` with no flags
- Assert exit code is non-zero
- Assert stderr contains "requires --mcp"

### `test_serve_appears_in_help`
- Run `authy --help`
- Assert output contains "serve"
