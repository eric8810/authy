//! MCP tool definitions and dispatch for the authy vault.

use serde_json::Value;

use crate::api::AuthyClient;

/// Return JSON Schema definitions for all MCP tools.
pub fn tool_definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "get_secret",
            "description": "Retrieve a secret value from the vault",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Secret name" }
                },
                "required": ["name"]
            }
        }),
        serde_json::json!({
            "name": "list_secrets",
            "description": "List all secret names, optionally filtered by a policy scope",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "scope": { "type": "string", "description": "Policy scope to filter by (optional)" }
                }
            }
        }),
        serde_json::json!({
            "name": "store_secret",
            "description": "Store a secret in the vault",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Secret name" },
                    "value": { "type": "string", "description": "Secret value" },
                    "force": { "type": "boolean", "description": "Overwrite if exists (default: false)" }
                },
                "required": ["name", "value"]
            }
        }),
        serde_json::json!({
            "name": "remove_secret",
            "description": "Remove a secret from the vault",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Secret name" }
                },
                "required": ["name"]
            }
        }),
        serde_json::json!({
            "name": "test_policy",
            "description": "Test whether a policy allows access to a secret name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "scope": { "type": "string", "description": "Policy/scope name" },
                    "secret_name": { "type": "string", "description": "Secret name to test" }
                },
                "required": ["scope", "secret_name"]
            }
        }),
    ]
}

/// Dispatch a tool call to the appropriate handler.
pub fn dispatch(client: &AuthyClient, tool_name: &str, args: &Value) -> Value {
    match tool_name {
        "get_secret" => handle_get_secret(client, args),
        "list_secrets" => handle_list_secrets(client, args),
        "store_secret" => handle_store_secret(client, args),
        "remove_secret" => handle_remove_secret(client, args),
        "test_policy" => handle_test_policy(client, args),
        _ => error_result(&format!("Unknown tool: {}", tool_name)),
    }
}

/// Build an MCP error result with `isError: true`.
pub fn error_result(msg: &str) -> Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": msg }],
        "isError": true
    })
}

/// Build an MCP success result.
fn text_result(text: &str) -> Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": text }]
    })
}

fn handle_get_secret(client: &AuthyClient, args: &Value) -> Value {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_result("Missing required parameter: name"),
    };

    match client.get_or_err(name) {
        Ok(value) => text_result(&value),
        Err(e) => error_result(&e.to_string()),
    }
}

fn handle_list_secrets(client: &AuthyClient, args: &Value) -> Value {
    let scope = args.get("scope").and_then(|v| v.as_str());

    match client.list(scope) {
        Ok(names) => {
            let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
            text_result(&json)
        }
        Err(e) => error_result(&e.to_string()),
    }
}

fn handle_store_secret(client: &AuthyClient, args: &Value) -> Value {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_result("Missing required parameter: name"),
    };
    let value = match args.get("value").and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return error_result("Missing required parameter: value"),
    };
    let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

    match client.store(name, value, force) {
        Ok(()) => text_result(&format!("Stored secret '{}'", name)),
        Err(e) => error_result(&e.to_string()),
    }
}

fn handle_remove_secret(client: &AuthyClient, args: &Value) -> Value {
    let name = match args.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_result("Missing required parameter: name"),
    };

    match client.remove(name) {
        Ok(true) => text_result(&format!("Removed secret '{}'", name)),
        Ok(false) => text_result(&format!("Secret '{}' not found", name)),
        Err(e) => error_result(&e.to_string()),
    }
}

fn handle_test_policy(client: &AuthyClient, args: &Value) -> Value {
    let scope = match args.get("scope").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return error_result("Missing required parameter: scope"),
    };
    let secret_name = match args.get("secret_name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return error_result("Missing required parameter: secret_name"),
    };

    match client.test_policy(scope, secret_name) {
        Ok(true) => text_result(&format!(
            "allowed: scope '{}' can read '{}'",
            scope, secret_name
        )),
        Ok(false) => text_result(&format!(
            "denied: scope '{}' cannot read '{}'",
            scope, secret_name
        )),
        Err(e) => error_result(&e.to_string()),
    }
}
