# v0.2 Specs — Agent-Native CLI

> Make Authy callable by any AI agent that can run bash.

## Goal

Any AI agent that can execute shell commands can use Authy to retrieve scoped secrets without human interaction. Machine-friendly output, non-interactive operation, structured data.

## Feature Specs

| Spec | Feature | Status |
|------|---------|--------|
| [01-json-output.md](01-json-output.md) | `--json` flag for structured machine output | Planned |
| [02-env-command.md](02-env-command.md) | `authy env` — export scoped secrets as shell env vars | Planned |
| [03-import-dotenv.md](03-import-dotenv.md) | `authy import` — migrate from `.env` files | Planned |
| [04-export-env.md](04-export-env.md) | `authy export --format env` — export as `.env` format | Planned |
| [05-non-interactive.md](05-non-interactive.md) | Non-interactive mode for headless/agent operation | Planned |
| [06-structured-errors.md](06-structured-errors.md) | Typed exit codes and JSON error output | Planned |
| [07-agent-discovery.md](07-agent-discovery.md) | Agent skills (Claude Code, OpenClaw, Agent Skills standard) | Planned |

## Design Principles

1. **Stdout is data, stderr is diagnostics.** This convention already exists — v0.2 makes it strict.
2. **No new dependencies unless necessary.** Prefer extending existing code over adding crates.
3. **Backward compatible.** All existing commands behave identically without new flags.
4. **Agent-first, human-compatible.** New features optimize for machine consumption but remain usable by humans.
