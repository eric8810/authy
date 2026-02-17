# CLI Secrets Store & Dispatch — Existing Solutions Research

> Research date: 2026-02-16

---

## 1. Established CLI-Native Tools (Open Source, Local-First)

### pass (The Standard Unix Password Manager)
- **Repo**: https://www.passwordstore.org/
- **How it works**: Each secret is a GPG-encrypted file in `~/.password-store/`. Directory structure = secret hierarchy. Git for versioning/sync.
- **Strengths**: Dead simple, Unix philosophy (pipe-friendly), GPG ecosystem, extension system (since v1.7), community extensions for OTP, import/export, etc.
- **Weaknesses**: No native access control / scoping — anyone with the GPG key sees everything. No daemon mode. No audit log. No built-in team features. Average user finds GPG setup non-trivial. Had a SigSpoof vulnerability (2018).
- **Agent relevance**: Low. No concept of scoped sessions or restricted access per consumer. An agent with access to the GPG key can read all secrets.

### gopass
- **Repo**: https://github.com/gopasspw/gopass
- **How it works**: Go rewrite of `pass`, backward-compatible, adds team features. Multiple stores (personal, company, etc.), each backed by a separate Git repo. Recipients = GPG identities with per-store access.
- **Strengths**: Team-oriented — per-store recipient management, add/remove team members, auto-sync via Git push/pull. Supports `age` encryption as alternative to GPG. JSON/YAML-aware secret bodies. REPL mode. Binary secret support.
- **Weaknesses**: Scoping is at the *store* level, not per-secret fine-grained ACL. No session tokens or TTL. No daemon. No audit log beyond Git history.
- **Agent relevance**: Medium. You could put agent-accessible secrets in a dedicated store, but no runtime scoping or short-lived token model.

### SOPS (Secrets OPerationS)
- **Repo**: https://github.com/getsops/sops
- **How it works**: Encrypts *values* in structured files (YAML, JSON, ENV, INI) while keeping keys visible. Supports `age`, GPG, AWS KMS, GCP KMS, Azure Key Vault as encryption backends.
- **Strengths**: "Secrets as code" — encrypted files live in Git. Selective encryption (keys readable, values encrypted). Fine-scoped `.sops.yaml` rules per path/file pattern. Cloud KMS integration.
- **Weaknesses**: File-oriented, not a runtime secrets service. No access control model — if you can decrypt the file, you see all values in it. No sessions, no audit, no dispatch model.
- **Agent relevance**: Low for runtime dispatch. Good as an encrypted storage layer, but no scoped access.

### age (Actually Good Encryption)
- **Repo**: https://github.com/FiloSottile/age
- **How it works**: Simple encrypt/decrypt CLI. X25519 key pairs, SSH key support, no config. Created by Filippo Valsorda.
- **Strengths**: Minimal, composable, no config, copy-pastable keys. Great building block for higher-level tools.
- **Weaknesses**: It's a *primitive*, not a secrets manager. No store, no access control, no dispatch.
- **Agent relevance**: Excellent as a building block for a custom solution. Not a solution itself.

---

## 2. Developer-Focused SaaS / Hybrid Platforms

### 1Password CLI (`op`)
- **Docs**: https://developer.1password.com/docs/cli/
- **How it works**: `op run` injects secrets as env vars into a subprocess. `op inject` templates secrets into config files. `op read` fetches a single secret. Secrets referenced via `op://vault/item/field` URIs.
- **Strengths**: `op run` model is exactly the "dispatch without exposure" pattern — secrets exist only in the subprocess env and disappear on exit. Service Accounts support vault-level scoping (least privilege). Biometric unlock. MCP server integration announced for securing agent configs.
- **Weaknesses**: Proprietary / commercial. Requires 1Password account. Not self-hostable.
- **Agent relevance**: **High**. Service Accounts + `op run` is a working model for scoped agent access today. A Service Account can be restricted to specific vaults, so an agent only sees what it needs.

### Doppler
- **Docs**: https://docs.doppler.com/docs/cli
- **How it works**: `doppler run -- <command>` injects secrets as env vars. Secrets organized by project + environment (dev/staging/prod). Service Tokens restrict access to specific project+config combos.
- **Strengths**: Great DX. Project/environment hierarchy maps well to multi-agent setups. Ephemeral `.env` file mounting with auto-cleanup. Works across all languages/frameworks.
- **Weaknesses**: SaaS (cloud-hosted secrets). Service Tokens are long-lived (no TTL). No per-secret granularity within a config.
- **Agent relevance**: **Medium-High**. Service Tokens scoped to project+config = decent agent isolation. But no per-secret ACL or short-lived tokens.

### Infisical
- **Repo**: https://github.com/Infisical/infisical
- **How it works**: Open-source platform for secrets, certificates, and privileged access. CLI injects secrets into local dev and CI/CD. Self-hostable. RBAC, secret versioning, point-in-time recovery, secret rotation, dynamic credentials.
- **Strengths**: Open source (MIT for core). Self-hostable. RBAC with fine-grained access. Secret scanning (140+ types). Secret syncs to GitHub/Vercel/AWS. Dynamic secrets. Audit logging.
- **Weaknesses**: Heavier — requires server deployment (Docker/K8s). More than what's needed for a simple local CLI use case.
- **Agent relevance**: **High**. RBAC + dynamic secrets + audit logging checks most boxes. Self-hosting keeps secrets local. But it's a platform, not a lightweight CLI tool.

---

## 3. Enterprise / Infrastructure Tools

### HashiCorp Vault
- **Docs**: https://developer.hashicorp.com/vault
- **How it works**: Server-based secrets engine. Supports KV store, dynamic secrets (generates DB creds on demand), PKI, transit encryption. Accessed via CLI, API, or UI. Policy-based ACL with path patterns.
- **Strengths**: Industry standard. Fine-grained policies (path-based ACL). Dynamic secrets with automatic revocation. TTL on everything. Audit logging. Identity-based access. Now has AI agent identity patterns with dynamic secrets for AI workloads.
- **Weaknesses**: Heavy — requires running a server. Complex to operate. BSL license (no longer fully open source). Overkill for single-developer local use.
- **Agent relevance**: **Very High** (if you can justify the complexity). Vault's model of scoped tokens with TTL + dynamic secrets + audit is exactly the ideal for agent credential management.

### Akeyless
- **Docs**: https://www.akeyless.io/secrets-management/
- **How it works**: SaaS vault with "Zero Standing Privileges" — JIT dynamic secrets. Supports "secretless" agent pattern where agents authenticate via trusted identity (AWS IAM, GitHub JWT) and get ephemeral credentials.
- **Agent relevance**: **High** for cloud-native setups. The "secretless agent" pattern is interesting — agents never hold secrets at all.

---

## 4. The Agent Exposure Problem (2025-2026 Landscape)

### OWASP MCP Top 10 — Token Mismanagement (#1 risk)
- **Source**: https://owasp.org/www-project-mcp-top-10/
- 88% of MCP servers require credentials, but 53% use static long-lived secrets
- Developers embed secrets in config files, env vars, and prompt templates
- MCP's long-lived sessions mean tokens can leak through context persistence

### Process-Scoped Credentials (Emerging Pattern)
- **Source**: https://dreamiurg.net/2026/02/11/reducing-attack-surface-for-ai-agents-process-scoped-credentials.html
- Each AI agent process gets credentials scoped to exactly its needs
- Credentials are bound to process lifetime — die with the process
- Runtime-determined least privilege, not static config

### 1Password + MCP Server Securing
- **Source**: https://1password.com/blog/securing-mcp-servers-with-1password-stop-credential-exposure-in-your-agent
- Using `op run` to inject credentials into MCP server processes
- Secrets never stored in agent config files
- Agent only gets secrets for the duration of the MCP session

### HashiCorp Vault for AI Agent Identity
- **Source**: https://developer.hashicorp.com/validated-patterns/vault/ai-agent-identity-with-hashicorp-vault
- Dynamic secrets generated per-agent with automatic expiration
- Agent authenticates via workload identity, gets scoped token
- Full audit trail of what each agent accessed

---

## 5. Comparison Matrix

| Tool | Open Source | Self-Hosted | Scoped Access | TTL/Ephemeral | Audit Log | Agent-Ready | Complexity |
|---|---|---|---|---|---|---|---|
| **pass** | Yes | Local-only | No | No | Git only | Low | Minimal |
| **gopass** | Yes | Local+Git | Store-level | No | Git only | Medium | Low |
| **SOPS** | Yes | File-based | File-level | No | Git only | Low | Low |
| **1Password CLI** | No | No | Vault-level | Session | Yes | High | Low |
| **Doppler** | No | No | Project+Config | No | Yes | Medium-High | Low |
| **Infisical** | Partial | Yes | RBAC | Dynamic secrets | Yes | High | Medium |
| **Vault** | BSL | Yes | Path-policy ACL | Yes (all tokens) | Yes | Very High | High |
| **Akeyless** | No | Hybrid | JIT/Dynamic | Yes | Yes | High | Medium |

---

## 6. Gap Analysis — What's Missing

None of the existing tools are purpose-built for this exact use case: **a lightweight, local CLI vault that dispatches scoped secrets to AI agents**. The gap:

| Need | Best Existing Fit | Gap |
|---|---|---|
| Local encrypted storage | pass/gopass/SOPS + age | Solved |
| Per-secret scoped access for agents | Vault (policies), Infisical (RBAC) | These require running a server |
| Session tokens with TTL | Vault, 1Password Service Accounts | Vault is heavy; 1Password is proprietary |
| Pipe-based dispatch (no env/history leak) | 1Password `op run`, Doppler `doppler run` | Tied to their ecosystems |
| Audit log of agent access | Vault, Infisical, Doppler | Not available in lightweight CLI tools |
| Zero-config local daemon | Nothing | **Open gap** |
| Process-scoped credential binding | Emerging pattern, no standalone tool | **Open gap** |

---

## 7. Conclusions & Recommendations

### If we build `authy`, the sweet spot is:
1. **Storage**: Local encrypted file using `age` (simple keys, no GPG hassle)
2. **Dispatch**: `authy run --scope <agent-scope> -- <command>` (1Password-style subprocess injection)
3. **Scoping**: Policy file defining which scope can access which secrets (Vault-inspired but file-based, no server)
4. **Sessions**: Short-lived scoped tokens via Unix socket (lightweight daemon that holds the decrypted state)
5. **Audit**: Append-only local log file
6. **No server required**: Daemon is optional — can also work stateless with passphrase per invocation

### Existing tools to potentially build on (not reinvent):
- **age** — encryption primitive
- **SQLite** — local indexed secret store (encrypted at rest with age)
- **Unix domain sockets** — IPC for daemon mode

### Or just use an existing tool if scope fits:
- **Small team, no agents**: gopass is excellent
- **Cloud-native with budget**: Doppler or 1Password
- **Enterprise / many agents**: Vault or Infisical
- **Just need encrypted files in Git**: SOPS + age
