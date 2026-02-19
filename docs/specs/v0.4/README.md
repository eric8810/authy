# v0.4 — File-Layer Secrets

`authy run` covers env vars. `authy resolve` covers config files. Together they handle both surfaces where secrets live.

## Specs

| # | Spec | Status |
|---|------|--------|
| 01 | [File placeholders + `authy resolve`](01-file-placeholders.md) | todo |
| 02 | [Safe/sensitive command split](02-safe-sensitive-split.md) | todo |
| 03 | [Passphrase change / re-key vault](03-rekey-vault.md) | todo |

## Build Order

All three specs are independent. Can be built in parallel.
