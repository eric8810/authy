# authy

A CLI secrets store & dispatch tool built for AI agents.

Authy stores encrypted secrets locally and dispatches them to agents with policy-based scoping, short-lived session tokens, and audit logging. No server required.

## Why

Existing secrets tools don't solve the agent problem:

- **pass/gopass** — no scoped access; GPG key = full access to everything
- **HashiCorp Vault** — requires running a server; overkill for local/CI use
- **1Password CLI** — great `op run` model but proprietary and requires an account
- **SOPS** — file encryption, not a runtime dispatch tool

Authy fills the gap: a single binary that gives each agent **only the secrets it needs, only for as long as it needs them**.

## Features

- **Encrypted vault** — single `age`-encrypted file, no metadata leakage
- **Scoped policies** — glob-based allow/deny rules per agent scope
- **Session tokens** — short-lived, read-only, HMAC-validated
- **Subprocess injection** — `authy run` injects secrets as env vars into a child process only
- **Audit log** — JSONL with HMAC chain for tamper detection
- **Headless operation** — works without interactive prompts via keyfile + token
- **Pipe-friendly** — `authy get` outputs raw values to stdout, diagnostics to stderr

## Install

**npm (recommended)**

```bash
npm install -g authy-cli
# or: npx authy-cli --help
```

**Install script**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/eric8810/authy/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/eric8810/authy/main/install.ps1 | iex
```

**From source**

```bash
cargo build --release
# Binary at target/release/authy
```

## Quick Start

```bash
# Initialize a vault with a keyfile
authy init --generate-keyfile ~/.authy/keys/master.key

# Recommended: Launch the admin TUI (secrets never touch shell history)
authy admin --keyfile ~/.authy/keys/master.key
# Or use passphrase: authy admin

# CLI alternative: store secrets via interactive prompt
authy store db-url
# Type secret, then press Ctrl+D

# Or from file (for scripts)
authy store db-url < ~/.secrets/db-credential

authy get db-url
authy list
```

## Admin TUI

`authy admin` launches an interactive terminal UI for managing secrets, policies, and sessions. Secrets entered through the TUI never appear in shell history, process arguments, or environment variables.

```bash
authy admin                                        # passphrase prompt
authy admin --keyfile ~/.authy/keys/master.key     # keyfile auth
```

The TUI provides:
- **Secrets** — store, reveal (masked by default, auto-close), rotate, delete
- **Policies** — create, edit, delete, test against secret names
- **Sessions** — create scoped tokens, revoke individual or all
- **Audit** — scrollable log, text filter, HMAC chain verification

Press `?` inside the TUI for key bindings.

## Agent Workflow

```bash
# 1. Admin creates a policy restricting what the agent can see
authy policy create deploy-agent \
  --allow "db-*" \
  --allow "github-token" \
  --deny "openai-*"

# 2. Admin creates a short-lived session token
authy session create --scope deploy-agent --ttl 1h
# Prints: authy_v1.dGhpcyBpcyBhIDMyIGJ5dGUgcmFuZG9t...

# 3. Agent authenticates with the token
export AUTHY_TOKEN="authy_v1.dGhpcyBpcyBhIDMyIGJ5dGUgcmFuZG9t..."
export AUTHY_KEYFILE=~/.authy/keys/master.key

# 4. Agent can only read allowed secrets
authy get db-url              # works
authy get openai-api-key      # denied

# 5. Or inject all allowed secrets into a subprocess
authy run --scope deploy-agent --uppercase --replace-dash _ -- ./deploy.sh
# deploy.sh sees DB_URL and GITHUB_TOKEN in its env, nothing else
```

## Commands

```
authy init                        Initialize a new vault
authy store <name>                Store a secret (reads from stdin)
authy get <name>                  Retrieve a secret value
authy list                        List secret names
authy remove <name>              Remove a secret
authy rotate <name>              Rotate a secret value

authy policy create <name>       Create an access policy
authy policy show <name>         Show policy details
authy policy update <name>       Modify a policy
authy policy list                List all policies
authy policy remove <name>       Remove a policy
authy policy test --scope <s> <name>  Test if a scope can access a secret

authy session create             Create a scoped session token
authy session list               List active sessions
authy session revoke <id>        Revoke a session
authy session revoke-all         Revoke all sessions

authy run --scope <s> -- <cmd>   Run a command with injected secrets
authy env --scope <s>            Output secrets as env vars (shell/dotenv/json)
authy import <file>              Import secrets from a .env file
authy export --format <fmt>      Export secrets as .env or JSON

authy audit show                 Show audit log entries
authy audit verify               Verify audit log integrity
authy audit export               Export audit log as JSON

authy config show                Show configuration

authy admin                      Launch admin TUI (interactive management)
```

All read commands support `--json` for structured JSON output.

### Env Command

Output secrets as environment variables in shell, dotenv, or JSON format:

```bash
# Shell format (sourceable)
eval "$(authy env --scope agent --format shell --uppercase --replace-dash '_')"

# Dotenv format
authy env --scope agent --format dotenv > .env

# JSON format
authy env --scope agent --format json | jq .
```

### Import / Export

Migrate from .env files or export for backup:

```bash
# Import from .env (transforms UPPER_SNAKE to lower-kebab by default)
authy import .env
authy import .env --keep-names --force
authy import .env --dry-run
authy import -   # read from stdin

# Export
authy export --format env --scope agent
authy export --format json
```

### Non-Interactive Mode

When stdin is not a TTY (e.g., in CI/CD or agent scripts), authy fails fast instead of prompting. Set credentials via environment variables:

```bash
export AUTHY_KEYFILE=~/.authy/keys/master.key
# or: export AUTHY_PASSPHRASE=...
# or: export AUTHY_TOKEN=... (with AUTHY_KEYFILE)
```

Set `AUTHY_NON_INTERACTIVE=1` to force non-interactive mode even with a TTY.

## Authentication Modes

| Mode | Use case | How |
|---|---|---|
| Passphrase | Human admin, interactive | Prompted at runtime |
| Keyfile | Automation, headless | `--keyfile` or `AUTHY_KEYFILE` |
| Session token | Agent access, scoped | `--token` or `AUTHY_TOKEN` (requires keyfile too) |

Session tokens are **read-only** — agents cannot store, remove, or modify secrets or policies.

## Security Model

- Secrets are encrypted at rest with [age](https://age-encryption.org/) (X25519)
- Secrets never appear in shell history, process argv, or parent environment
- Session tokens use HMAC-SHA256 with constant-time validation
- Tokens have a scannable `authy_v1.` prefix for leak detection
- Policies are stored inside the encrypted vault (tamper-proof)
- The audit log uses an HMAC chain — any modification breaks the chain
- All secret-holding types are zeroized on drop

## File Layout

```
~/.authy/
  vault.age           Encrypted vault (secrets + policies + sessions)
  audit.log           Append-only audit log (JSONL)
  authy.toml          Configuration (optional)
  keys/
    master.key        age identity (private key)
```

## Agent Skills

Authy ships with an [Agent Skills](https://agentskills.io) compatible skill at `skills/authy/SKILL.md`. This works with Claude Code, Cursor, OpenClaw, and 38+ other AI coding agents.

**Install via npx:**

```bash
npx skills add eric8810/authy
```

**Install via ClawHub:**

```bash
clawhub install eric8810/authy
```

**Manual install (Claude Code):**

```bash
cp -r skills/authy ~/.claude/skills/authy
```

The skill teaches agents how to retrieve secrets, list available credentials, inject secrets into subprocesses, and handle errors — without ever reading `.env` files or hardcoding credentials.

## Documentation

- [README.md](README.md) — Project overview and quick start
- [ARCHITECTURE.md](ARCHITECTURE.md) — System design and data flow
- [SECURITY.md](SECURITY.md) — Security model and threat analysis
- [AI_AGENT_GUIDE.md](AI_AGENT_GUIDE.md) — Claude Code, OpenClaw & MCP integration

## License

MIT
