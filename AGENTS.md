# AGENTS.md — AI Agent Project Guide

## Project Overview

Authy is a CLI secrets store & dispatch tool for AI agents. Built in Rust. Single binary, no server, no accounts. It stores encrypted secrets locally and dispatches them to agents with policy-based scoping, short-lived session tokens, and audit logging.

**Tech Stack:** Rust 2021 edition, clap (CLI), age (encryption), MessagePack (serialization), HMAC-SHA256 (session tokens).

## Build & Test Commands

```bash
cargo build                    # dev build
cargo build --release          # release build (binary at target/release/authy)
cargo test                     # run all tests (unit + integration)
cargo test --test integration  # run integration tests only
cargo test <test_name>         # run a single test
cargo clippy -- -D warnings    # lint (must pass clean)
```

## Project Structure

```
src/
  main.rs               Entry point — parse CLI args, dispatch to handlers
  error.rs              AuthyError enum (thiserror), Result type alias
  types.rs              Common re-exports (serde, chrono, BTreeMap, PathBuf)

  cli/                  clap command definitions (one file per command)
    mod.rs              CLI struct with Subcommand enum
    init.rs             authy init — create vault, generate keyfile or passphrase
    store.rs            authy store — decrypt vault, insert secret, re-encrypt
    get.rs              authy get — decrypt vault, policy check, output to stdout
    list.rs             authy list — list secret names with optional scope filter
    remove.rs           authy remove — delete secret from vault
    rotate.rs           authy rotate — update secret value, bump version
    policy.rs           authy policy * — CRUD for scope policies
    session.rs          authy session * — create/list/revoke tokens
    run.rs              authy run — subprocess injection with scoped secrets
    audit.rs            authy audit * — show/verify/export audit log
    config.rs           authy config — show configuration

  vault/                Encrypted vault storage
    mod.rs              Vault struct, VaultKey enum, load_vault(), save_vault()
    crypto.rs           age encrypt/decrypt (passphrase + keyfile), HKDF derivation
    secret.rs           SecretEntry, SecretMetadata (with Zeroize)

  auth/                 Authentication dispatcher
    mod.rs              resolve_auth() — resolve credentials to VaultKey + AuthContext
    context.rs          AuthContext — carries resolved identity and permission level

  policy/               Access control
    mod.rs              Policy struct, can_read() with globset matching

  session/              Session token management
    mod.rs              SessionRecord, generate_token(), validate_token()

  audit/                Audit logging
    mod.rs              AuditEntry, append_entry(), verify_chain()

  subprocess/           Child process spawning
    mod.rs              Spawn child process with env var injection

  config/               Configuration parsing
    mod.rs              authy.toml parsing

tests/integration/      Integration tests using assert_cmd + tempfile
```

## Code Style Guidelines

### Imports
- Group imports: std first, external crates second, crate modules third
- Use `crate::types::*` for common re-exports (Serialize, Deserialize, DateTime, BTreeMap, PathBuf)
- Example from `src/vault/mod.rs`:
  ```rust
  use std::fs;
  use crate::error::{AuthyError, Result};
  use crate::policy::Policy;
  use crate::types::*;
  ```

### Error Handling
- Use `AuthyError` enum from `src/error.rs` for all errors
- Return `Result<T>` type alias (maps to `std::result::Result<T, AuthyError>`)
- Use `?` operator with `map_err()` for external errors:
  ```rust
  .map_err(|e| AuthyError::Serialization(e.to_string()))?
  ```
- User-facing error messages should be descriptive and actionable

### Types & Serialization
- Use `BTreeMap<String, T>` for ordered key-value storage (not HashMap)
- All persistent types must derive `Serialize, Deserialize` from serde
- DateTime fields use `DateTime<Utc>` from chrono
- Use `#[serde(default)]` for optional/empty Vec fields

### Naming Conventions
- Functions: `snake_case` (e.g., `load_vault`, `derive_key`)
- Types: `PascalCase` (e.g., `VaultKey`, `SecretEntry`)
- Module names: `snake_case` (one word preferred)
- CLI commands: kebab-case in help text, PascalCase in enum variants

### Documentation
- Use `///` doc comments on public items
- Brief one-line descriptions preferred

## Key Conventions

1. **Secret values flow through stdin/stdout**, never CLI arguments. This prevents secrets from appearing in shell history or process listings.
2. **Diagnostics go to stderr**, secret values to stdout. This enables safe piping.
3. **All secret-holding types must derive `Zeroize` and `ZeroizeOnDrop`**. Memory is wiped when the value is dropped.
4. **Session tokens are read-only** — no mutation commands accept token auth.
5. **Policy evaluation: deny overrides allow, default deny**.
6. **Vault writes use atomic rename** (write to .tmp, then rename) to prevent corruption.
7. **Policies are stored inside the encrypted vault** — cannot be tampered with without the master key.

## Testing

### Integration Tests
- Located in `tests/integration/`
- Use `assert_cmd` for CLI testing, `tempfile` for isolated home directories
- Each test creates a `TempDir` and sets `HOME` env var to isolate tests
- Helper pattern:
  ```rust
  fn authy_cmd(home: &TempDir) -> Command {
      let mut cmd = Command::cargo_bin("authy").unwrap();
      cmd.env("HOME", home.path());
      cmd.env_remove("AUTHY_PASSPHRASE");
      cmd
  }
  ```

### Test Naming
- `test_<command>_<scenario>` pattern (e.g., `test_store_and_get`, `test_init_twice_fails`)

## Security Considerations

### Crypto Stack
- `age` crate for vault encryption (passphrase via scrypt, keyfile via X25519)
- `hmac` + `sha2` for session token HMAC-SHA256
- `hkdf` for deriving session and audit keys from master key
- `subtle` for constant-time token comparison
- `rand::OsRng` for cryptographic random generation

### Secret Handling
- Secrets never appear in CLI arguments, shell history, or process argv
- Session tokens have a scannable `authy_v1.` prefix for leak detection
- Audit log uses HMAC chain — any modification breaks the chain
- All secret-holding types are zeroized on drop

### File Layout
```
~/.authy/
  vault.age           Encrypted vault (secrets + policies + sessions)
  audit.log           Append-only audit log (JSONL)
  authy.toml          Configuration (optional)
  keys/
    master.key        age identity (private key)
```

## Data Flow Examples

### Store a secret (admin, master key)
```
authy store db-url
  → auth: resolve passphrase/keyfile → VaultKey
  → vault: load_vault(key) → decrypt vault.age → Vault in memory
  → read secret value from stdin
  → insert into vault.secrets["db-url"]
  → vault: save_vault(vault, key) → serialize → encrypt → atomic write
  → audit: append SecretWrite entry
```

### Get a secret (agent, session token)
```
authy get db-url (with AUTHY_TOKEN + AUTHY_KEYFILE)
  → auth: resolve token + keyfile → VaultKey + session scope
  → vault: load_vault(key) → decrypt vault.age
  → session: validate_token() → find matching session → get scope
  → policy: scope.can_read("db-url") → allow/deny
  → if allowed: write secret value to stdout
  → audit: append SecretRead entry (GRANTED or DENIED)
```
