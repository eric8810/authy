# 01 — Rust Core: `build_env_map()`

## Summary

Add a `build_env_map()` method to `AuthyClient` in `src/api.rs` that loads the vault, filters secrets through a policy scope, transforms names into environment-variable-style keys, and returns a `HashMap<String, String>`. This provides the cross-FFI entry point that native language bindings use to implement `run()`-equivalent functionality.

## Motivation

The existing `authy run` command builds an env var map internally (via `src/cli/common.rs:resolve_scoped_secrets()`), but this logic lives in the CLI layer behind the `cli` feature gate. Native bindings (PyO3, napi-rs) need to build the same map from the library API without reimplementing policy evaluation.

## Current Behavior

- `resolve_scoped_secrets()` in `src/cli/common.rs` builds `HashMap<String, String>` from vault + policy
- Only accessible from CLI commands (`run`, `env`, `export`)
- No name transformation (uppercase, dash replacement) at the API level

## Proposed Behavior

```rust
impl AuthyClient {
    pub fn build_env_map(
        &self,
        scope: &str,
        uppercase: bool,
        replace_dash: Option<char>,
    ) -> Result<HashMap<String, String>>
}
```

- Loads vault, looks up policy by `scope` name
- Filters all secret names through the policy's `filter_secrets()`
- Applies name transformations: dash replacement, then uppercase
- Returns the final `HashMap<env_var_name, secret_value>`
- Audits the operation with scope and count

## API Surface

```rust
// Build env vars for the "backend" scope:
// db-url → DB_URL, api-key → API_KEY
let env = client.build_env_map("backend", true, Some('_'))?;

// No transformation (raw secret names as keys):
let env = client.build_env_map("all", false, None)?;
```

## Implementation

**File:** `src/api.rs`

The method reuses the same vault/policy loading pattern as `list()` and `test_policy()`, plus adds name transformation logic that mirrors what `authy run` does internally.

## Tests

**File:** `tests/api_test.rs`

- `test_api_build_env_map` — create secrets + policy, verify uppercase + dash replacement, verify policy filtering
- `test_api_build_env_map_no_transform` — verify raw names preserved when no transformations requested
- `test_api_build_env_map_policy_not_found` — verify error when scope doesn't exist

## Acceptance Criteria

- [ ] `build_env_map()` returns correct env var map filtered by policy
- [ ] Name transformations (uppercase, dash replacement) work correctly
- [ ] Returns `PolicyNotFound` error for missing scope
- [ ] Audit entry logged with scope and count
- [ ] All existing tests still pass
