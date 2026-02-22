# 02 — MCP Server Protocol

## Summary

Implement an MCP (Model Context Protocol) server that speaks stdio JSON-RPC 2.0. No external MCP crate — implement directly with `serde_json`. The server exposes 5 vault tools that delegate to `AuthyClient`.

## Motivation

AI agent platforms (Claude Desktop, Cursor, Continue, etc.) use MCP as their standard tool protocol. Exposing authy via MCP means platforms can call vault operations natively over JSON-RPC instead of shelling out to the CLI. This serves Segment 3 (operators) through the platforms they already use.

## Protocol

MCP uses JSON-RPC 2.0 over stdio (line-delimited JSON, one message per line).

### Handled Methods

| Method | Type | Description |
|--------|------|-------------|
| `initialize` | Request | MCP handshake — returns server info and capabilities |
| `notifications/initialized` | Notification | Client confirms init — no response |
| `tools/list` | Request | Returns tool definitions with JSON Schema |
| `tools/call` | Request | Invokes a tool by name with arguments |
| `ping` | Request | Health check — returns `{}` |

### Error Codes

| Code | Meaning |
|------|---------|
| `-32700` | Parse error (invalid JSON) |
| `-32601` | Method not found |

Tool-level errors are returned as successful JSON-RPC responses with `isError: true` in the tool result content, per MCP spec.

## JSON-RPC Types

```rust
#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,       // None = notification
    method: String,
    params: Value,            // defaults to {}
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,    // present on success
    error: Option<JsonRpcError>,  // present on error
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}
```

## `initialize` Response

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {}
  },
  "serverInfo": {
    "name": "authy",
    "version": "0.6.0"
  }
}
```

## McpServer Architecture

```rust
pub struct McpServer {
    client: Option<AuthyClient>,  // None if no creds at startup
}
```

- `McpServer::new(client)` — construct with optional client
- `McpServer::run(reader, writer)` — line-delimited read loop, dispatch, flush
- If `client` is `None`, all `tools/call` requests return `isError: true` with a credentials error message
- Notifications (requests with no `id`) produce no output

## File Changes

| File | Change |
|------|--------|
| `src/mcp/mod.rs` | **Create** — JSON-RPC handler, McpServer struct (~200 lines) |
| `src/mcp/tools.rs` | **Create** — Tool definitions and dispatch (~150 lines) |
| `src/lib.rs` | Add `pub mod mcp;` |

## MCP Tools

See [02a-mcp-tools.md](02a-mcp-tools.md) for tool definitions and dispatch logic.

## Tests

See `todo.md` Phase 6 for MCP test cases.
