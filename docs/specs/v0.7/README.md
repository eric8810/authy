# v0.7 — Language Wrappers & Migration

Lower adoption friction by meeting developers in their language. Ship thin SDK wrappers for Python, TypeScript, and Go that provide native API access, plus importers for the major secret stores people are migrating from.

## Specs

| # | Spec | Status |
|---|------|--------|
| 01 | [Import from external stores](01-import-external.md) | todo |
| 02 | [Python SDK](02-python-sdk.md) | todo |
| 03 | [TypeScript SDK](03-typescript-sdk.md) | todo |
| 04 | [Go SDK](04-go-sdk.md) | todo |

## Build Order

```
Phase 1: 01 (import)              — no deps, pure Rust CLI work
Phase 2: 02, 03, 04 (SDKs)       — parallel, each is an independent package
Phase 3: Tests + CI               — depends on all
Phase 4: Docs + version bump      — depends on all
```

All three SDKs are independent of each other and can be built in parallel. The import spec is pure CLI work and can also be built in parallel with the SDKs.

## Deferred (from v0.6)

- REST/HTTP endpoint mode
- Multi-vault support (`--vault path`)
- Secret namespaces (`prod/db-url`)
- n8n community node
- MCP Registry / Smithery publishing
