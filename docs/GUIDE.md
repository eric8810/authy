# Authy — Detailed Guide

Everything beyond the [README quick start](../README.md). Reference for authentication, commands, configuration, and agent integration.

## Authentication Modes

| Mode | Use case | How |
|------|----------|-----|
| Passphrase | Human admin, interactive | Prompted at runtime |
| Keyfile | Automation, headless | `--keyfile` or `AUTHY_KEYFILE` env var |
| Session token | Agent access, scoped | `--token` or `AUTHY_TOKEN` (requires keyfile too) |

Session tokens are **read-only** — agents cannot store, remove, or modify secrets or policies.

### Run-Only Mode

Run-only restricts agents to `authy run` (subprocess injection) and `authy list` (names only). Commands that expose secret values (`get`, `env`, `export`) return exit code 4.

Run-only can be set at either level — either one triggers the restriction:

```bash
# Token-level
authy session create --scope my-scope --ttl 1h --run-only

# Policy-level
authy policy create agent-scope --allow "*" --run-only

# Toggle on existing policy
authy policy update agent-scope --run-only true
```

### Non-Interactive Mode

When stdin is not a TTY (CI/CD, agent scripts), authy fails fast instead of prompting. Set credentials via environment:

```bash
export AUTHY_KEYFILE=~/.authy/keys/master.key
# or: export AUTHY_PASSPHRASE=...
# or: export AUTHY_TOKEN=... (with AUTHY_KEYFILE)
```

Set `AUTHY_NON_INTERACTIVE=1` to force non-interactive mode even with a TTY.

## Commands

### Secrets

```bash
authy store <name>                # reads value from stdin, Ctrl+D to finish
authy get <name>                  # output value to stdout
authy list [--scope <s>] [--json] # list secret names
authy remove <name>               # delete a secret
authy rotate <name>               # update value, bumps version
```

### Environment Variable Output

Output secrets as environment variables in different formats:

```bash
# Shell format (sourceable)
eval "$(authy env --scope agent --format shell --uppercase --replace-dash '_')"

# Dotenv format
authy env --scope agent --format dotenv > .env

# JSON format
authy env --scope agent --format json | jq .
```

Options:
- `--format shell|dotenv|json` — output format (default: shell)
- `--uppercase` — transform names to UPPER_CASE
- `--replace-dash <char>` — replace `-` in names (e.g., `_`)
- `--no-export` — omit `export` prefix in shell format

### Import / Export

```bash
# Import from .env files
authy import .env                 # transforms UPPER_SNAKE to lower-kebab
authy import .env --keep-names    # preserve original names
authy import .env --force         # overwrite existing secrets
authy import .env --prefix api    # prefix all names with "api-"
authy import .env --dry-run       # preview without writing
authy import -                    # read from stdin

# Export
authy export --format env [--scope <s>]
authy export --format json
```

### Policies

```bash
authy policy create <name> --allow "db-*" --deny "prod-*"
authy policy create <name> --allow "*" --run-only
authy policy show <name>
authy policy update <name> --run-only true
authy policy list [--json]
authy policy remove <name>
authy policy test --scope <s> <name>   # test if scope can access a secret
```

Policy evaluation: deny overrides allow, default deny.

### Sessions

```bash
authy session create --scope <policy> --ttl <duration> [--run-only]
authy session list [--json]
authy session revoke <id>
authy session revoke-all
```

Token format: `authy_v1.<base64>` — scannable prefix for leak detection.

### Subprocess Injection

```bash
authy run [--scope <s>] [--uppercase] [--replace-dash <c>] -- <command> [args...]
```

Secrets matching the scope are injected as environment variables into the child process. The parent process (agent) never sees them.

### Audit

```bash
authy audit show [--json]         # show log entries
authy audit verify                # verify HMAC chain integrity
authy audit export                # export as JSON
```

### Project Config

```bash
authy project-info                # show .authy.toml config
authy alias [scope] [tools...]    # generate shell aliases
authy hook <shell>                # output shell hook code
```

### Admin

```bash
authy admin [--keyfile <path>]    # launch TUI
authy config show                 # show configuration
```

## Project Config (`.authy.toml`)

Auto-discovered from current directory upward. Makes `--scope` optional on `run`, `env`, `export`:

```toml
[authy]
scope = "my-project"
keyfile = "~/.authy/keys/master.key"
uppercase = true
replace_dash = "_"
aliases = ["claude", "aider"]     # tools to generate aliases for
```

### Shell Hook

Auto-activate project config on `cd` (like direnv):

```bash
eval "$(authy hook bash)"    # add to ~/.bashrc
eval "$(authy hook zsh)"     # add to ~/.zshrc
authy hook fish | source     # add to ~/.config/fish/config.fish
```

### Shell Aliases

Generate aliases that wrap tools with `authy run`:

```bash
authy alias --from-project        # uses .authy.toml aliases list
authy alias my-scope claude aider # explicit scope and tools
```

## File Layout

```
~/.authy/
  vault.age           Encrypted vault (secrets + policies + sessions)
  audit.log           Append-only audit log (JSONL)
  authy.toml          Configuration (optional)
  keys/
    master.key        age identity (private key)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication failed |
| 3 | Not found (secret, policy, session) |
| 4 | Access denied / run-only restriction |
| 5 | Vault error (corrupt, missing) |
| 6 | Token invalid, expired, or revoked |
| 7 | Subprocess error |

## JSON Output

All read commands support `--json`. Errors with `--json` emit to stderr:

```json
{"error": {"code": "access_denied", "message": "Run-only mode", "exit_code": 4}}
```

## Agent Skills

Install the skill for AI coding agents:

```bash
npx skills add eric8810/authy          # Agent Skills standard
clawhub install eric8810/authy         # ClawHub
cp -r skills/authy ~/.claude/skills/   # Manual (Claude Code)
```

The skill teaches agents only `authy run` and `authy list` — agents never learn commands that expose values.
