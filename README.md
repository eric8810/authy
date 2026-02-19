# authy

Encrypted secrets for AI agents. Single binary, no server, no accounts.

## 30-Second Start

```bash
npm install -g authy-cli

authy init --generate-keyfile ~/.authy/keys/master.key
authy store api-key                          # type value, Ctrl+D
authy run --scope "*" -- ./my-script.sh      # script sees $API_KEY in its env
```

That's it. Secret is encrypted in the vault, injected into the subprocess, never in your shell history or `.env` files.

## Install

```bash
# npm (recommended)
npm install -g authy-cli

# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/eric8810/authy/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/eric8810/authy/main/install.ps1 | iex

# From source
cargo build --release
```

## How It Works

```
You store secrets    →  authy vault (encrypted)
Agent runs command   →  authy run injects secrets as env vars into subprocess
Subprocess finishes  →  env vars gone, nothing on disk
```

Secrets never appear in shell history, `.env` files, process arguments, or LLM context.

## Give Agents Scoped Access

```bash
# Create a policy — agent only sees db-* secrets
authy policy create backend --allow "db-*" --run-only

# Create a time-limited token
authy session create --scope backend --ttl 1h --run-only
# → authy_v1.dGhpcyBpcyBhIDMyIGJ5dGUgcmFuZG9t...

# Agent uses the token — can only inject, never read values
export AUTHY_TOKEN="authy_v1...."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy run --scope backend --uppercase --replace-dash '_' -- node server.js
```

`--run-only` means the agent can inject secrets into subprocesses but can never read values directly. `authy get`, `authy env`, `authy export` all return an error.

## Project Config

Drop `.authy.toml` in your project root. No more `--scope` flags:

```toml
[authy]
scope = "my-project"
keyfile = "~/.authy/keys/master.key"
uppercase = true
replace_dash = "_"
```

```bash
authy run -- ./deploy.sh          # scope inferred from .authy.toml
eval "$(authy hook bash)"         # auto-activate on cd (like direnv)
```

## Migrate from .env

```bash
authy import .env                 # imports all keys, transforms names
authy import .env --dry-run       # preview first
```

## Admin TUI

`authy admin` — manage secrets, policies, sessions, and audit logs interactively. Secrets entered through the TUI never touch shell history.

```bash
authy admin --keyfile ~/.authy/keys/master.key
```

## Agent Skills

Works with Claude Code, Cursor, OpenClaw, and 38+ AI coding agents:

```bash
npx skills add eric8810/authy
```

The skill teaches agents to use `authy run` (inject secrets) and `authy list` (discover names). Agents never learn commands that expose values.

## Security

- **age encryption** (X25519) — vault encrypted at rest
- **HMAC-SHA256 session tokens** — short-lived, read-only, constant-time validation
- **Glob-based policies** — deny overrides allow, default deny
- **HMAC-chained audit log** — tamper detection on every entry
- **Zeroize on drop** — all secret-holding memory wiped when freed
- **Run-only mode** — agents can inject but never read

## All Commands

<details>
<summary>Full command reference</summary>

```
Basics
  authy init                        Initialize a new vault
  authy store <name>                Store a secret (reads from stdin)
  authy get <name>                  Retrieve a secret value
  authy list                        List secret names
  authy remove <name>              Remove a secret
  authy rotate <name>              Rotate a secret value

Policies
  authy policy create <name>       Create an access policy
  authy policy show <name>         Show policy details
  authy policy update <name>       Modify a policy
  authy policy list                List all policies
  authy policy remove <name>       Remove a policy
  authy policy test --scope <s> <name>  Test access

Sessions
  authy session create             Create a scoped session token
  authy session list               List active sessions
  authy session revoke <id>        Revoke a session
  authy session revoke-all         Revoke all sessions

Agent Commands
  authy run [--scope <s>] -- <cmd> Run a command with injected secrets
  authy env [--scope <s>]          Output secrets as env vars
  authy import <file>              Import from .env file
  authy export --format <fmt>      Export as .env or JSON

Project
  authy project-info               Show .authy.toml config
  authy alias [scope] [tools...]   Generate shell aliases
  authy hook <shell>               Shell hook for auto-activation

Audit
  authy audit show                 Show audit log
  authy audit verify               Verify log integrity
  authy audit export               Export log as JSON

Admin
  authy admin                      Launch admin TUI
  authy config show                Show configuration
```

All read commands support `--json`. `--scope` is optional when `.authy.toml` is present.

</details>

## Docs

- [docs/GUIDE.md](docs/GUIDE.md) — full command reference, auth modes, config, exit codes
- [ARCHITECTURE.md](ARCHITECTURE.md) — system design
- [SECURITY.md](SECURITY.md) — threat model
- [CHANGELOG.md](CHANGELOG.md) — version history

## License

MIT
