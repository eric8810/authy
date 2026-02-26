# v0.7.1 — Native Bindings & Unified Naming

Replace subprocess-wrapper SDKs with native Rust bindings. Ship PyO3 wheels for Python and napi-rs binaries for Node.js — users get the vault engine compiled into the language package, no separate authy binary needed. Unify all package names to `authy-cli`.

## Specs

| # | Spec | Status |
|---|------|--------|
| 01 | [Rust core: `build_env_map()`](01-rust-core.md) | todo |
| 02 | [Python native binding (PyO3)](02-python-binding.md) | todo |
| 03 | [Node.js native binding (napi-rs)](03-node-binding.md) | todo |

## Build Order

```
Phase 1: 01 (Rust core)          — build_env_map() method on AuthyClient
Phase 2: Cargo workspace         — root workspace with bindings members
Phase 3: 02, 03 (bindings)       — parallel, each is a cdylib crate
Phase 4: Go SDK rename           — docs-only change
Phase 5: Tests + CI              — depends on all
Phase 6: Docs + version bump     — depends on all
```

Specs 02 and 03 are independent of each other and can be built in parallel. The Rust core change (01) must land first as both bindings depend on `build_env_map()`.

## Deferred (from v0.7)

- `authy import --from 1password` (external importers)
- `authy import --from pass` (password-store)
- `authy import --from sops` (SOPS)
- `authy import --from vault` (HashiCorp Vault)
