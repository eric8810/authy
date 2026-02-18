# 04 — Export as `.env` (`authy export`)

## Summary

Extend the existing `authy export` concept to support exporting vault secrets as `.env` files. Enables round-tripping between Authy and tools that expect `.env` format.

## Motivation

Backward compatibility with the `.env` ecosystem. Some tools, CI systems, and Docker workflows require a `.env` file. A developer who has migrated to Authy should be able to generate a `.env` file for these integrations without manually extracting secrets.

This pairs with `authy import` — together they enable round-tripping: `.env` → Authy → `.env`.

## Current Behavior

`authy export` does not exist as a top-level command. `authy audit export` exists but exports audit log entries as JSON.

## Proposed Behavior

### New subcommand

```bash
$ authy export --format env --scope deploy > .env
```

Outputs vault secrets as a `.env` file filtered by scope.

### Flags

```
authy export --format <FORMAT> [--scope <SCOPE>] [--uppercase] [--replace-dash <CHAR>] [--prefix <PREFIX>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format <FORMAT>` | required | Output format: `env`, `json` |
| `--scope <SCOPE>` | none | Filter by policy scope (optional — without it, exports all) |
| `--uppercase` | off | Convert names to UPPERCASE |
| `--replace-dash <CHAR>` | none | Replace dashes in names |
| `--prefix <PREFIX>` | none | Add prefix to names |

### Output formats

**`--format env`**

```
ANTHROPIC_API_KEY=sk-ant-xxx
DB_URL=postgresql://user:pass@host/db
GITHUB_TOKEN=ghp_xxx
```

Standard `.env` format. Values containing spaces, `#`, `"`, or newlines are double-quoted with proper escaping.

**`--format json`**

```json
{
  "secrets": [
    {
      "name": "anthropic-api-key",
      "value": "sk-ant-xxx",
      "version": 1,
      "created": "2026-02-10T08:30:00Z",
      "modified": "2026-02-10T08:30:00Z"
    }
  ]
}
```

Full metadata export. Useful for backup or migration.

### Name transformation

Same flags as `authy env` and `authy run`. Transformation order: (1) replace-dash, (2) prefix, (3) uppercase.

Default for `--format env`: `--uppercase --replace-dash _` is NOT applied automatically. The user controls the transform. This avoids surprising name changes.

### Scoped export

```bash
# Export only secrets the deploy policy allows
authy export --format env --scope deploy --uppercase --replace-dash _
```

Without `--scope`, exports all secrets in the vault. Requires master key auth (no session token — exporting all secrets is a privileged operation).

With `--scope`, works with session tokens (read-only, scoped).

### Redirect to file

```bash
authy export --format env --scope dev --uppercase --replace-dash _ > .env.dev
```

Output goes to stdout. User redirects to file.

## Relationship to `authy env`

`authy env` and `authy export --format env` produce similar output. The distinction:

| | `authy env` | `authy export --format env` |
|---|---|---|
| Purpose | Source into current shell | Generate a `.env` file |
| Default output | `export KEY='VALUE'` | `KEY=VALUE` |
| Requires `--scope` | Yes | Optional |
| Quoting | Shell quoting (single quotes) | Dotenv quoting (double quotes if needed) |
| Typical use | `eval "$(authy env ...)"` | `authy export ... > .env` |

They share the same underlying secret-resolution and name-transformation logic.

## Edge Cases

- No secrets match scope: output nothing, exit 0
- Secret value contains `=`: fine in `.env` format (first `=` is the delimiter)
- Secret value contains newlines: double-quote with `\n` escaping
- `--format env` without `--scope`: requires master key auth (not token)
- Naming conflict: `authy audit export` vs `authy export` — resolve by making `export` a top-level command separate from `audit export`

## Acceptance Criteria

- [ ] `authy export --format env` outputs valid `.env` format
- [ ] `authy export --format json` outputs full secret metadata
- [ ] `--scope` filters by policy
- [ ] Name transformation flags work correctly
- [ ] Output goes to stdout for redirection
- [ ] Special characters in values are properly escaped
- [ ] Without `--scope`, requires master key auth
- [ ] With `--scope`, works with session tokens
- [ ] Does not conflict with `authy audit export`
