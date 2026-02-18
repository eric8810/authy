# 01 — JSON Output (`--json`)

## Summary

Add a `--json` flag to commands that produce output, so agents can parse responses reliably without scraping text.

## Motivation

Today, `authy get` outputs a raw string. `authy list` outputs one name per line. `authy session list` outputs a formatted table. None of these are machine-parseable without brittle string splitting. AI agents that call `authy` via bash need structured output to make decisions based on the result.

## Current Behavior

```
$ authy get db-url
postgresql://user:pass@host/db

$ authy list
api-key
db-url
github-token

$ authy session list
ID        SCOPE       STATUS   EXPIRES
abc123    deploy      active   2h remaining
def456    dev-agent   expired  —

$ authy policy show deploy
Name: deploy
Allow: db-*, github-token
Deny: prod-*
Description: Deploy agent policy
```

## Proposed Behavior

### Global `--json` flag

Add `--json` as a global flag on the root `Cli` struct (clap). When present, all output-producing commands emit JSON to stdout.

### Per-command output schemas

**`authy get <name> --json`**

```json
{
  "name": "db-url",
  "value": "postgresql://user:pass@host/db",
  "version": 3,
  "created": "2026-02-10T08:30:00Z",
  "modified": "2026-02-15T14:22:00Z"
}
```

**`authy list --json`**

```json
{
  "secrets": [
    {
      "name": "api-key",
      "version": 1,
      "created": "2026-02-10T08:30:00Z",
      "modified": "2026-02-10T08:30:00Z"
    },
    {
      "name": "db-url",
      "version": 3,
      "created": "2026-02-10T08:30:00Z",
      "modified": "2026-02-15T14:22:00Z"
    }
  ]
}
```

Note: `list --json` does NOT include secret values. Use `get` for values.

**`authy policy show <name> --json`**

```json
{
  "name": "deploy",
  "allow": ["db-*", "github-token"],
  "deny": ["prod-*"],
  "description": "Deploy agent policy"
}
```

**`authy policy list --json`**

```json
{
  "policies": [
    {
      "name": "deploy",
      "allow_count": 2,
      "deny_count": 1,
      "description": "Deploy agent policy"
    }
  ]
}
```

**`authy policy test --scope <scope> <name> --json`**

```json
{
  "secret": "db-url",
  "scope": "deploy",
  "allowed": true
}
```

**`authy session list --json`**

```json
{
  "sessions": [
    {
      "id": "abc123",
      "scope": "deploy",
      "label": "ci-deploy",
      "status": "active",
      "created": "2026-02-15T10:00:00Z",
      "expires": "2026-02-15T18:00:00Z"
    }
  ]
}
```

**`authy session create --json`**

```json
{
  "token": "authy_v1.dGhpcyBpcyBhIDMyIGJ5dGUg...",
  "session_id": "abc123",
  "scope": "deploy",
  "expires": "2026-02-15T18:00:00Z"
}
```

**`authy audit show --json`**

```json
{
  "entries": [
    {
      "timestamp": "2026-02-15T14:22:00Z",
      "operation": "secret_read",
      "secret": "db-url",
      "actor": "token:abc123",
      "outcome": "granted",
      "detail": ""
    }
  ]
}
```

### Commands NOT affected

- `authy get` (without `--json`): unchanged, raw value to stdout
- `authy run`: passthrough to child process, `--json` has no effect
- `authy admin`: TUI, `--json` has no effect
- `authy config show`: already TOML, `--json` could optionally emit JSON
- `authy audit export`: already JSON, `--json` is redundant (no-op)

### Error output with `--json`

When `--json` is set and an error occurs, stderr emits a JSON error object instead of a plain message. See [06-structured-errors.md](06-structured-errors.md).

## Interface

```
authy [--json] <command> [args...]
```

The `--json` flag is positional on the root parser, before the subcommand.

## Edge Cases

- `--json` + pipe to another command: works naturally (stdout is JSON)
- `--json` + `authy get`: includes metadata (name, version, timestamps), not just raw value
- `--json` + `authy run`: ignored silently (child process owns stdout)
- `--json` + `authy admin`: ignored silently (TUI owns terminal)
- Empty list results: return `{"secrets": []}`, not an error

## Acceptance Criteria

- [ ] `--json` flag available on root CLI
- [ ] `get`, `list`, `policy show`, `policy list`, `policy test`, `session list`, `session create`, `audit show` all produce valid JSON when `--json` is set
- [ ] JSON output goes to stdout; no diagnostic messages mixed in
- [ ] Without `--json`, all commands behave identically to today
- [ ] All JSON output is valid (parseable by `jq`)
