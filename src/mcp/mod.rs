//! MCP (Model Context Protocol) server — stdio JSON-RPC 2.0.
//!
//! Implements the minimal MCP handshake (`initialize`, `notifications/initialized`,
//! `tools/list`, `tools/call`, `ping`) over line-delimited JSON on stdin/stdout.

pub mod tools;

use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::AuthyClient;

// ── JSON-RPC 2.0 types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

// ── MCP Server ───────────────────────────────────────────────────

/// MCP server that dispatches JSON-RPC requests to the AuthyClient API.
pub struct McpServer {
    client: Option<AuthyClient>,
}

impl McpServer {
    pub fn new(client: Option<AuthyClient>) -> Self {
        Self { client }
    }

    /// Run the server read loop on the given reader/writer pair.
    ///
    /// Reads line-delimited JSON-RPC from `reader`, dispatches, and writes
    /// responses to `writer`. Returns when the reader reaches EOF.
    pub fn run<R: BufRead, W: Write>(&self, reader: R, mut writer: W) -> std::io::Result<()> {
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(line) {
                Ok(r) => r,
                Err(e) => {
                    let resp =
                        JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e));
                    Self::write_response(&mut writer, &resp)?;
                    continue;
                }
            };

            // Notifications (no id) produce no response
            let is_notification = request.id.is_none();
            let response = self.dispatch(&request);

            if !is_notification {
                if let Some(resp) = response {
                    Self::write_response(&mut writer, &resp)?;
                }
            }
        }
        Ok(())
    }

    fn write_response<W: Write>(writer: &mut W, resp: &JsonRpcResponse) -> std::io::Result<()> {
        let json = serde_json::to_string(resp)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(writer, "{}", json)?;
        writer.flush()
    }

    fn dispatch(&self, req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        match req.method.as_str() {
            "initialize" => Some(self.handle_initialize(req)),
            "notifications/initialized" => None,
            "ping" => Some(self.handle_ping(req)),
            "tools/list" => Some(self.handle_tools_list(req)),
            "tools/call" => Some(self.handle_tools_call(req)),
            _ => Some(JsonRpcResponse::error(
                req.id.clone(),
                -32601,
                format!("Method not found: {}", req.method),
            )),
        }
    }

    fn handle_initialize(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "authy",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        JsonRpcResponse::success(req.id.clone(), result)
    }

    fn handle_ping(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse::success(req.id.clone(), serde_json::json!({}))
    }

    fn handle_tools_list(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let tool_defs = tools::tool_definitions();
        let result = serde_json::json!({ "tools": tool_defs });
        JsonRpcResponse::success(req.id.clone(), result)
    }

    fn handle_tools_call(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let tool_name = req
            .params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let arguments = req
            .params
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));

        let client = match &self.client {
            Some(c) => c,
            None => {
                let result = tools::error_result(
                    "No credentials configured. Set AUTHY_KEYFILE or AUTHY_PASSPHRASE.",
                );
                return JsonRpcResponse::success(req.id.clone(), result);
            }
        };

        let result = tools::dispatch(client, tool_name, &arguments);
        JsonRpcResponse::success(req.id.clone(), result)
    }
}
