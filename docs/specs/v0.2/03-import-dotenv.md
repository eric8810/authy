# 03 — Import from `.env` (`authy import`)

## Summary

New `authy import` command that reads a `.env` file and stores each key-value pair as a secret in the vault. One command to migrate from plaintext to encrypted.

## Motivation

Every developer has `.env` files. Every agent framework tutorial starts with `touch .env && echo "OPENAI_API_KEY=sk-..." >> .env`. The migration path from insecure to encrypted must be one command — any friction kills adoption.

This is the gateway drug. Once secrets are in Authy, the developer starts using `authy env`, `authy run`, and scoped policies. But first they need to get their secrets in.

## Current Behavior

No import command exists. Users must store secrets one at a time:

```bash
echo "sk-ant-xxx" | authy store anthropic-api-key
echo "ghp_xxx" | authy store github-token
# repeat for every secret...
```

## Proposed Behavior

### Basic usage

```bash
$ authy import .env
Imported 5 secrets: anthropic-api-key, db-url, github-token, redis-url, stripe-key
```

### What it does

1. Reads the `.env` file, parsing `KEY=VALUE` pairs
2. Transforms key names to Authy convention (lowercase, dashes)
3. Stores each as a secret in the vault
4. Reports what was imported

### Name transformation

`.env` files use `UPPER_SNAKE_CASE`. Authy convention is `lower-kebab-case`. By default, import transforms names:

```
ANTHROPIC_API_KEY  →  anthropic-api-key
DB_URL             →  db-url
GITHUB_TOKEN       →  github-token
```

Use `--keep-names` to preserve original names as-is.

### Flags

```
authy import <FILE> [--keep-names] [--prefix <PREFIX>] [--force] [--dry-run]
```

| Flag | Default | Description |
|------|---------|-------------|
| `<FILE>` | required | Path to `.env` file (or `-` for stdin) |
| `--keep-names` | off | Preserve original key names (no lowercasing or dash conversion) |
| `--prefix <PREFIX>` | none | Add prefix to imported secret names |
| `--force` | off | Overwrite existing secrets with same name |
| `--dry-run` | off | Show what would be imported without storing |

### Dotenv parsing rules

Follow the standard `.env` parsing behavior:

- Lines starting with `#` are comments (skipped)
- Empty lines are skipped
- `KEY=VALUE` — unquoted value, trimmed
- `KEY="VALUE"` — double-quoted, supports `\n`, `\r`, `\\`, `\"`
- `KEY='VALUE'` — single-quoted, literal (no escape processing)
- `KEY=` — empty value (stored as empty string)
- `export KEY=VALUE` — `export` prefix is stripped
- Lines without `=` are skipped with a warning to stderr
- Inline comments after unquoted values are stripped (`KEY=value # comment` → `value`)

### Conflict handling

When a secret name already exists in the vault:

- **Without `--force`**: skip it, print warning to stderr
- **With `--force`**: overwrite, bump version

```bash
$ authy import .env
Imported 3 secrets: api-key, db-url, stripe-key
Skipped 2 (already exist): github-token, redis-url
Use --force to overwrite existing secrets.
```

### Dry run

```bash
$ authy import .env --dry-run
Would import 5 secrets:
  ANTHROPIC_API_KEY → anthropic-api-key
  DB_URL → db-url
  GITHUB_TOKEN → github-token (exists, would skip)
  REDIS_URL → redis-url
  STRIPE_KEY → stripe-key
```

### Read from stdin

```bash
cat .env | authy import -
# or
authy import - < .env
```

Useful for piping from other tools or decrypted sources.

## Edge Cases

- File doesn't exist: error, exit 1
- File is empty: no-op, print "No secrets found in file", exit 0
- Duplicate keys in file: last value wins (standard dotenv behavior)
- Binary/non-UTF8 values: error on that entry, continue with others
- Very large files: no practical limit (vault is in-memory anyway)
- Name collision after transformation: `DB_URL` and `db-url` in same file both become `db-url` — last wins with warning

## Acceptance Criteria

- [ ] `authy import .env` reads and stores all key-value pairs
- [ ] Names are lowercased and underscores replaced with dashes by default
- [ ] `--keep-names` preserves original names
- [ ] `--force` overwrites existing secrets
- [ ] `--dry-run` shows preview without storing
- [ ] Handles quoted values, comments, empty lines, `export` prefix
- [ ] Reads from stdin with `-`
- [ ] Skips existing secrets without `--force`, reports them
- [ ] Audit log records each imported secret
