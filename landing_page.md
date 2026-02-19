# Authy

**Secrets Store & Dispatch for AI Agents**

Your agents need API keys. They don't need the keys to the kingdom.

Encrypted vault. Scoped policies. Short-lived tokens. Full audit trail. Single binary, no server, no accounts.

[Get Started](#quick-start) · [View on GitHub](https://github.com/eric8810/authy) · [Read the Docs](#documentation)

---

## The Problem

AI agents run with your secrets — but today's tools give you no control over what they can see.

- **Claude Code** reads `.env` files silently. Infostealer malware targets these files.
- **OpenClaw** stores API keys in plaintext JSON. Over 30,000 instances found exposed online.
- **MCP servers** require secrets hardcoded in config files that get committed to repos.
- **CI/CD pipelines** inject secrets as env vars with no scoping — every step sees everything.

Existing secrets tools don't solve the agent problem:

| Tool | Limitation |
|------|------------|
| **pass/gopass** | No scoped access — GPG key = full access to everything |
| **HashiCorp Vault** | Requires running a server; overkill for local and CI use |
| **1Password CLI** | Proprietary, requires an account |
| **SOPS** | File encryption only, not a runtime dispatch tool |

---

## How Authy Works

```
Admin (TUI or CLI)          Agent
      │                       │
      ├─ store secrets ───►   │
      ├─ create policy ───►   │
      ├─ create token ────►   │
      │                       ├─ authy run --scope X -- agent
      │                       │    ├─ decrypt vault
      │                       │    ├─ filter by policy
      │                       │    ├─ inject as env vars
      │                       │    └─ spawn child process
      │                       │
      └─ audit show ──────►   └─ every access logged
```

1. **Store** secrets in an encrypted vault (never in shell history, config files, or process args)
2. **Scope** access with glob-based allow/deny policies per agent
3. **Dispatch** secrets into child processes as environment variables
4. **Audit** every access with a tamper-evident HMAC-chained log

---

## Features

| Feature | Description |
|---------|-------------|
| **Encrypted Vault** | `age`-encrypted single file; passphrase or X25519 keyfile auth |
| **Scoped Policies** | Glob-based allow/deny rules; deny overrides allow; default deny |
| **Run-Only Mode** | Restrict agents to subprocess injection only — `get`, `env`, `export` blocked |
| **Session Tokens** | Short-lived, HMAC-validated, run-only capable; `authy_v1.` prefix for leak detection |
| **Subprocess Injection** | `authy run` injects secrets as env vars into the child process only |
| **Config File Resolution** | `authy resolve` replaces `<authy:key>` placeholders in any config file |
| **Vault Rekey** | `authy rekey` re-encrypts the vault with new credentials; switch between passphrase and keyfile |
| **JSON Output** | `--json` on all read commands; structured errors to stderr |
| **Audit Log** | Append-only JSONL with HMAC chain; actor, timestamp, outcome on every access |
| **Admin TUI** | Full-screen terminal UI for secrets, policies, sessions, and audit — nothing touches shell history |
| **Headless Mode** | CI/CD friendly; non-interactive with fail-fast; keyfile + token auth |

---

## Quick Start

### Install

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
git clone https://github.com/eric8810/authy.git
cd authy && cargo build --release
cp target/release/authy /usr/local/bin/
```

### Initialize

```bash
authy init --generate-keyfile ~/.authy/keys/master.key
```

### Manage Secrets (TUI)

```bash
# Using keyfile (no prompt)
authy admin --keyfile ~/.authy/keys/master.key

# Using passphrase (interactive prompt)
authy admin
```

The TUI will prompt for your passphrase if no keyfile is provided. It provides masked input fields for storing secrets, a policy editor with live testing, session token management, and a scrollable audit log with HMAC verification. Press `?` for key bindings.

### Manage Secrets (CLI)

**推荐：使用 TUI（最直观安全）**

```bash
authy admin --keyfile ~/.authy/keys/master.key
# 或使用 passphrase 启动
authy admin
# 在 TUI 中选择 Secrets → Add，输入密钥名和值（输入被掩码保护）
```

**或使用命令行：**

```bash
# 通过提示输入（安全 - 不会进入 shell history）
authy store anthropic-api-key
# 输入密钥值后按 Ctrl+D

# 或从文件管道输入
authy store anthropic-api-key < ~/.secrets/anthropic-key

authy list
```

### Create a Policy and Launch an Agent

```bash
# Define what the agent can see — run-only means it can never read values directly
authy policy create claude-code \
  --allow "anthropic-*" \
  --allow "github-*" \
  --deny "prod-*" \
  --run-only

# Launch with scoped secrets injected
authy run --scope claude-code --uppercase --replace-dash _ -- claude
# Claude Code sees ANTHROPIC_API_KEY and GITHUB_TOKEN — nothing else
# authy get / authy env / authy export are blocked
```

---

## Use Cases

### Config File Templates

```bash
# Template with placeholders (safe to commit)
cat config.yaml.tpl
# host: <authy:db-host>
# port: <authy:db-port>
# api_key: <authy:api-key>

# Resolve at deploy time
authy resolve config.yaml.tpl --scope deploy --output config.yaml
```

### AI Agents

```bash
# Claude Code — run-only policy, secrets injected, agent can't read them
authy policy create claude-code --allow "anthropic-*" --allow "github-*" --run-only
authy run --scope claude-code --uppercase --replace-dash _ -- claude

# OpenClaw — replaces plaintext ~/.openclaw/openclaw.json
authy policy create openclaw --allow "anthropic-*" --allow "discord-*" --run-only
authy run --scope openclaw --uppercase --replace-dash _ -- openclaw

# Any agent that reads env vars
authy run --scope my-agent --uppercase -- ./my-agent.sh
```

### MCP Servers

Keep secrets out of `.mcp.json` using environment variable expansion:

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/mcp-server-github"],
      "env": { "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}" }
    }
  }
}
```

Launch Claude Code with `authy run` so `${GITHUB_TOKEN}` resolves from the vault.

For MCP servers that need secrets in args (like Postgres), use wrapper scripts:

```json
{
  "mcpServers": {
    "postgres": {
      "command": "authy",
      "args": ["run", "--scope", "claude-db", "--", "bash", "~/.authy/mcp-postgres.sh"]
    }
  }
}
```

### Project Config + Shell Hook

Drop `.authy.toml` in your project root for automatic activation:

```toml
# .authy.toml
[authy]
scope = "claude-code"
keyfile = "~/.authy/keys/master.key"
uppercase = true
replace_dash = "_"
aliases = ["claude", "aider"]
```

```bash
# Add to ~/.bashrc or ~/.zshrc (one-time)
eval "$(authy hook bash)"
```

Now `cd` into the project and type `claude` — secrets are injected automatically.

### Shell Alias (Manual)

Or generate aliases explicitly:

```bash
eval "$(authy alias --from-project)"
# Or: eval "$(authy alias claude-code claude aider)"
```

### CI/CD

```yaml
- name: Deploy
  env:
    AUTHY_KEYFILE: ${{ secrets.AUTHY_KEYFILE }}
  run: authy run --scope ci-deploy --uppercase --replace-dash _ -- ./deploy.sh
```

### Team Sharing

```bash
# Admin creates a run-only, time-limited token
authy session create --scope dev --ttl 8h --label "alice-dev" --run-only

# Developer uses the token — can inject secrets, cannot read values
export AUTHY_TOKEN="authy_v1.eyJ..."
export AUTHY_KEYFILE=~/.authy/keys/master.key
authy run --scope dev --uppercase -- npm run dev

# Revoke instantly if compromised
authy session revoke-all
```

### Docker

```bash
docker run \
  -v ~/.authy:/root/.authy:ro \
  -e AUTHY_KEYFILE=/root/.authy/keys/master.key \
  --entrypoint authy \
  my-agent-image \
  run --scope my-agent --uppercase --replace-dash _ -- agent-binary
```

[AI Agent Integration Guide](AI_AGENT_GUIDE.md) — Full integration docs for Claude Code, OpenClaw, MCP servers, and more.

---

## Core Commands

| Command | Description |
|---------|-------------|
| `authy init` | Initialize vault with passphrase or keyfile |
| `authy admin` | Launch interactive TUI |
| `authy store <name>` | Store a secret (reads from stdin) |
| `authy get <name>` | Retrieve a secret value (blocked in run-only mode) |
| `authy list [--json]` | List secret names (allowed in run-only mode) |
| `authy remove <name>` | Delete a secret |
| `authy rotate <name>` | Update a secret value |
| `authy run [--scope <s>] -- <cmd>` | Run command with scoped secrets injected |
| `authy resolve <file>` | Resolve `<authy:key>` placeholders in config files |
| `authy rekey` | Re-encrypt vault with new credentials |
| `authy env [--scope <s>]` | Output secrets as env vars (blocked in run-only mode) |
| `authy import <file>` | Import secrets from .env file |
| `authy export --format <f>` | Export secrets (blocked in run-only mode) |
| `authy policy create <name> [--run-only]` | Create an access policy |
| `authy policy test --scope <s> <name>` | Test if a policy allows access |
| `authy session create --scope <s> [--run-only]` | Create a scoped, time-limited token |
| `authy session revoke <id>` | Revoke a session token |
| `authy audit show [--json]` | View audit log |
| `authy project-info` | Show .authy.toml project config |
| `authy alias` | Generate shell aliases for tools |
| `authy hook <shell>` | Output shell hook code for auto-activation |
| `authy audit verify` | Verify audit log integrity |

---

## Comparison

| | Authy | pass | Vault | 1Password CLI |
|---|:---:|:---:|:---:|:---:|
| Single binary | Y | Y | N | Y |
| No server required | Y | Y | N | Y |
| No account required | Y | Y | Y | N |
| Open source | Y | Y | Y | N |
| Scoped policies | Y | N | Y | Y |
| Run-only mode | Y | N | N | N |
| Short-lived tokens | Y | N | Y | Y |
| Subprocess injection | Y | N | N | Y |
| JSON output | Y | N | Y | Y |
| Tamper-evident audit | Y | N | Y | Y |
| Built for agents | Y | N | N | N |

---

## Security

- **Encryption** — `age` with X25519 (keyfile) or scrypt (passphrase); single encrypted vault file
- **Secret hygiene** — Never in shell history, process argv, or parent env; zeroized on drop
- **Access control** — Policies inside the encrypted vault; deny overrides allow; default deny
- **Run-only enforcement** — Agents can only inject secrets via subprocess; direct value access blocked
- **Token model** — HMAC-SHA256 with constant-time comparison; only the HMAC is stored, not the token
- **Audit trail** — HMAC-chained JSONL; actor identity, timestamp, operation, outcome on every entry
- **Revocation** — Instant token revocation; `authy_v1.` prefix for automated leak scanning

---

## Documentation

- [README](README.md) — Quick start and command reference
- [Architecture](ARCHITECTURE.md) — System design and data flow
- [AI Agent Guide](AI_AGENT_GUIDE.md) — Claude Code, OpenClaw, MCP servers, CI/CD, Docker

---

Built with Rust. Secured with age.
