# AI Agent Integration Guide

Secure secrets management for AI agents, automation pipelines, and developer tools.

---

## The Problem

AI agents need API keys, database credentials, and other secrets to function. Current approaches have serious issues:

| Issue | Description |
|-------|-------------|
| **Plaintext config files** | Tools like OpenClaw store API keys in `~/.openclaw/openclaw.json` in plain text |
| **Hardcoded in project files** | MCP servers, CI configs, and `.env` files contain raw secrets |
| **No scope control** | Once a secret is in the environment, the agent sees everything |
| **Audit gaps** | No way to track which secrets the agent accessed or when |
| **No revocation** | Can't cut off a compromised agent without rotating the actual secret |

Authy solves this with encrypted storage, scoped policies, short-lived tokens, subprocess injection, and a tamper-evident audit log. No server, no accounts, single binary.

---

## Managing Secrets with the TUI

The recommended way to manage secrets is through the admin TUI. Secrets entered through the TUI never appear in shell history, process arguments, or environment variables.

```bash
authy admin --keyfile ~/.authy/keys/master.key
```

The TUI provides four sections:

| Section | Actions |
|---------|---------|
| **Secrets** | Store, reveal (auto-masked, auto-close), rotate, delete |
| **Policies** | Create, edit, delete, test against secret names |
| **Sessions** | Create scoped tokens with TTL, revoke individual or all |
| **Audit** | Scrollable log with text filter, HMAC chain verification |

Press `?` inside the TUI for key bindings. Press `1`–`4` or `Tab` to switch sections.

All management can also be done via CLI commands — see the [README](README.md) for the full command reference.

---

## Core Concepts

### Policies

Policies use glob patterns with deny-override semantics. An agent scoped to a policy can only access secrets that match an allow pattern and don't match any deny pattern.

```bash
authy policy create my-agent \
  --allow "api-*" \
  --allow "db-dev-*" \
  --deny "db-prod-*" \
  --deny "*-admin"
```

Test a policy before deploying:

```bash
authy policy test --scope my-agent db-dev-host    # ALLOWED
authy policy test --scope my-agent db-prod-host   # DENIED
```

### Session Tokens

Session tokens provide time-limited, read-only access scoped to a policy. Agents cannot store, remove, or modify secrets with a token.

```bash
# Create a token valid for 8 hours
authy session create --scope my-agent --ttl 8h
# Output: authy_v1.dGhpcyBpcyBhIDMyIGJ5dGUg...

# Create a run-only token — agent can only use `authy run`, not `get`/`env`/`export`
authy session create --scope my-agent --ttl 8h --run-only
```

Tokens have an `authy_v1.` prefix for leak detection in logs and code scanning tools.

### Run-Only Mode

For maximum security with AI agents, use `--run-only` on tokens and/or policies. This ensures agents can only inject secrets into subprocesses via `authy run` — they cannot read secret values directly.

```bash
# Policy-level: all tokens using this scope are restricted
authy policy create agent-scope --allow "api-*" --allow "db-*" --run-only

# Token-level: this specific token is restricted
authy session create --scope agent-scope --ttl 8h --run-only
```

When run-only is active:
- `authy run` works normally (secrets injected into subprocess)
- `authy list` works (shows names only, no values)
- `authy get`, `authy env`, `authy export` return exit code 4

Either token-level or policy-level run-only triggers the restriction.

### Subprocess Injection

`authy run` spawns a child process with allowed secrets injected as environment variables. Secrets never touch the parent environment, shell history, or process arguments.

```bash
authy run --scope my-agent --uppercase --replace-dash _ -- ./my-script.sh
# my-script.sh sees DB_DEV_HOST, API_KEY, etc. — nothing else
```

Flags:
- `--uppercase` — `db-url` becomes `DB_URL`
- `--replace-dash _` — `db-url` becomes `db_url`
- `--prefix AUTHY_` — `db-url` becomes `AUTHY_db_url`

---

## Claude Code Integration

Claude Code lacks a built-in secrets vault. It relies on environment variables and file-based deny rules. Authy fills this gap.

### Approach 1: Launch Claude Code with Injected Secrets

The simplest approach — wrap the `claude` command with `authy run`:

```bash
# Store secrets
echo "sk-ant-..." | authy store anthropic-api-key
echo "ghp_..." | authy store github-token
echo "BSAxxxx..." | authy store brave-api-key

# Create a policy
authy policy create claude-code \
  --allow "anthropic-*" \
  --allow "github-*" \
  --allow "brave-*" \
  --deny "prod-*"

# Launch Claude Code with scoped secrets
authy run --scope claude-code --uppercase --replace-dash _ -- claude
```

Claude Code inherits all environment variables from its parent process. Secrets like `ANTHROPIC_API_KEY`, `GITHUB_TOKEN`, and `BRAVE_API_KEY` are available to Claude Code and its MCP servers without appearing in any config file.

### Approach 2: MCP Server Secrets via `.mcp.json`

Claude Code's `.mcp.json` supports `${VAR}` expansion for environment variables. Combine this with `authy run` to keep secrets out of config files:

```json
{
  "mcpServers": {
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/mcp-server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "${BRAVE_API_KEY}"
      }
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/mcp-server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

The `${VAR}` references resolve from the environment at MCP server startup. Since `authy run` injected these variables, no secrets are hardcoded in `.mcp.json`, and the file is safe to commit.

### Approach 3: MCP Servers That Require Secrets in Args

Some MCP servers (like the Postgres server) require secrets as command-line arguments, not environment variables. Use a wrapper script:

```bash
#!/bin/bash
# ~/.authy/mcp-postgres.sh
USER=$(authy get db-user)
PASS=$(authy get db-password)
HOST=$(authy get db-host)
DB=$(authy get db-name)

exec npx -y @anthropic-ai/mcp-server-postgres "postgresql://${USER}:${PASS}@${HOST}/${DB}"
```

Configure in `.mcp.json`:

```json
{
  "mcpServers": {
    "postgres": {
      "command": "authy",
      "args": ["run", "--scope", "claude-db", "--", "/bin/bash", "~/.authy/mcp-postgres.sh"]
    }
  }
}
```

### Approach 4: Project Config + Shell Hook

The most seamless approach — drop a `.authy.toml` in your project and let the shell hook handle everything:

```toml
# .authy.toml (in project root)
[authy]
scope = "claude-code"
keyfile = "~/.authy/keys/master.key"
uppercase = true
replace_dash = "_"
aliases = ["claude", "aider"]
```

```bash
# Add shell hook to ~/.bashrc or ~/.zshrc (one-time setup)
eval "$(authy hook bash)"    # or: eval "$(authy hook zsh)"
```

Now when you `cd` into the project, Authy automatically:
1. Sets `AUTHY_KEYFILE` from the config
2. Creates aliases so `claude` → `authy run --scope claude-code --uppercase --replace-dash _ -- claude`
3. Cleans up aliases when you leave the project

No manual wrapping needed — just type `claude`.

### Approach 5: Shell Alias (Manual)

If you prefer explicit control over the generated alias:

```bash
# Generate aliases from .authy.toml
eval "$(authy alias --from-project)"

# Or generate a one-off alias
eval "$(authy alias claude-code claude aider)"
```

Or add a static alias to `~/.bashrc` / `~/.zshrc`:

```bash
alias claude='authy run --scope claude-code --uppercase --replace-dash _ -- claude'
```

### Protecting Secrets from Claude Code

Claude Code can read `.env` files and other sensitive files unless explicitly blocked. Combine Authy with Claude Code's `permissions.deny`:

```json
{
  "permissions": {
    "deny": [
      "Read(./.env)",
      "Read(./.env.*)",
      "Read(./secrets/**)",
      "Read(~/.authy/**)"
    ]
  }
}
```

This prevents Claude Code from directly reading your vault, keyfiles, or `.env` files. Secrets are only accessible through the environment variables injected by `authy run`.

### Giving Claude Code a Session Token

For team setups, an admin can create a time-limited token so developers can run Claude Code with scoped access without sharing the master keyfile:

```bash
# Admin creates token
TOKEN=$(authy session create --scope claude-code --ttl 8h --label "dev session")

# Developer sets environment
export AUTHY_TOKEN="$TOKEN"
export AUTHY_KEYFILE=~/.authy/keys/master.key

# Developer launches Claude Code
authy run --scope claude-code --uppercase --replace-dash _ -- claude
```

The session token is read-only — Claude Code (or any agent using it) cannot store, delete, or modify secrets.

---

## OpenClaw Integration

OpenClaw stores API keys in plaintext in `~/.openclaw/openclaw.json` and `~/.openclaw/.env`. Infostealer malware has been observed targeting these files. Authy replaces this with encrypted storage and scoped access.

### Direct Launch

```bash
# Store the secrets OpenClaw needs
echo "sk-ant-..." | authy store anthropic-api-key
echo "your-discord-token" | authy store discord-token
echo "your-telegram-token" | authy store telegram-bot-token

# Create a policy
authy policy create openclaw \
  --allow "anthropic-*" \
  --allow "discord-*" \
  --allow "telegram-*" \
  --deny "prod-*"

# Launch OpenClaw with secrets injected
authy run --scope openclaw --uppercase --replace-dash _ -- openclaw
```

OpenClaw reads API keys from environment variables (`ANTHROPIC_API_KEY`, `DISCORD_TOKEN`, `TELEGRAM_BOT_TOKEN`). With `authy run`, these come from the encrypted vault instead of plaintext config files.

### OpenClaw with Docker

```bash
docker run \
  -v ~/.authy:/root/.authy:ro \
  -e AUTHY_KEYFILE=/root/.authy/keys/master.key \
  --entrypoint authy \
  openclaw/openclaw:latest \
  run --scope openclaw --uppercase --replace-dash _ -- openclaw
```

Or use a Dockerfile that includes Authy:

```dockerfile
FROM openclaw/openclaw:latest

# Install authy binary
COPY --from=authy-builder /usr/local/bin/authy /usr/local/bin/authy

# Don't bake secrets into the image
COPY openclaw.json /config/openclaw.json

ENTRYPOINT ["authy", "run", "--scope", "openclaw", "--uppercase", "--replace-dash", "_", "--", "openclaw", "--config", "/config/openclaw.json"]
```

Mount the vault at runtime:

```bash
docker run -v ~/.authy:/root/.authy:ro my-openclaw-image
```

### Per-Agent Scoping

If you run multiple OpenClaw agents with different permissions:

```bash
# Agent that only talks to Discord
authy policy create openclaw-discord \
  --allow "anthropic-api-key" \
  --allow "discord-*"

# Agent that only talks to Telegram
authy policy create openclaw-telegram \
  --allow "anthropic-api-key" \
  --allow "telegram-*"

authy run --scope openclaw-discord --uppercase --replace-dash _ -- openclaw --profile discord
authy run --scope openclaw-telegram --uppercase --replace-dash _ -- openclaw --profile telegram
```

---

## General Agent Patterns

Authy works with any agent or tool that reads secrets from environment variables or stdin.

### Pattern: CI/CD Pipeline

```bash
# Store deployment credentials
echo "AKIAIOSFODNN7EXAMPLE" | authy store aws-access-key-id
echo "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY" | authy store aws-secret-access-key

# Create a deploy-only policy
authy policy create ci-deploy \
  --allow "aws-*" \
  --allow "docker-registry-*" \
  --deny "aws-root-*"

# Run the deploy script with scoped secrets
authy run --scope ci-deploy --uppercase --replace-dash _ -- ./scripts/deploy.sh
```

### Pattern: Multi-Project Setup with `.authy.toml`

Different projects need different secrets. Use `.authy.toml` for per-project config:

```bash
# Frontend project
authy policy create frontend --allow "api-*" --allow "feature-flags-*"

# Backend project
authy policy create backend --allow "db-dev-*" --allow "cache-*" --allow "api-*" --deny "db-prod-*"
```

```toml
# ~/frontend/.authy.toml
[authy]
scope = "frontend"
uppercase = true
aliases = ["claude"]
```

```toml
# ~/backend/.authy.toml
[authy]
scope = "backend"
uppercase = true
replace_dash = "_"
aliases = ["claude", "aider"]
```

With the shell hook active (`eval "$(authy hook bash)"`), aliases activate automatically when you `cd` into each project:

```bash
cd ~/frontend && claude    # → authy run --scope frontend --uppercase -- claude
cd ~/backend && claude     # → authy run --scope backend --uppercase --replace-dash _ -- claude
```

Or without the shell hook, `--scope` is still inferred from `.authy.toml`:

```bash
cd ~/frontend && authy run -- npm run dev     # scope=frontend from .authy.toml
cd ~/backend && authy run -- cargo run        # scope=backend from .authy.toml
```

### Pattern: Shared Team Access

An admin provisions scoped, time-limited access for team members:

```bash
# Admin creates tokens for each developer
authy session create --scope frontend --ttl 8h --label "alice-frontend"
authy session create --scope backend --ttl 8h --label "bob-backend"

# Developers use their tokens (read-only, time-limited)
export AUTHY_TOKEN="authy_v1...."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy run --scope frontend -- npm run dev
```

When a developer leaves or a token is compromised:

```bash
authy session revoke abc123        # revoke one session
authy session revoke-all           # nuclear option
```

### Pattern: Wrapper Script for Any Tool

For any tool that doesn't support environment variables natively:

```bash
#!/bin/bash
# ~/.authy/wrap-mytool.sh
export MY_TOOL_API_KEY=$(authy get mytool-api-key)
export MY_TOOL_SECRET=$(authy get mytool-secret)
exec mytool "$@"
```

```bash
authy run --scope mytool-scope -- bash ~/.authy/wrap-mytool.sh --some-flag
```

---

## Policy Templates

### Development Agent (Restricted)

```bash
authy policy create dev-agent \
  --allow "api-*" \
  --allow "search-*" \
  --allow "github-token" \
  --deny "prod-*" \
  --deny "admin-*"
```

### Database Development

```bash
authy policy create db-dev \
  --allow "db-dev-*" \
  --allow "api-*" \
  --deny "db-prod-*" \
  --deny "db-admin-*"
```

### Full Access (Senior Engineer)

```bash
authy policy create senior-dev \
  --allow "*" \
  --deny "master-key" \
  --deny "recovery-codes-*"
```

### Read-Only Monitoring

```bash
authy policy create monitor \
  --allow "datadog-api-key" \
  --allow "grafana-token" \
  --allow "sentry-dsn"
```

---

## Troubleshooting

### Agent Can't Access a Secret

Test the policy:

```bash
authy policy test --scope my-scope secret-name
```

If denied, update the policy:

```bash
authy policy update my-scope --allow secret-name
```

### Session Token Expired

```
Error: Session expired
```

Create a new token:

```bash
authy session create --scope my-scope --ttl 8h
```

### Verifying What Gets Injected

List the environment variables a scope would inject:

```bash
# Preview with authy env (no subprocess needed)
authy env --scope my-scope --format shell --uppercase --replace-dash '_'

# Or via run:
authy run --scope my-scope --uppercase --replace-dash _ -- env | grep -E 'API|DB|TOKEN'
```

### Claude Code MCP Server Not Seeing Secrets

Common causes:
1. Claude Code was already running — restart it after setting up `authy run`
2. Wrong variable name in `.mcp.json` `env` — check the MCP server's documentation
3. Secrets are lowercase but the MCP server expects uppercase — use `--uppercase`
4. `${VAR}` expansion in `.mcp.json` requires the variable to exist in the parent environment at startup

### OpenClaw Ignoring Injected Environment

OpenClaw's shell-level environment variables override values in `openclaw.json`. If both are set, the environment variable wins. Remove the plaintext keys from `~/.openclaw/openclaw.json` after switching to Authy.

---

## Audit and Compliance

Every secret access is logged to `~/.authy/audit.log` with an HMAC chain:

```bash
# View recent access
authy audit show

# Verify no entries have been tampered with
authy audit verify

# Export for compliance
authy audit export > audit-$(date +%Y%m%d).json
```

Audit entries include: timestamp, operation, secret name, actor identity (master/keyfile/token), outcome (success/denied), and chain HMAC.

---

## Best Practices

1. **Use `--run-only` for agent tokens and policies** — agents should inject secrets via `authy run`, never read values directly
2. **Use the TUI for secret management** — secrets never touch shell history
3. **Use scoped policies** — never use `--allow "*"` unless necessary
4. **Prefer short-lived tokens** — use `--ttl` of hours, not days
5. **Rotate secrets regularly** — `authy rotate` to update values
6. **Monitor audit logs** — `authy audit show` to review access patterns
7. **Revoke immediately** — `authy session revoke-all` if an agent is compromised
8. **Block direct file access** — use `permissions.deny` in Claude Code settings to block `.env` and vault files
9. **Don't bake secrets into Docker images** — mount the vault at runtime

---

## Further Reading

- [Authy README](README.md) — Quick start and command reference
- [Authy Architecture](ARCHITECTURE.md) — System design and data flow
- [Claude Code MCP Docs](https://code.claude.com/docs/en/mcp) — MCP server configuration
- [Claude Code Settings](https://code.claude.com/docs/en/settings) — Settings hierarchy and permissions
- [OpenClaw Security Docs](https://docs.openclaw.ai/gateway/security) — OpenClaw security guidance
