# 02 — File Placeholders

## Summary

Config files use `<authy:key-name>` placeholders instead of real secret values. `authy resolve` replaces placeholders with real values from the vault. This keeps secrets out of source files, git history, and LLM context.

## Motivation

`authy run` covers secrets that flow through environment variables. But many tools need secrets in config files (database.yml, docker-compose.yml, stripe config, etc.). Today developers put real values in these files or use `.env` + template substitution. Both leak secrets to agents that read files.

## Placeholder Format

```
<authy:key-name>
```

- Prefix: `<authy:`
- Key name: lowercase alphanumeric + hyphens (matches authy secret name rules)
- Suffix: `>`
- Regex: `<authy:[a-z0-9][a-z0-9-]*>`

### Examples

```yaml
# config/stripe.yaml
api_key: <authy:stripe-api-key>
webhook_secret: <authy:stripe-webhook-secret>
```

```json
{
  "database": {
    "url": "<authy:db-url>",
    "password": "<authy:db-password>"
  }
}
```

The format is:
- Visually obvious in any file
- Invalid shell syntax (won't accidentally expand)
- Invalid in most template engines (not `${}`, `{{}}`, `%s`)
- Safe to commit to git
- Safe to send to LLMs

## `authy resolve` Command

```bash
authy resolve <file> [--output <path>]
```

- Reads the source file
- Finds all `<authy:key-name>` placeholders
- Looks up each key in the vault (using current auth context)
- Replaces placeholders with real values
- Writes to `--output` path (or stdout if omitted)

### Behavior

- If a placeholder references a key not in the vault → error with the missing key name
- If a placeholder references a key denied by scope → error with access denied
- Scope is resolved from `.authy.toml` or `--scope` flag
- File format is irrelevant — authy does plain text substitution, works with yaml, json, toml, ini, xml, anything

### CLI Definition

```rust
/// Resolve <authy:key-name> placeholders in a file
Resolve {
    /// Source file with placeholders
    file: String,
    /// Output path (default: stdout)
    #[arg(long, short)]
    output: Option<String>,
    /// Scope for secret access
    #[arg(long)]
    scope: Option<String>,
},
```

## Tests

- Resolve single placeholder in yaml
- Resolve multiple placeholders in json
- Error on missing key
- Error on access denied by scope
- Stdout output when no `--output`
- File output with `--output`
- No placeholders = passthrough (file copied as-is)
- Nested placeholder-like strings that don't match format are left alone
