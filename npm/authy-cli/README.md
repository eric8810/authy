# authy-cli

A CLI secrets store & dispatch tool built for AI agents.

Authy stores encrypted secrets locally and dispatches them to agents with policy-based scoping, run-only tokens, and audit logging. Agents inject secrets into subprocesses but never see the values. No server required.

## Install

```bash
npx authy-cli --help
```

Or install globally:

```bash
npm install -g authy-cli
authy --help
```

### Other install methods

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/eric8810/authy/main/install.sh | sh

# Windows (PowerShell)
irm https://raw.githubusercontent.com/eric8810/authy/main/install.ps1 | iex
```

## Quick Start

```bash
# Initialize a vault with a keyfile
authy init --generate-keyfile ~/.authy/keys/master.key

# Store a secret (reads from stdin)
authy store db-url

# Launch the admin TUI (secrets never touch shell history)
authy admin --keyfile ~/.authy/keys/master.key
```

## Agent Workflow

```bash
# Create a run-only policy — agent can inject secrets but never read values
authy policy create deploy-agent --allow "db-*" --deny "openai-*" --run-only

# Create a run-only session token
authy session create --scope deploy-agent --ttl 1h --run-only

# Agent injects secrets into a subprocess (the only allowed path)
export AUTHY_TOKEN="authy_v1...."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy run --scope deploy-agent --uppercase --replace-dash _ -- ./deploy.sh

# Direct value access is blocked
authy get db-url              # Error: Run-only mode
authy list --json             # OK — shows names only
```

## What's New in v0.2.0

- **Run-only mode** — `--run-only` on tokens and policies blocks `get`/`env`/`export`
- **JSON output** — `--json` global flag on all read commands
- **`authy env`** — output secrets as shell/dotenv/json
- **`authy import`** — import from .env files
- **`authy export`** — export as .env or JSON
- **Non-interactive mode** — fails fast in CI/CD without prompting

## Supported Platforms

| Platform | Architecture |
|----------|-------------|
| Linux | x64, arm64 |
| macOS | x64, arm64 |
| Windows | x64 |

## Documentation

For full documentation, see the [GitHub repository](https://github.com/eric8810/authy).

## License

MIT
