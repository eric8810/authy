# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-02-17

### Added

- **Run-only enforcement** — Restrict agents to `authy run` only. Blocks `get`, `env`, and `export` at both token and policy level. Agents can inject secrets into subprocesses but never read values directly.
  - `authy session create --scope <s> --run-only` creates a run-only token
  - `authy policy create <name> --allow <glob> --run-only` creates a run-only policy
  - `authy policy update <name> --run-only true|false` toggles run-only on existing policies
  - Either token-level or policy-level run-only triggers the restriction
- **JSON output** — Global `--json` flag on all read commands. Structured JSON to stdout, errors as JSON to stderr.
- **`authy env` command** — Output secrets as environment variables in shell, dotenv, or JSON format. Supports `--format shell|dotenv|json`, `--no-export`, and naming transforms.
- **`authy import` command** — Import secrets from `.env` files. Supports `--dry-run`, `--force`, `--keep-names`, `--prefix`, and stdin (`-`). Transforms `UPPER_SNAKE_CASE` to `lower-kebab-case` by default.
- **`authy export` command** — Export secrets as `.env` or JSON format with optional scope filtering and naming transforms.
- **Non-interactive mode** — Fails fast when stdin is not a TTY and no credentials are provided. Supports `AUTHY_NON_INTERACTIVE=1` env var.
- **Typed exit codes** — Each error category maps to a specific exit code (1-7). `error_code()` method returns string identifiers for programmatic use.
- **JSON error output** — When `--json` is active, errors emit `{"error":{"code":"...","message":"...","exit_code":N}}` to stderr.
- **Agent Skills** — Unified `skills/authy/SKILL.md` following the Agent Skills standard. Works with Claude Code, Cursor, OpenClaw, and 38+ AI coding agents.

### Changed

- `authy run` refactored to use shared `resolve_scoped_secrets()` helper
- Session and policy JSON output now includes `run_only` field
- `Policy::new()` constructor initializes `run_only: false`
- `AuthContext` carries `run_only` flag propagated from token auth

### Security

- Run-only tokens and policies enforce that agents can only inject secrets into subprocesses via `authy run`. Direct value access (`get`, `env`, `export`) is blocked with exit code 4.
- Token-level run-only propagates through `SessionRecord.run_only` → `AuthContext.run_only`
- Policy-level run-only checked independently in each command handler

## [0.1.0] - 2026-02-10

### Added

- Initial release
- Encrypted vault with age encryption (X25519 keyfile or scrypt passphrase)
- Glob-based scoped access policies (deny overrides allow, default deny)
- HMAC-SHA256 session tokens with configurable TTL and instant revocation
- Subprocess secret injection via `authy run`
- Append-only JSONL audit log with HMAC chain verification
- Admin TUI for managing secrets, policies, sessions, and audit logs
- Pipe-friendly CLI (values to stdout, diagnostics to stderr)
- Atomic vault writes (write to .tmp, rename)
- Zeroize on drop for all secret-holding types
