# 01 — Import from External Secret Stores

## Summary

Extend `authy import` to pull secrets from 1Password, `pass`, SOPS, and HashiCorp Vault. One command migrates an existing secrets workflow into authy.

## Motivation

Today `authy import` only reads `.env` files. Most teams migrating to authy already have secrets in a dedicated store. Asking them to manually `authy store` each secret is a non-starter. A direct import path removes the biggest adoption barrier.

## Current Behavior

```bash
authy import .env              # only source supported
authy import - < secrets.env   # stdin
```

## Proposed Behavior

```bash
# .env (existing)
authy import .env

# 1Password — via `op` CLI
authy import --from 1password --vault "Engineering"
authy import --from 1password --vault "Engineering" --tag "api-keys"

# pass — via password-store directory
authy import --from pass
authy import --from pass --path ~/.password-store

# SOPS — via sops CLI
authy import --from sops secrets.enc.yaml
authy import --from sops secrets.enc.json

# HashiCorp Vault — via `vault` CLI
authy import --from vault --path secret/data/myapp
authy import --from vault --path secret/data/myapp --mount kv
```

All sources support the existing flags: `--dry-run`, `--force`, `--prefix`, `--keep-names`.

## Interface

### CLI

```rust
/// Import secrets from external sources
Import {
    /// Source file (.env, SOPS encrypted file)
    /// Not required when --from is 1password, pass, or vault
    file: Option<String>,

    /// External source type
    #[arg(long, value_enum)]
    from: Option<ImportSource>,

    /// 1Password vault name (--from 1password)
    #[arg(long, alias = "op-vault")]
    vault: Option<String>,

    /// 1Password tag filter (--from 1password)
    #[arg(long)]
    tag: Option<String>,

    /// Path (pass store dir, or Vault secret path)
    #[arg(long)]
    path: Option<String>,

    /// HashiCorp Vault mount point (default: "secret")
    #[arg(long, default_value = "secret")]
    mount: String,

    // ... existing flags: --keep-names, --prefix, --force, --dry-run
}

#[derive(ValueEnum, Clone)]
enum ImportSource {
    Dotenv,
    #[value(name = "1password")]
    OnePassword,
    Pass,
    Sops,
    Vault,
}
```

When `--from` is omitted and a `file` argument is given, the behavior is identical to today (dotenv import).

### Source Adapters

Each source is implemented as a function that returns `Vec<(String, String)>` — the same format the existing dotenv importer produces. The shared import logic (dedup, force, prefix, audit) stays in `import.rs`.

```rust
// src/cli/import_sources/mod.rs
pub mod onepassword;
pub mod pass;
pub mod sops;
pub mod hcvault;

pub trait ImportAdapter {
    fn fetch(&self) -> Result<Vec<(String, String)>>;
}
```

## Source Details

### 1Password (`--from 1password`)

**Requires:** `op` CLI installed and authenticated (`op signin`).

```bash
# Under the hood:
op item list --vault "Engineering" --format json
op item get <id> --fields label=password --format json
```

- Lists items in the specified vault (or all vaults if `--vault` omitted)
- Extracts `password` or `credential` field from each item
- Item title becomes the secret name (transformed to lower-kebab by default)
- `--tag` filters items by 1Password tag

**Error handling:**
- `op` not found → error: "1Password CLI (`op`) not found. Install from https://1password.com/downloads/command-line/"
- Not signed in → error: "Not signed in to 1Password. Run `op signin` first."

### `pass` (`--from pass`)

**Requires:** `pass` CLI or the `~/.password-store/` directory.

```bash
# Under the hood: walk the password-store directory
# Each .gpg file = one secret
# File path (relative, minus .gpg) = secret name
find ~/.password-store -name "*.gpg" -type f
gpg --quiet --yes --batch --decrypt <file.gpg>
```

- Default path: `~/.password-store` (or `$PASSWORD_STORE_DIR`)
- Walks directory tree recursively
- File path becomes secret name: `subdir/api-key.gpg` → `subdir-api-key` (or `subdir/api-key` with `--keep-names`)
- Only the first line of each decrypted file is used as the value (pass convention)
- `--path` overrides the store directory

**Error handling:**
- `gpg` not found → error: "GPG not found. Install gnupg."
- Decryption failure → skip with warning, continue with remaining secrets

### SOPS (`--from sops`)

**Requires:** `sops` CLI installed.

```bash
# Under the hood:
sops --decrypt secrets.enc.yaml   # outputs plaintext YAML/JSON
```

- Runs `sops --decrypt <file>` to get plaintext
- Parses output as YAML or JSON (detected from file extension)
- Flattens nested keys with dot notation: `database.password` → `database-password` (or `database.password` with `--keep-names`)
- Supports `.yaml`, `.yml`, `.json` encrypted files

**Error handling:**
- `sops` not found → error: "SOPS CLI not found. Install from https://github.com/getsops/sops"
- Decryption failure → surface the sops error message

### HashiCorp Vault (`--from vault`)

**Requires:** `vault` CLI installed and authenticated (`vault login` or `VAULT_TOKEN`).

```bash
# Under the hood:
vault kv get -format=json -mount=secret secret/data/myapp
```

- Reads all key-value pairs at the specified path
- KV v2 (default): reads from `secret/data/<path>`
- `--mount` overrides the mount point (default: `secret`)
- Each key in the KV response becomes a secret name

**Error handling:**
- `vault` not found → error: "HashiCorp Vault CLI not found. Install from https://www.vaultproject.io/downloads"
- Not authenticated → error: "Not authenticated. Run `vault login` or set VAULT_TOKEN."
- Path not found → error with the path

## Name Transformation

All sources use the same transformation pipeline:

1. Raw name from source (e.g., `DATABASE_PASSWORD`, `subdir/api-key`, `My API Key`)
2. If `--keep-names`: use raw name as-is
3. Otherwise: normalize to `lower-kebab-case` (replace `_`, `/`, spaces, `.` with `-`, lowercase)
4. If `--prefix`: prepend prefix

## File Changes

| File | Change |
|------|--------|
| `src/cli/import.rs` | Extend with `--from` flag, dispatch to adapters |
| `src/cli/import_sources/mod.rs` | **Create** — adapter trait and module declarations |
| `src/cli/import_sources/onepassword.rs` | **Create** — 1Password `op` CLI adapter |
| `src/cli/import_sources/pass.rs` | **Create** — `pass`/password-store adapter |
| `src/cli/import_sources/sops.rs` | **Create** — SOPS decryption adapter |
| `src/cli/import_sources/hcvault.rs` | **Create** — HashiCorp Vault KV adapter |
| `src/cli/mod.rs` | Update `Import` variant with new args |

## Edge Cases

- External CLI not installed → actionable error with install link
- External CLI installed but not authenticated → actionable error with auth instructions
- Empty vault/path on external source → import 0 secrets, print message
- Secrets with binary values → skip with warning (authy stores UTF-8 strings)
- Very large imports (100+ secrets) → works, print count summary
- Name collisions after transformation → same `--force`/skip behavior as dotenv import
- `--dry-run` works with all sources — fetches secrets but doesn't store

## Acceptance Criteria

- [ ] `authy import --from 1password --vault X` imports secrets from 1Password
- [ ] `authy import --from pass` imports from password-store
- [ ] `authy import --from sops file.enc.yaml` imports from SOPS-encrypted file
- [ ] `authy import --from vault --path secret/myapp` imports from HashiCorp Vault
- [ ] `--dry-run` shows what would be imported without writing
- [ ] `--force` overwrites existing secrets
- [ ] `--prefix` and `--keep-names` work with all sources
- [ ] Missing CLI tool produces actionable error with install instructions
- [ ] Unauthenticated CLI produces actionable error with auth instructions
- [ ] Each import produces audit log entries
- [ ] Existing `.env` import behavior unchanged when `--from` is omitted
