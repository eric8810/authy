# v0.7 — Task Tracker

Status: `[ ]` todo · `[~]` in progress · `[x]` done

---

## Phase 1 — Import from external stores

### CLI changes

- [ ] Add `ImportSource` enum (`Dotenv`, `OnePassword`, `Pass`, `Sops`, `Vault`) to `src/cli/mod.rs`
- [ ] Add `--from`, `--vault`, `--tag`, `--path`, `--mount` args to `Import` variant
- [ ] Refactor `src/cli/import.rs` to dispatch to source adapters

### Adapter module

- [ ] Create `src/cli/import_sources/mod.rs` — `ImportAdapter` trait, module declarations
- [ ] Create `src/cli/import_sources/onepassword.rs` — 1Password `op` CLI adapter
- [ ] Create `src/cli/import_sources/pass.rs` — `pass` / password-store adapter
- [ ] Create `src/cli/import_sources/sops.rs` — SOPS decryption adapter
- [ ] Create `src/cli/import_sources/hcvault.rs` — HashiCorp Vault KV adapter

### Tests

- [ ] Integration test: `authy import .env` still works (regression)
- [ ] Integration test: `authy import --from 1password` with mock `op` script
- [ ] Integration test: `authy import --from pass` with mock gpg/store
- [ ] Integration test: `authy import --from sops` with mock sops script
- [ ] Integration test: `authy import --from vault` with mock vault script
- [ ] Integration test: missing CLI tool produces actionable error
- [ ] Integration test: `--dry-run` with external source

## Phase 2 — Python SDK

### Package setup

- [ ] Create `packages/python/pyproject.toml`
- [ ] Create `packages/python/src/authy_secrets/__init__.py`
- [ ] Create `packages/python/src/authy_secrets/errors.py`
- [ ] Create `packages/python/src/authy_secrets/client.py`

### Implementation

- [ ] `Authy.__init__` — binary discovery, credential env setup
- [ ] `Authy._run_cmd` — subprocess + JSON parsing + error mapping
- [ ] `Authy.get`, `get_or_none`
- [ ] `Authy.store` (value via stdin)
- [ ] `Authy.remove`
- [ ] `Authy.rotate`
- [ ] `Authy.list` (with scope)
- [ ] `Authy.run`
- [ ] `Authy.init`, `is_initialized`
- [ ] Error hierarchy: `AuthyError`, `SecretNotFound`, `AuthFailed`, etc.

### Tests

- [ ] Unit tests with mocked subprocess (test_client.py)
- [ ] Integration tests with real binary (test_integration.py)

## Phase 3 — TypeScript SDK

### Package setup

- [ ] Create `packages/typescript/package.json`
- [ ] Create `packages/typescript/tsconfig.json`
- [ ] Create `packages/typescript/src/index.ts`
- [ ] Create `packages/typescript/src/errors.ts`
- [ ] Create `packages/typescript/src/types.ts`

### Implementation

- [ ] `Authy` async class (`src/client.ts`)
- [ ] `AuthySync` sync class (`src/sync.ts`)
- [ ] `runCmd` — execFile + JSON parsing + error mapping
- [ ] `get`, `getOrNull`
- [ ] `store` (value via stdin)
- [ ] `remove`
- [ ] `rotate`
- [ ] `list` (with scope)
- [ ] `run`
- [ ] `init`, `isInitialized`
- [ ] Error hierarchy: `AuthyError`, `SecretNotFound`, `AuthFailed`, etc.
- [ ] Dual CJS/ESM build output

### Tests

- [ ] Unit tests with mocked child_process
- [ ] Integration tests with real binary

## Phase 4 — Go SDK

### Module setup

- [ ] Create `packages/go/go.mod`
- [ ] Create `packages/go/authy.go` — Client struct, New(), options
- [ ] Create `packages/go/errors.go` — error types + sentinels

### Implementation

- [ ] `operations.go` — Get, Store, Remove, Rotate, List, Run
- [ ] `runCmd` — exec.CommandContext + JSON parsing + error mapping
- [ ] Context support on all methods
- [ ] Option pattern: WithBinary, WithPassphrase, WithKeyfile, Force, WithScope

### Tests

- [ ] Unit tests with mock binary
- [ ] Integration tests (`//go:build integration`)

## Phase 5 — CI

- [ ] GitHub Actions job: Python SDK tests (unit + integration)
- [ ] GitHub Actions job: TypeScript SDK tests (unit + integration)
- [ ] GitHub Actions job: Go SDK tests (unit + integration)
- [ ] All SDK integration tests use a shared authy binary built from the same commit

## Phase 6 — Docs & version bump

- [ ] Bump version in `Cargo.toml` to `0.7.0`
- [ ] Add v0.7.0 section to `CHANGELOG.md`
- [ ] Update `CLAUDE.md` project structure with `packages/` directory
- [ ] Update `milestones.md` — check off v0.7 items
- [ ] README mentions for each SDK with install + quickstart

---

## Final verification

- [ ] `cargo build` compiles clean
- [ ] `cargo clippy -- -D warnings` passes clean
- [ ] `cargo test` — all Rust tests pass
- [ ] Python: `pytest` passes
- [ ] TypeScript: `npm test` passes
- [ ] Go: `go test ./...` passes
- [ ] Existing `.env` import behavior unchanged (regression)
