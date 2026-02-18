# v0.2 Development Tracker

Status key: `[ ]` todo · `[~]` in progress · `[x]` done · `[-]` skipped

---

## 01 — JSON Output (`--json`)

Spec: [01-json-output.md](01-json-output.md)

### Implementation

- [x] Add `--json` global flag to root `Cli` struct in `src/cli/mod.rs`
- [x] Thread `json` flag through command dispatch in `src/main.rs`
- [x] Define JSON response structs (serde `Serialize`) for each command output
- [x] `authy get --json` — output `{ name, value, version, created, modified }`
- [x] `authy list --json` — output `{ secrets: [{ name, version, created, modified }] }`
- [x] `authy policy show --json` — output `{ name, allow, deny, description }`
- [x] `authy policy list --json` — output `{ policies: [{ name, allow_count, deny_count, description }] }`
- [x] `authy policy test --json` — output `{ secret, scope, allowed }`
- [x] `authy session create --json` — output `{ token, session_id, scope, expires }`
- [x] `authy session list --json` — output `{ sessions: [{ id, scope, label, status, created, expires }] }`
- [x] `authy audit show --json` — output `{ entries: [...] }`
- [x] Suppress all `eprintln!` diagnostic messages when `--json` is active
- [x] Ignore `--json` silently on `run` and `admin` commands

### Tests

- [x] Integration test: `authy get --json` outputs valid JSON parseable by `serde_json`
- [x] Integration test: `authy list --json` with empty vault returns `{ "secrets": [] }`
- [x] Integration test: `authy list --json` pipe to `jq` succeeds
- [x] Integration test: `--json` on `run` does not affect child process stdout

---

## 02 — Env Command (`authy env`)

Spec: [02-env-command.md](02-env-command.md)

### Implementation

- [x] Add `Env` variant to `Commands` enum in `src/cli/mod.rs`
- [x] Define `EnvArgs` struct: `--scope`, `--uppercase`, `--replace-dash`, `--prefix`, `--format`, `--no-export`
- [x] Create `src/cli/env.rs` — handler
- [x] Extract shared secret-resolution + name-transform logic from `src/cli/run.rs` into a shared function
- [x] `--format shell` output: `export KEY='VALUE'` with proper shell escaping
- [x] `--format dotenv` output: `KEY=VALUE` with dotenv quoting rules
- [x] `--format json` output: flat `{ "KEY": "VALUE" }` object
- [x] `--no-export` omits `export` keyword in shell format
- [x] Handle values with single quotes (shell), newlines, special chars per format
- [x] Empty scope result: output nothing, exit 0
- [x] Audit log: record `env_export` operation with scope

### Tests

- [x] Integration test: `authy env --scope X --format shell` produces sourceable `export` lines
- [x] Integration test: `authy env --scope X --format dotenv` produces valid `.env` format
- [x] Integration test: `authy env --scope X --format json` produces valid JSON
- [x] Integration test: `--uppercase --replace-dash _ --prefix AUTHY_` transforms names correctly
- [x] Integration test: empty scope returns empty output, exit 0
- [x] Integration test: works with session token auth

---

## 03 — Import from `.env` (`authy import`)

Spec: [03-import-dotenv.md](03-import-dotenv.md)

### Implementation

- [x] Add `Import` variant to `Commands` enum in `src/cli/mod.rs`
- [x] Define `ImportArgs` struct: `file`, `--keep-names`, `--prefix`, `--force`, `--dry-run`
- [x] Create `src/cli/import.rs` — handler
- [x] Dotenv parser: handle `KEY=VALUE`, `KEY="VALUE"`, `KEY='VALUE'`
- [x] Dotenv parser: skip comments (`#`), empty lines, lines without `=`
- [x] Dotenv parser: strip `export` prefix
- [x] Dotenv parser: handle inline comments after unquoted values
- [x] Dotenv parser: handle escape sequences in double-quoted values (`\n`, `\\`, `\"`)
- [x] Name transformation: `UPPER_SNAKE_CASE` → `lower-kebab-case` by default
- [x] `--keep-names` preserves original names
- [x] `--prefix` adds prefix to imported names
- [x] `--force` overwrites existing secrets (bump version)
- [x] `--dry-run` shows preview of transformations without storing
- [x] Read from stdin with `-` as file argument
- [x] Skip existing secrets without `--force`, report skipped names to stderr
- [x] Audit log: record each imported secret

### Tests

- [x] Integration test: import a `.env` file, verify secrets stored in vault
- [x] Integration test: name transformation `UPPER_SNAKE` → `lower-kebab`
- [x] Integration test: `--keep-names` preserves original names
- [x] Integration test: `--force` overwrites existing secret
- [x] Integration test: without `--force`, existing secrets are skipped
- [x] Integration test: `--dry-run` does not modify vault
- [x] Integration test: comments and empty lines are ignored
- [x] Integration test: quoted values parsed correctly (single, double)
- [x] Integration test: read from stdin with `-`

---

## 04 — Export as `.env` (`authy export`)

Spec: [04-export-env.md](04-export-env.md)

### Implementation

- [x] Add `Export` variant to `Commands` enum in `src/cli/mod.rs`
- [x] Define `ExportArgs` struct: `--format`, `--scope`, `--uppercase`, `--replace-dash`, `--prefix`
- [x] Create `src/cli/export.rs` — handler
- [x] `--format env` output: `KEY=VALUE` with dotenv quoting for special chars
- [x] `--format json` output: full secret metadata array
- [x] `--scope` filters secrets by policy
- [x] Name transformation flags (shared logic with `env` and `run`)
- [x] Without `--scope`: require master key auth (reject session token)
- [x] With `--scope`: allow session token auth
- [x] Resolve namespace: `authy export` vs existing `authy audit export` — no conflict (different subcommand tree)

### Tests

- [x] Integration test: `authy export --format env` outputs valid `.env`
- [x] Integration test: `authy export --format json` outputs full metadata
- [x] Integration test: `--scope` filters secrets correctly
- [x] Integration test: round-trip `authy export --format env > .env` then `authy import .env` preserves values
- [x] Integration test: without `--scope`, session token is rejected
- [x] Integration test: special characters in values are escaped properly

---

## 05 — Non-Interactive Mode

Spec: [05-non-interactive.md](05-non-interactive.md)

### Implementation

- [x] Add TTY detection using `std::io::IsTerminal` in auth resolver (`src/auth/mod.rs`)
- [x] When stdin is not a TTY and no credentials provided: return error immediately (no prompt)
- [x] Support `AUTHY_NON_INTERACTIVE=1` env var to force non-interactive mode even on TTY
- [x] `authy admin` in non-interactive mode: error with "admin TUI requires an interactive terminal"
- [x] Error messages include specific remediation: "Set AUTHY_KEYFILE, AUTHY_PASSPHRASE, or AUTHY_TOKEN"
- [x] Use exit code 2 for missing credentials (distinct from exit code 1)
- [x] Ensure `authy store` and `authy rotate` still read secret values from piped stdin

### Tests

- [x] Integration test: `authy get` without credentials and stdin piped fails immediately (not hang)
- [x] Integration test: `AUTHY_NON_INTERACTIVE=1` prevents passphrase prompt
- [x] Integration test: error message mentions `AUTHY_KEYFILE`, `AUTHY_PASSPHRASE`, `AUTHY_TOKEN`
- [x] Integration test: exit code is 2 for missing credentials
- [x] Integration test: `echo "val" | authy store name` still works (piped stdin for value, keyfile for auth)

---

## 06 — Structured Errors

Spec: [06-structured-errors.md](06-structured-errors.md)

### Implementation

- [x] Map each `AuthyError` variant to a typed exit code (2-7, 10)
- [x] Create `exit_code()` method on `AuthyError`
- [x] Create `error_code()` method returning string identifier (e.g., `"not_found"`, `"access_denied"`)
- [x] Update `src/main.rs` to use typed exit codes
- [x] When `--json` is set: emit JSON error object to stderr on failure
- [x] JSON error format: `{ "error": { "code": "...", "message": "...", "exit_code": N } }`
- [x] `authy run`: forward child process exit code (child code, not Authy's code)
- [x] `authy run`: child killed by signal → exit 128 + signal number

### Tests

- [x] Integration test: `authy get nonexistent` exits with code 3
- [x] Integration test: `authy get` with wrong passphrase exits with code 2
- [x] Integration test: `authy get` with denied scope exits with code 4
- [x] Integration test: `authy store existing` exits with code 5
- [x] Integration test: `authy get --json nonexistent` emits JSON error to stderr
- [x] Integration test: `authy run -- false` forwards exit code 1 from child
- [x] Integration test: existing commands still exit 0 on success

---

## 07 — Agent Skills

Spec: [07-agent-discovery.md](07-agent-discovery.md)

### Implementation

- [x] Create unified `skills/authy/SKILL.md` — Agent Skills standard (agentskills.io) format
- [x] Create `skills/authy/references/commands.md` — detailed command reference for progressive disclosure
- [x] Skill follows Agent Skills spec: YAML frontmatter with name, description, license, compatibility, metadata
- [x] Skill is agent-facing: instructions for retrieving secrets, not developer docs
- [x] Skills document agent-facing commands: `get`, `list`, `run`, `env`
- [x] Skills include auth check workflow
- [x] Skills include "never read .env" rules and error handling table
- [x] Update README.md with skills section and install instructions (npx skills, clawhub, manual)

### Validation

- [-] Test: skill loads in Claude Code via `~/.claude/skills/authy/`
- [-] Test: skill loads in OpenClaw via `~/.openclaw/skills/authy/`
- [-] Test: `npx skills add eric8810/authy` installs correctly

---

## Cross-Cutting

### Shared infrastructure (supports multiple specs)

- [x] Extract secret-resolution + name-transform logic from `run.rs` into shared module (used by `env`, `export`, `run`)
- [x] Add `--json` global flag plumbing (used by json-output + structured-errors)
- [x] Add TTY detection utility function (used by non-interactive mode)

### Documentation

- [x] Update README.md: document new commands (`env`, `import`, `export`)
- [x] Update README.md: document `--json` flag
- [x] Update README.md: document non-interactive usage for agents
- [x] Update CLAUDE.md: add `env`, `import`, `export` to command list
- [x] Update AI_AGENT_GUIDE.md: replace manual env var examples with `authy env` usage

### Release

- [x] All `cargo test` pass
- [x] `cargo clippy -- -D warnings` clean
- [x] Bump version to 0.2.0 in `Cargo.toml`
- [ ] Tag `v0.2.0`
