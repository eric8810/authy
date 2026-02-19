# 02 — Safe/Sensitive Command Split

## Summary

Formalize the classification of commands into "safe" (agent-usable) and "sensitive" (human-only). Safe commands work with run-only tokens. Sensitive commands require TTY or master key auth. This is mostly documenting and tightening what already exists.

## Motivation

Run-only mode already blocks `get`, `env`, `export` for agent tokens. But the enforcement is ad-hoc — each command handler checks individually. The classification isn't documented as a first-class concept. Agent-vault showed that an explicit safe/sensitive split, enforced by TTY requirement, is a clean pattern.

## Command Classification

### Safe Commands (agents can use)

| Command | Why safe |
|---------|----------|
| `authy list` | Names only, no values |
| `authy run` | Secrets injected into subprocess, not returned to caller |
| `authy resolve` | New in v0.4 — resolves file placeholders (values go to file, not stdout) |
| `authy project-info` | Shows config, no secrets |

### Sensitive Commands (require TTY or master key)

| Command | Why sensitive |
|---------|--------------|
| `authy get` | Returns secret value to stdout |
| `authy store` | Writes to vault |
| `authy remove` | Deletes from vault |
| `authy rotate` | Writes to vault |
| `authy env` | Outputs secret values |
| `authy import` | Writes to vault |
| `authy export` | Outputs secret values |
| `authy policy *` | Modifies access control |
| `authy session *` | Creates/revokes tokens |
| `authy admin` | Full vault access |
| `authy init` | Creates vault |
| `authy rekey` | Changes vault credentials |

## Current State

Run-only enforcement already exists in:
- `src/cli/get.rs` — checks `auth_ctx.run_only`
- `src/cli/env.rs` — checks `auth_ctx.run_only`
- `src/cli/export.rs` — checks `auth_ctx.run_only`

Token read-only enforcement exists in:
- `src/cli/store.rs`, `remove.rs`, `rotate.rs`, `policy.rs`, `session.rs` — check `auth_ctx.is_token()`

## Changes Needed

### 1. Document the classification

Add the safe/sensitive table to `docs/GUIDE.md` and SKILL.md references.

### 2. Add run-only check to new commands

- `authy resolve` — should work with run-only tokens (values go to file, not stdout)

### 3. Update skill to only reference safe commands

The skill already only teaches `run` and `list`. Add `resolve` as agents learn about v0.4 features.

### 4. Consider: should `authy resolve` require run-only?

`authy resolve` writes secret values to a file. It's similar to `authy run` (values go somewhere other than stdout) but the output file is readable. For now, allow it with run-only tokens — the developer controls where the file goes. We can tighten this later if needed.

## Tests

- Run-only token can use `list`, `run`, `resolve`
- Run-only token cannot use `get`, `env`, `export`, `store`
- Existing tests continue to pass
