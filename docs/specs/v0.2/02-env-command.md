# 02 — Env Command (`authy env`)

## Summary

New `authy env` command that outputs scoped secrets as shell-sourceable environment variable declarations. The bridge between Authy's encrypted vault and the `export FOO=bar` world that every tool and agent already understands.

## Motivation

Today, `authy run --scope X -- cmd` spawns a child process with env vars injected. This works for launching a single process, but doesn't help when:

1. A shell session needs multiple tools to see the same secrets
2. A `SessionStart` hook needs to write env vars to `CLAUDE_ENV_FILE`
3. A CI pipeline step needs to source secrets into the current shell
4. A container entrypoint needs to set up the environment before exec

`authy env` produces the text output that feeds these use cases. It's the composable building block — the output can be `eval`'d, appended to files, piped, or redirected.

## Current Behavior

No `authy env` command exists. The closest is `authy run`, which spawns a subprocess.

## Proposed Behavior

### Basic usage

```bash
$ authy env --scope deploy
export DB_URL='postgresql://user:pass@host/db'
export GITHUB_TOKEN='ghp_xxxxxxxxxxxx'
```

Output is `export KEY='VALUE'` format, one per line. Directly sourceable by bash/zsh.

### Source it

```bash
eval "$(authy env --scope deploy)"
# DB_URL and GITHUB_TOKEN are now in the current shell
```

### Write to a file

```bash
authy env --scope deploy >> "$CLAUDE_ENV_FILE"
```

### Flags

```
authy env --scope <SCOPE> [--uppercase] [--replace-dash <CHAR>] [--prefix <PREFIX>] [--format <FORMAT>] [--no-export]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--scope <SCOPE>` | required | Policy scope to filter secrets |
| `--uppercase` | off | Convert secret names to UPPERCASE |
| `--replace-dash <CHAR>` | none | Replace `-` in names with given character |
| `--prefix <PREFIX>` | none | Prepend prefix to each variable name |
| `--format <FORMAT>` | `shell` | Output format (see below) |
| `--no-export` | off | Omit `export` keyword (just `KEY='VALUE'`) |

These flags mirror `authy run` for consistency.

### Output formats

**`--format shell`** (default)

```bash
export DB_URL='postgresql://user:pass@host/db'
export GITHUB_TOKEN='ghp_xxxxxxxxxxxx'
```

**`--format dotenv`**

```
DB_URL=postgresql://user:pass@host/db
GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

Standard `.env` format (no `export`, no quoting unless value contains special chars). Values with spaces, `#`, or newlines are double-quoted.

**`--format json`**

```json
{
  "DB_URL": "postgresql://user:pass@host/db",
  "GITHUB_TOKEN": "ghp_xxxxxxxxxxxx"
}
```

Flat key-value JSON object.

### Name transformation

Transformation order: (1) replace-dash, (2) prefix, (3) uppercase.

```bash
$ authy env --scope deploy --uppercase --replace-dash _ --prefix AUTHY_
export AUTHY_DB_URL='postgresql://user:pass@host/db'
export AUTHY_GITHUB_TOKEN='ghp_xxxxxxxxxxxx'
```

This matches the existing `authy run` transformation behavior.

### Authentication

Supports all auth methods: passphrase, keyfile, session token. When using a session token, the token's scope is used and `--scope` is optional (defaults to token scope).

## Edge Cases

- Empty scope (no secrets match policy): output nothing, exit 0
- Secret value contains single quotes: escape as `'\''` in shell format
- Secret value contains newlines: use `$'...\n...'` quoting in shell format; `"..."` with `\n` in dotenv
- `--scope` required unless using session token (which has implicit scope)
- `--format json` ignores `--no-export` (not applicable)

## Relationship to `authy run`

`authy env` and `authy run` share the same policy evaluation and name transformation logic. Internally, `authy run` could be refactored to use the same secret-resolution code as `authy env`, then pass the result to `Command::envs()`.

## Use Cases

**Claude Code SessionStart hook:**
```bash
#!/bin/bash
if [ -n "$CLAUDE_ENV_FILE" ]; then
  authy env --scope claude-code --uppercase --replace-dash _ >> "$CLAUDE_ENV_FILE"
fi
```

**Docker container entrypoint:**
```bash
#!/bin/bash
eval "$(authy env --scope $AGENT_SCOPE --uppercase --replace-dash _)"
exec "$@"
```

**CI pipeline step:**
```bash
eval "$(authy env --scope ci-deploy --uppercase --replace-dash _)"
./deploy.sh
```

**Pipe to another tool:**
```bash
authy env --scope dev --format json | jq '.DB_URL'
```

## Acceptance Criteria

- [ ] `authy env --scope <scope>` outputs shell-sourceable `export` statements
- [ ] `--format shell`, `--format dotenv`, `--format json` all work
- [ ] `--uppercase`, `--replace-dash`, `--prefix` transform names correctly
- [ ] `--no-export` omits `export` keyword in shell format
- [ ] Values with special characters are properly escaped per format
- [ ] Empty result (no matching secrets) outputs nothing, exits 0
- [ ] Works with all auth methods (passphrase, keyfile, token)
- [ ] Audit log records the access (operation: `env_export`, lists scope)
