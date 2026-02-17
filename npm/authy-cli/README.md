# authy-cli

A CLI secrets store & dispatch tool built for AI agents.

Authy stores encrypted secrets locally and dispatches them to agents with policy-based scoping, short-lived session tokens, and audit logging. No server required.

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

# Retrieve it
authy get db-url

# Launch the admin TUI (secrets never touch shell history)
authy admin --keyfile ~/.authy/keys/master.key
```

## Agent Workflow

```bash
# Create a scoped policy
authy policy create deploy-agent --allow "db-*" --deny "openai-*"

# Create a short-lived session token
authy session create --scope deploy-agent --ttl 1h

# Agent uses the token to read only allowed secrets
export AUTHY_TOKEN="authy_v1...."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy get db-url          # works
authy get openai-api-key  # denied

# Or inject secrets into a subprocess
authy run --scope deploy-agent -- ./deploy.sh
```

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
