# v0.6 — Platform Integration Layer

Expose the AuthyClient API via MCP (Model Context Protocol) so AI agent platforms can call authy natively over stdio JSON-RPC instead of shelling out. Also ships two deferred TUI polish items: clipboard copy and vault change detection.

## Specs

| # | Spec | Status |
|---|------|--------|
| 01 | [AuthyClient API extensions](01-api-extensions.md) | todo |
| 02 | [MCP server protocol](02-mcp-protocol.md) | todo |
| 03 | [CLI `serve` command](03-cli-serve.md) | todo |
| 04 | [TUI clipboard copy (OSC 52)](04-tui-clipboard.md) | todo |
| 05 | [TUI vault change detection](05-tui-vault-change.md) | todo |

## Build Order

```
Phase 1: 01 (API extensions)         — no deps
Phase 2: 02 (MCP protocol)           — depends on 01
Phase 3: 03 (CLI serve)              — depends on 02
Phase 4: 04 (TUI clipboard)          — no deps, parallel with 1-3
Phase 5: 05 (TUI vault change)       — no deps, parallel with 1-3
Phase 6: Tests (in todo.md)          — depends on 01-03
Phase 7: Docs + version bump         — depends on all
```

Specs 04 and 05 are fully independent of 01-03. They can be built in parallel with the MCP work.

## Deferred

- REST/HTTP endpoint mode
- Multi-vault support (`--vault path`)
- Secret namespaces (`prod/db-url`)
- n8n community node
- MCP Registry / Smithery publishing
