# 02a — MCP Tool Definitions

## Summary

5 MCP tools exposed by the authy server. Each tool maps directly to an `AuthyClient` method.

## Tool Dispatch

```rust
fn dispatch(client: &AuthyClient, tool_name: &str, args: &Value) -> Value
```

Routes `tool_name` to a handler function, extracts params from `args`, calls the corresponding `AuthyClient` method, wraps the result in MCP tool result format.

## Tool Result Format (MCP spec)

Success:
```json
{
  "content": [{ "type": "text", "text": "..." }]
}
```

Error:
```json
{
  "content": [{ "type": "text", "text": "Error: ..." }],
  "isError": true
}
```

## Tool Definitions

### `get_secret`

Retrieve a secret value by name.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string", "description": "Secret name" }
  },
  "required": ["name"]
}
```

**Handler:** `client.get_or_err(name)` → returns value as text content, or `isError` if not found.

### `list_secrets`

List all secret names, optionally filtered by a policy scope.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "scope": { "type": "string", "description": "Policy scope to filter by (optional)" }
  }
}
```

**Handler:** `client.list(scope)` → returns JSON array of names as text content.

### `store_secret`

Store or update a secret.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string", "description": "Secret name" },
    "value": { "type": "string", "description": "Secret value" },
    "force": { "type": "boolean", "description": "Overwrite if exists (default: false)" }
  },
  "required": ["name", "value"]
}
```

**Handler:** `client.store(name, value, force)` → returns confirmation text, or `isError` if already exists and `force` is false.

### `remove_secret`

Remove a secret from the vault.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string", "description": "Secret name" }
  },
  "required": ["name"]
}
```

**Handler:** `client.remove(name)` → returns "Removed" or "Not found" text.

### `test_policy`

Test whether a policy allows access to a secret name.

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "scope": { "type": "string", "description": "Policy/scope name" },
    "secret_name": { "type": "string", "description": "Secret name to test" }
  },
  "required": ["scope", "secret_name"]
}
```

**Handler:** `client.test_policy(scope, secret_name)` → returns "allowed" or "denied" text, or `isError` if policy not found.

## `tool_definitions()` Function

Returns `Vec<Value>` — the JSON tool list for `tools/list` response. Each entry:

```json
{
  "name": "get_secret",
  "description": "Retrieve a secret value from the vault",
  "inputSchema": { ... }
}
```

## `error_result()` Helper

```rust
pub fn error_result(msg: &str) -> Value
```

Returns the MCP error result format with `isError: true`.

## Unknown Tool

If `tool_name` doesn't match any known tool, return `error_result("Unknown tool: {tool_name}")`.
