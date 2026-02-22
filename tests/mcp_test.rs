//! In-memory MCP server tests — no actual stdio, no subprocess.
//!
//! Each test creates an isolated vault, builds an `McpServer` with a real
//! `AuthyClient`, and feeds JSON-RPC requests through `run()` via byte buffers.

use serial_test::serial;
use tempfile::TempDir;

use authy::api::AuthyClient;
use authy::mcp::McpServer;

/// Set HOME to an isolated temp dir so vault operations don't collide.
fn with_isolated_home(f: impl FnOnce(&TempDir)) {
    let home = TempDir::new().unwrap();
    std::env::set_var("HOME", home.path());
    f(&home);
}

/// Send a single JSON-RPC request to the server and return the response line.
fn send_request(server: &McpServer, request: &str) -> String {
    let input = format!("{}\n", request);
    let reader = std::io::Cursor::new(input.into_bytes());
    let mut output = Vec::new();
    server.run(reader, &mut output).unwrap();
    String::from_utf8(output).unwrap().trim().to_string()
}

/// Parse a JSON-RPC response and return the parsed value.
fn parse_response(response: &str) -> serde_json::Value {
    serde_json::from_str(response).unwrap()
}

// ── initialize ──────────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_initialize() {
    let server = McpServer::new(None);
    let resp = send_request(
        &server,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    );

    let json = parse_response(&resp);
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);

    let result = &json["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"]["tools"].is_object());
    assert_eq!(result["serverInfo"]["name"], "authy");
    assert!(result["serverInfo"]["version"].is_string());
}

// ── tools/list ──────────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_tools_list() {
    let server = McpServer::new(None);
    let resp = send_request(
        &server,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
    );

    let json = parse_response(&resp);
    let tools = json["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 5);

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"get_secret"));
    assert!(names.contains(&"list_secrets"));
    assert!(names.contains(&"store_secret"));
    assert!(names.contains(&"remove_secret"));
    assert!(names.contains(&"test_policy"));
}

// ── get_secret ──────────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_get_secret() {
    with_isolated_home(|_home| {
        let client = AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();
        client.store("api-key", "sk-secret-123", false).unwrap();

        let server = McpServer::new(Some(
            AuthyClient::with_passphrase("test-pass").unwrap(),
        ));
        let resp = send_request(
            &server,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_secret","arguments":{"name":"api-key"}}}"#,
        );

        let json = parse_response(&resp);
        let content = &json["result"]["content"][0];
        assert_eq!(content["type"], "text");
        assert_eq!(content["text"], "sk-secret-123");
        assert!(json["result"]["isError"].is_null());
    });
}

// ── store + list ────────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_store_and_list() {
    with_isolated_home(|_home| {
        let client = AuthyClient::with_passphrase("test-pass").unwrap();
        client.init_vault().unwrap();

        let server = McpServer::new(Some(
            AuthyClient::with_passphrase("test-pass").unwrap(),
        ));

        // Store via MCP
        let resp = send_request(
            &server,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"store_secret","arguments":{"name":"mcp-key","value":"mcp-val"}}}"#,
        );
        let json = parse_response(&resp);
        assert!(json["result"]["isError"].is_null());

        // List via MCP
        let resp = send_request(
            &server,
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"list_secrets","arguments":{}}}"#,
        );
        let json = parse_response(&resp);
        let text = json["result"]["content"][0]["text"].as_str().unwrap();
        let names: Vec<String> = serde_json::from_str(text).unwrap();
        assert!(names.contains(&"mcp-key".to_string()));
    });
}

// ── no credentials ──────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_no_credentials() {
    let server = McpServer::new(None);
    let resp = send_request(
        &server,
        r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"get_secret","arguments":{"name":"foo"}}}"#,
    );

    let json = parse_response(&resp);
    assert_eq!(json["result"]["isError"], true);
    let text = json["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No credentials"));
}

// ── notification (no response) ──────────────────────────────────

#[test]
#[serial]
fn test_mcp_notification_no_response() {
    let server = McpServer::new(None);
    // Notification has no "id" field
    let resp = send_request(
        &server,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#,
    );

    // Should produce no output
    assert!(resp.is_empty());
}

// ── unknown method ──────────────────────────────────────────────

#[test]
#[serial]
fn test_mcp_unknown_method() {
    let server = McpServer::new(None);
    let resp = send_request(
        &server,
        r#"{"jsonrpc":"2.0","id":7,"method":"bogus/method","params":{}}"#,
    );

    let json = parse_response(&resp);
    assert_eq!(json["error"]["code"], -32601);
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Method not found"));
}
