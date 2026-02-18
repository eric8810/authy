# 07 — Agent Skills

## Summary

Ship Authy as installable skills for agent platforms. Skills are the mechanism agent platforms use to discover and invoke tools — Authy publishes skills so agents know it exists and how to call it.

AGENTS.md and CLAUDE.md are user/project-owned files. We don't generate or edit them programmatically. Skills are what Authy ships and controls.

## Motivation

An agent that can run bash already has everything it needs to call `authy get`, `authy run`, `authy env`. But it doesn't know Authy exists unless the platform tells it. Skills are how agent platforms load tool knowledge into the agent's context:

- **Agent Skills standard** (agentskills.io) — Cross-platform skill format adopted by Claude Code, Cursor, Microsoft, OpenAI, Atlassian, GitHub.
- **OpenClaw skills** — SKILL.md format loaded into agent system prompt. 5,700+ skills on ClawHub.
- **Claude Code skills** — `.claude/skills/` directory, loaded on demand.

Each platform has a slightly different format, but the core is the same: a markdown file that teaches the agent what tools exist and when/how to use them.

## Deliverables

### 1. Claude Code Skill

File: `skills/claude-code/SKILL.md`

Published to the Claude Code skill ecosystem. Users install by copying to `~/.claude/skills/authy/` or their project's `.claude/skills/authy/`.

```markdown
---
name: authy
description: Encrypted secrets management for AI agents
user-invocable: false
---

# Authy — Secrets Management

Use `authy` to retrieve secrets from an encrypted vault. Never read `.env` files directly.

## When to use

- When you need an API key, database URL, token, or any credential
- When launching a subprocess that needs secrets in its environment
- When the project CLAUDE.md or AGENTS.md mentions authy

## Retrieving a secret

```bash
authy get <name>
```

Outputs the raw secret value to stdout. Errors go to stderr.

With metadata:

```bash
authy get <name> --json
```

## Listing secrets

```bash
authy list --scope <scope>
```

## Injecting secrets into a command

```bash
authy run --scope <scope> --uppercase --replace-dash _ -- <command>
```

Only secrets allowed by the scope policy are injected. The child process sees them as environment variables.

## Sourcing secrets into the shell

```bash
eval "$(authy env --scope <scope> --uppercase --replace-dash _)"
```

## Rules

- Never read `.env`, `.env.*`, or `~/.authy/vault.age` directly
- Never put secret values in CLI arguments or echo them
- Auth is via `AUTHY_KEYFILE` or `AUTHY_TOKEN` env var — never prompt for passphrases
- Stdout = secret data, stderr = diagnostics
```

### 2. OpenClaw Skill

File: `skills/openclaw/SKILL.md`

Published to ClawHub. Addresses OpenClaw GitHub issue #7916 (encrypted secrets management).

```markdown
---
name: authy
description: Encrypted secrets vault — replaces plaintext .env and openclaw.json credentials
homepage: https://github.com/eric8810/authy
user-invocable: true
command-dispatch: tool
command-tool: bash
command-arg-mode: raw
metadata:
  openclaw:
    requires:
      bins:
        - authy
      env:
        - AUTHY_KEYFILE
---

# Authy — Secrets Management

Use `authy` to retrieve secrets from an encrypted vault instead of reading plaintext `.env` files or `openclaw.json` credentials.

## Retrieving a secret

When you need an API key or credential:

```bash
authy get <name>
```

## Listing available secrets

```bash
authy list
# Or filtered by scope:
authy list --scope <scope>
```

## Running a command with secrets

To launch a process with scoped secrets injected as environment variables:

```bash
authy run --scope <scope> --uppercase --replace-dash _ -- <command>
```

## Important

- Never read `~/.openclaw/.env` or `openclaw.json` for credentials
- Never put secret values in command arguments
- Auth is via `AUTHY_KEYFILE` environment variable
- API keys should never pass through the LLM context window — use `authy run` to inject them directly into processes
```

### 3. Agent Skills Standard Package

File: `skills/agent-skills/SKILL.md`

For the cross-platform Agent Skills standard (agentskills.io). Same content as the Claude Code skill with standard frontmatter.

```markdown
---
name: authy
description: Encrypted secrets management for AI agents
version: 0.2.0
homepage: https://github.com/eric8810/authy
tags: [secrets, security, credentials, vault, encryption]
requires:
  bins: [authy]
---

# Authy — Secrets Management

Use `authy` to retrieve secrets from an encrypted vault. Never read `.env` files directly.

## Commands

| Command | Purpose |
|---------|---------|
| `authy get <name>` | Retrieve a secret value (stdout) |
| `authy get <name> --json` | Retrieve with metadata |
| `authy list [--scope <scope>]` | List secret names |
| `authy env --scope <scope>` | Export scoped secrets as shell env vars |
| `authy run --scope <scope> -- <cmd>` | Run command with scoped secrets injected |

## Rules

- Never read `.env` files or vault files directly
- Never put secret values in CLI arguments
- Auth via `AUTHY_KEYFILE` or `AUTHY_TOKEN` env var
```

## Directory Structure

```
skills/
  claude-code/
    SKILL.md
  openclaw/
    SKILL.md
  agent-skills/
    SKILL.md
```

Shipped in the Authy repository. Each can be installed independently.

## Design Considerations

### Skills are read-only instructions

Skills teach the agent what commands exist and when to use them. They don't execute code themselves — the agent reads the skill, then decides to call `authy get` via bash. No wrappers, no scripts, no runtime dependencies beyond the `authy` binary being on PATH.

### Agent-facing commands only

Skills document only what an agent should call:

| Include | Exclude |
|---------|---------|
| `authy get` | `authy store` |
| `authy list` | `authy remove` |
| `authy run` | `authy rotate` |
| `authy env` | `authy policy *` |
| | `authy session *` |
| | `authy admin` |
| | `authy init` |
| | `authy import` |
| | `authy export` |

Admin commands are for humans. Agent commands are for agents. Skills only document agent commands.

### Keep skills minimal

Agent context is expensive. Each skill should be under 500 tokens. Focus on the 4 core commands (`get`, `list`, `run`, `env`) and the 3 rules (no .env, no argv secrets, auth via env vars).

### OpenClaw gating

The OpenClaw skill uses `requires.bins: [authy]` and `requires.env: [AUTHY_KEYFILE]` so it only activates when Authy is installed and configured. Agents without Authy never see the skill.

## Acceptance Criteria

- [ ] Claude Code skill at `skills/claude-code/SKILL.md` — valid format, installable to `~/.claude/skills/`
- [ ] OpenClaw skill at `skills/openclaw/SKILL.md` — valid ClawHub format with proper frontmatter and gating
- [ ] Agent Skills standard package at `skills/agent-skills/SKILL.md`
- [ ] Each skill is under 500 tokens
- [ ] Skills only document agent-facing commands (get, list, run, env)
- [ ] Skills include auth instructions and "no .env" rules
- [ ] README updated to reference skills and installation instructions
