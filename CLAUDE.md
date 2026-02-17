# CLAUDE.md — Project Instructions

## What is this

Authy is a CLI secrets store & dispatch tool for AI agents. Built in Rust. Single binary, no server, no accounts.

## Build & Test

```bash
cargo build                    # dev build
cargo build --release          # release build
cargo test                     # all tests (unit + integration)
cargo clippy -- -D warnings    # lint (must pass clean)
```

## Project Structure

- `src/cli/` — clap command definitions and handlers (one file per command)
- `src/vault/` — encrypted vault storage (age encryption, MessagePack serialization)
- `src/auth/` — authentication dispatcher (passphrase / keyfile / session token)
- `src/policy/` — glob-based access control policies
- `src/session/` — HMAC session token generation and validation
- `src/audit/` — append-only JSONL audit log with HMAC chain
- `src/subprocess/` — child process spawning with env var injection
- `tests/integration/` — integration tests using assert_cmd + tempfile

## Key Conventions

- All secret-holding types must derive `Zeroize` and be zeroized on drop
- Secret values flow through stdin/stdout, never CLI arguments
- Diagnostics and errors go to stderr, secret values to stdout
- Session tokens are read-only — no mutation commands accept token auth
- Policy evaluation: deny overrides allow, default deny
- Vault writes use atomic rename (write to .tmp, then rename)

## Crypto Stack

- `age` crate for vault encryption (passphrase via scrypt, keyfile via X25519)
- `hmac` + `sha2` for session token HMAC
- `hkdf` for deriving session and audit keys from master key
- `subtle` for constant-time token comparison
- `rand::OsRng` for cryptographic random generation
