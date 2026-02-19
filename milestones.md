# Milestones

## Vision

Authy becomes the secrets protocol for AI agents — the way every layer of the agent stack handles credentials. Not a tool people install, but infrastructure everything else assumes. Like `git` for version control, `docker` for containers, `.env` for config — `authy` for agent secrets.

## Why CLI-First

AI agents are converging on CLI/bash as the universal execution interface. Claude Code, Goose, Aider, Codex CLI — they all call tools via shell commands. The evidence:

- CLI uses 98.7% fewer tokens than MCP for equivalent operations
- AGENTS.md (60K+ repos, Linux Foundation) declares tools to agents via convention files
- Models know CLI tools from training data — zero registration, zero schema overhead
- "The best agent interface was never a new protocol — it was the one every tool already speaks"

Authy is CLI-first because CLI is the protocol agents already speak.

## User Segments

### Segment 1: Software developers using AI coding tools
- ~20-25M users (Claude Code, Cursor, Aider, Copilot, Windsurf)
- Have `.env` files with DB URLs, API keys, cloud credentials
- Pain: AI tools auto-read `.env` files, secrets leak into LLM context
- Reach them through: shell hooks, Claude Code skills, AGENTS.md, shell aliases — invisible integration

### Segment 2: Agent builders (PRIMARY TARGET)
- ~500K-1.5M users, growing 143% YoY
- Build with LangChain, CrewAI, AutoGen, OpenAI Agents SDK, Anthropic Agent SDK
- 10-50+ secrets per project, no per-agent scoping exists today
- 45% of teams use shared API keys across agents
- Pain: highest intensity — no tool scopes credentials per agent without a server
- Reach them through: `authy run` per-agent scoping, agent platform skills, tutorials

### Segment 3: Agent operators
- ~5-10M users (OpenClaw, n8n, Zapier, Make)
- Paste 1-5 API keys into config files, low technical sophistication
- Won't install CLI tools directly
- Reach them through: platform integrations (OpenClaw skill, n8n node) — they use Authy without knowing

## Agent Stack Layers

Each layer of the agent stack needs Authy differently:

| Layer | Examples | How they use Authy |
|-------|----------|-------------------|
| Framework | LangChain, LlamaIndex | `authy get` injects keys before framework init |
| Orchestrator | CrewAI, LangGraph, Agents SDK | `authy run --scope agent-a` isolates per-agent credentials |
| Tools | MCP servers, skills, function tools | `authy run --scope tool-x` replaces hardcoded env vars |
| Runtime | E2B, Modal, Docker | `authy env --scope sandbox-1` piped into container creation |
| Platform | OpenClaw, Replit, Warp | Integrate Authy as their secrets backend |

The cross-cutting problem: every layer independently reads `os.environ` with no coordination, no scoping, no rotation, no audit. Authy becomes the standard handoff protocol between layers.

---

## v0.2 — Agent-Native CLI ✓

Make Authy callable by any agent that can run bash. Machine-friendly output, non-interactive operation, structured data.

- [x] `authy get <name> --json` — structured JSON output for machine consumption
- [x] `authy env --scope <scope>` — export scoped secrets as env vars (shell-sourceable)
- [x] `authy import .env` — one-command migration from dotenv files
- [x] `authy export --format env` — export vault as `.env` format for backward compat
- [x] `--no-interactive` flag on all commands — agents don't do prompts
- [x] Machine-parseable error output (exit codes, JSON errors)
- [x] Run-only enforcement — `--run-only` on tokens and policies blocks `get`/`env`/`export`
- [x] AGENTS.md snippet — declares Authy to any agent that reads convention files
- [x] CLAUDE.md snippet — teaches Claude Code how to call `authy get` / `authy run`

**Success criteria:** Any AI agent that can execute bash can use Authy to retrieve scoped secrets without human interaction.

## v0.3 — Deep Plant Into Agent Platforms

Embed Authy into the places developers already are. Distribution, not features.

- [x] **Claude Code skill** — `.claude/skills/authy/SKILL.md` teaching Claude to use Authy
- [x] **OpenClaw skill** — publish to ClawHub (published as authy@0.2.2)
- [x] **Agent Skills standard** package — cross-platform skill at agentskills.io (npx skills add eric8810/authy)
- [x] **Shell hook** — `eval "$(authy hook bash/zsh/fish)"` — direnv-style auto-activation per directory
- [x] `.authy.toml` project marker file — scope + vault config per project directory
- [x] **Shell alias generator** — `authy alias claude-code` outputs alias for wrapping commands with `authy run`
- [ ] Docs and tutorials written as training data: "How to manage AI agent secrets"
- [ ] Blog posts, examples, and guides that LLMs will train on

**Success criteria:** A developer setting up Claude Code, OpenClaw, or a new agent project encounters Authy as the default secrets pattern.

## v0.4 — File-Layer Secrets

`authy run` covers env vars. `authy resolve` covers config files. Together they handle both surfaces where secrets live.

- [ ] **`authy resolve <file>`** — replace `<authy:key-name>` placeholders with real values from vault, output to `--output` path or stdout
- [ ] **Placeholder format** — `<authy:key-name>` in any config file (yaml, json, toml, etc.), safe to commit and share
- [ ] **Safe/sensitive command split** — formalize: safe commands (list, run, resolve) work with agent tokens; sensitive commands (get, store, export, import, rotate) require TTY or master key
- [ ] **`authy rekey`** — change passphrase or switch between passphrase/keyfile auth

**Success criteria:** Secrets in config files use placeholders. `authy resolve` produces real files at deploy/launch time. Agents only see placeholder files.

### Deferred to v0.5+

- `authy up` / `authy down` (tool launcher — agent platforms already handle process management)
- Agent identity (named agents with scoped access)
- Per-agent audit attribution
- Delegation tokens (agent-to-agent scope narrowing)

## v0.5 — Platform Integration Layer

Serve segment 3 (operators) through the platforms they already use. Serve platforms that need a service interface, not just CLI.

- [ ] `authy serve --mcp` — MCP server mode (stdio + Streamable HTTP)
- [ ] MCP tools: `get_secret`, `list_secrets`, `test_policy`
- [ ] Publish to official MCP Registry + Smithery
- [ ] REST/HTTP endpoint mode for platforms that need an API
- [ ] Multi-vault support (`--vault path`) — per-project isolation
- [ ] Secret namespaces (`prod/db-url`, `dev/db-url`)
- [ ] n8n community node (reaches operators through their platform)

**Success criteria:** Platforms can integrate Authy as their secrets backend. Operators use Authy through platforms without knowing it.

## v0.6 — Breach Response & Security Hardening

When agents get compromised (not if — when), Authy is the incident response tool.

- [ ] Audit alerting — detect suspicious access patterns (bulk reads, unusual scopes)
- [ ] Secret rotation workflow — `authy rotate` with hooks for downstream notification
- [ ] Webhook/hook notifications on sensitive secret access
- [ ] Multiple age recipients — multi-identity decryption for team key management
- [ ] Import from 1Password, pass, SOPS — broader migration paths

**Success criteria:** When an agent is compromised, the operator knows exactly what was accessed, revokes in one command, and rotates affected secrets.

## v1.0 — The Protocol

Authy is on every agent's PATH like `git` is on every developer's PATH.

- [ ] Stable CLI interface — semver guarantee, output formats are API contracts
- [ ] Daemon mode with auto-lock — keep vault unlocked in memory, lock after timeout
- [ ] `lib.rs` extraction — make core modules public for Rust crate consumers
- [ ] Publish to crates.io (lib + CLI split) via Trusted Publishing
- [ ] Comprehensive unit test coverage
- [ ] Vault format versioning + migration for future-proofing
- [ ] cargo-dist for release automation (Homebrew tap, shell/PS installers, cargo-binstall)

**Success criteria:** `authy` is the assumed default for agent secrets. The verb, not a tool.

---

## Key Metrics (How We Know It's Working)

- GitHub stars and npm weekly downloads
- Number of agent platforms with native Authy integration
- Mentions in AGENTS.md / CLAUDE.md files across public repos
- Appearance in LLM training data (can an agent use Authy without being told?)
- Number of `.authy.toml` files in public repositories
- Community skills/plugins built on top of Authy

## Competitive Landscape

| Tool | What it does | Authy's advantage |
|------|-------------|-------------------|
| **vestauth** | Agent identity via Ed25519 HTTP signatures (RFC 9421). Agents prove who they are, tools verify. | Requires every tool to add verification code — won't scale. Authy uses process isolation (zero tool changes). No secret storage — relies on dotenvx. Needs hosted key directory. |
| **agent-vault** | File I/O redaction layer. Agents see `<agent-vault:key>` placeholders, real values restored on write. | File-only — doesn't cover runtime/env secrets. No scoping, no policies, no audit log. AES-256-GCM vs age. TypeScript/npm vs Rust single binary. We adopt their file redaction idea in v0.4 (`authy read`/`authy write`). |
| **dotenvx** | Encrypted `.env` files with `dotenvx run`. | No agent scoping, no policies, no session tokens, no audit. Secrets still flow into agent's env. |
| **.env files** | The status quo. Plain text secrets in project directories. | Everything. But this is what 90% of developers use today. |

**Our position:** Authy covers both surfaces (runtime + file) with scoping, policies, audit, and process isolation. No other tool does all of these.

## Anti-Goals (What We Don't Build)

- GUI app — the TUI is sufficient for human interaction
- Cloud sync or hosted service — "no server, no accounts" is the value prop
- Secret generation — use dedicated tools (openssl rand, etc.)
- PKI / certificate management — different problem domain
- Multi-user RBAC — keep it single-operator; policies + tokens handle delegation
- Python/TypeScript SDK (yet) — CLI is the universal interface; SDKs are wrappers that come later if needed
