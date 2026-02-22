# 01 — AuthyClient API Extensions

## Summary

Add `test_policy` and `create_policy` methods to `AuthyClient` in `src/api.rs`. These are needed by the MCP `test_policy` tool and for test setup, and are natural additions to the programmatic API surface.

## Motivation

The MCP server needs to expose a `test_policy` tool. Currently the policy test logic lives only in the CLI handler (`src/cli/policy.rs`) and the TUI popup. Lifting it into `AuthyClient` follows the existing pattern: every vault operation has a library-level method.

`create_policy` is needed so MCP tests (and future MCP tools) can create policies programmatically without going through the CLI.

## API

### `test_policy`

```rust
/// Test whether a policy allows access to a secret.
/// Returns `true` if allowed, `false` if denied.
pub fn test_policy(&self, scope: &str, secret_name: &str) -> Result<bool>
```

**Behavior:**
1. Load vault
2. Find policy by `scope` name — return `PolicyNotFound` if missing
3. Call `policy.can_read(secret_name)`
4. Audit: operation=`policy.test`, secret=`secret_name`, outcome=`allowed`|`denied`, detail=`scope={scope}`
5. Return the boolean result

### `create_policy`

```rust
/// Create a new policy in the vault.
pub fn create_policy(
    &self,
    name: &str,
    allow: Vec<String>,
    deny: Vec<String>,
    description: Option<&str>,
    run_only: bool,
) -> Result<()>
```

**Behavior:**
1. Load vault
2. Check policy doesn't already exist — return `PolicyAlreadyExists` if it does
3. Create `Policy::new(name, allow, deny)`, set `description` and `run_only`
4. Insert into vault, touch, save
5. Audit: operation=`policy.create`, outcome=`success`, detail=`policy={name}`

## File Changes

| File | Change |
|------|--------|
| `src/api.rs` | Add `test_policy()` and `create_policy()` methods (~30 lines) |
| `tests/api_test.rs` | Add tests for both methods (~30 lines) |

## Tests

### `test_api_test_policy_allowed`
- Create vault, store secret, create policy allowing `*`
- `test_policy("scope", "secret")` returns `true`

### `test_api_test_policy_denied`
- Create vault, store secret, create policy allowing `other-*`
- `test_policy("scope", "secret")` returns `false`

### `test_api_test_policy_not_found`
- `test_policy("nonexistent", "secret")` returns `PolicyNotFound` error

### `test_api_create_policy`
- Create vault, call `create_policy` with allow/deny patterns
- `test_policy` on matching secret returns `true`

### `test_api_create_policy_duplicate_fails`
- Create policy, then try to create again with same name
- Returns `PolicyAlreadyExists` error
