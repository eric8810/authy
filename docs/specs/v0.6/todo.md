# v0.6 — Task Tracker

Status: `[ ]` todo · `[~]` in progress · `[x]` done

---

## Phase 1 — AuthyClient API extensions

- [x] Add `test_policy(&self, scope, secret_name) -> Result<bool>` to `src/api.rs`
- [x] Add `create_policy(&self, name, allow, deny, description, run_only) -> Result<()>` to `src/api.rs`
- [x] Add `test_api_test_policy_allowed` test in `tests/api_test.rs`
- [x] Add `test_api_test_policy_denied` test in `tests/api_test.rs`
- [x] Add `test_api_test_policy_not_found` test in `tests/api_test.rs`
- [x] Add `test_api_create_policy` test in `tests/api_test.rs`
- [x] Add `test_api_create_policy_duplicate_fails` test in `tests/api_test.rs`

## Phase 2 — MCP protocol module

- [x] Create `src/mcp/mod.rs` with JSON-RPC types and `McpServer` struct
  - [x] `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError` serde types
  - [x] `McpServer::new(client)` constructor
  - [x] `McpServer::run(reader, writer)` — line-delimited read loop
  - [x] `handle_initialize` — returns protocol version, capabilities, server info
  - [x] `handle_ping` — returns `{}`
  - [x] `handle_tools_list` — returns tool definitions from `tools::tool_definitions()`
  - [x] `handle_tools_call` — extracts tool name and args, dispatches to tools module
  - [x] Notifications (no `id`) produce no response
  - [x] Parse errors return -32700
  - [x] Unknown methods return -32601
- [x] Create `src/mcp/tools.rs` with tool definitions and dispatch
  - [x] `tool_definitions() -> Vec<Value>` — JSON Schema for 5 tools
  - [x] `dispatch(client, tool_name, args) -> Value` — routes to handler functions
  - [x] `error_result(msg) -> Value` — MCP error format helper
  - [x] `get_secret` handler — calls `client.get_or_err()`
  - [x] `list_secrets` handler — calls `client.list()`
  - [x] `store_secret` handler — calls `client.store()`
  - [x] `remove_secret` handler — calls `client.remove()`
  - [x] `test_policy` handler — calls `client.test_policy()`
  - [x] Unknown tool returns error result
- [x] Add `pub mod mcp;` to `src/lib.rs`

## Phase 3 — CLI `serve` command

- [x] Add `pub mod serve;` to `src/cli/mod.rs`
- [x] Add `Serve { #[arg(long)] mcp: bool }` variant to `Commands` enum
- [x] Create `src/cli/serve.rs` — handler that creates McpServer, runs on stdin/stdout
- [x] Add `Commands::Serve { mcp } => cli::serve::run(*mcp)` to `src/main.rs`

## Phase 4 — TUI clipboard (OSC 52)

- [x] Add `copy_to_clipboard(data: &str) -> bool` helper to `src/tui/mod.rs`
- [x] Add `Ctrl+Y` handler in `RevealSecret` popup → copies value, shows status
- [x] Add `Ctrl+Y` handler in `ShowToken` popup → copies token, shows status
- [x] Update `RevealSecret` popup footer to include `[Ctrl+Y] copy`
- [x] Update `ShowToken` popup footer to include `[Ctrl+Y] copy`
- [x] Update help overlay to include `Ctrl+Y` binding

## Phase 5 — TUI vault change detection

- [x] Add `last_vault_mtime: Option<SystemTime>` field to `TuiApp`
- [x] Add `record_vault_mtime()` method
- [x] Add `vault_changed_externally()` method
- [x] Change `save_vault(&self)` to `save_vault(&mut self)`, call `record_vault_mtime()` after save
- [x] Call `record_vault_mtime()` after auth success in `src/tui/auth.rs`
- [x] Add mtime check in tick section of event loop (main screen, no popup)
- [x] Add `PopupKind::VaultChanged` variant
- [x] Add `VaultChanged` input handler (`y` reloads, `n` dismisses)
- [x] Add `VaultChanged` draw handler (ConfirmDialog)

## Phase 6 — Tests

### MCP tests (`tests/mcp_test.rs`)

- [x] Create `tests/mcp_test.rs`
- [x] Add `[[test]] name = "mcp"` entry to `Cargo.toml`
- [x] `test_mcp_initialize` — handshake returns server info and capabilities
- [x] `test_mcp_tools_list` — lists all 5 tools
- [x] `test_mcp_get_secret` — store then retrieve via MCP
- [x] `test_mcp_store_and_list` — store via MCP, list via MCP
- [x] `test_mcp_no_credentials` — returns `isError: true`
- [x] `test_mcp_notification_no_response` — notifications produce no output
- [x] `test_mcp_unknown_method` — returns -32601

### Integration tests (`tests/integration/serve_test.rs`)

- [x] Create `tests/integration/serve_test.rs`
- [x] Add `mod serve_test;` to `tests/integration/mod.rs`
- [x] `test_serve_without_mcp_flag_fails` — exits non-zero, stderr contains "requires --mcp"
- [x] `test_serve_appears_in_help` — `authy --help` output contains "serve"

## Phase 7 — Docs & version bump

- [x] Bump version in `Cargo.toml` to `0.6.0`
- [x] Add v0.6.0 section to `CHANGELOG.md`
- [x] Add `src/mcp/`, `src/cli/serve.rs` to `CLAUDE.md` project structure
- [x] Check off MCP items in `milestones.md`
- [x] Check off clipboard and vault change detection in `tui_todo.md`

---

## Final verification

- [x] `cargo build` compiles clean
- [x] `cargo clippy -- -D warnings` passes clean
- [x] `cargo test` — all 183 tests pass (8 unit + 24 API + 143 integration + 7 MCP + 1 doctest)
- [x] Manual: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | AUTHY_PASSPHRASE=test authy serve --mcp` returns server info JSON
- [ ] Manual: TUI `Ctrl+Y` on reveal/token popup copies to clipboard
