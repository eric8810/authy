# Architecture

## Overview

```
┌──────────────────────────────────────────────────────────┐
│                        CLI Layer                         │
│  src/cli/*.rs — clap command definitions + handlers      │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                      Auth Layer                          │
│  src/auth/ — resolves passphrase/keyfile/token → context │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                    Policy Layer                          │
│  src/policy/ — glob-based allow/deny evaluation          │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                     Vault Layer                          │
│  src/vault/ — encrypted storage, load/save, crypto       │
└──────────────────────┬───────────────────────────────────┘
                       │
            ┌──────────┴──────────┐
            │                     │
┌───────────▼──────┐   ┌─────────▼────────┐
│  Session Layer   │   │   Audit Layer    │
│  src/session/    │   │   src/audit/     │
│  token gen/val   │   │   JSONL + HMAC   │
└──────────────────┘   └──────────────────┘
```

## Source Layout

```
src/
  main.rs               Entry point — parse CLI args, dispatch to command handlers
  error.rs              AuthyError enum (thiserror), Result type alias
  types.rs              Common re-exports (serde, chrono, BTreeMap, PathBuf)

  cli/
    mod.rs              Clap derive structs for all commands and subcommands
    init.rs             authy init — create vault, generate keyfile or prompt passphrase
    store.rs            authy store — decrypt vault, insert secret, re-encrypt
    get.rs              authy get — decrypt vault, policy check, run-only check, output to stdout
    list.rs             authy list — decrypt vault, optional scope filter (allowed in run-only)
    remove.rs           authy remove — decrypt vault, delete secret, re-encrypt
    rotate.rs           authy rotate — update secret value, bump version
    policy.rs           authy policy * — CRUD for scope policies (supports --run-only)
    session.rs          authy session * — create/list/revoke tokens (supports --run-only)
    run.rs              authy run — subprocess injection with scoped secrets (allowed in run-only)
    env.rs              authy env — output secrets as shell/dotenv/json (blocked in run-only)
    import.rs           authy import — import secrets from .env files
    export.rs           authy export — export secrets as .env or JSON (blocked in run-only)
    common.rs           Shared secret resolution (resolve_scoped_secrets)
    json_output.rs      Serialize structs for JSON output
    audit.rs            authy audit * — show/verify/export audit log
    config.rs           authy config — show configuration
    admin.rs            authy admin — launch TUI

  vault/
    mod.rs              Vault struct, VaultKey enum, load_vault(), save_vault()
    crypto.rs           age encrypt/decrypt (passphrase + keyfile), HKDF derivation
    secret.rs           SecretEntry, SecretMetadata

  auth/
    mod.rs              Auth dispatcher — resolve credentials to an AuthContext
    context.rs          AuthContext — carries resolved identity and permission level

  policy/
    mod.rs              Policy struct, can_read() with globset matching

  session/
    mod.rs              SessionRecord, generate_token(), validate_token()

  audit/
    mod.rs              AuditEntry, append_entry(), verify_chain()

  subprocess/
    mod.rs              Spawn child process with env var injection

  config/
    mod.rs              authy.toml parsing
```

## Data Flow

### Store a secret (admin, master key)

```
authy store db-url
  → auth: resolve passphrase/keyfile → VaultKey
  → vault: load_vault(key) → decrypt vault.age → Vault in memory
  → read secret value from stdin
  → insert into vault.secrets["db-url"]
  → vault: save_vault(vault, key) → serialize → encrypt → atomic write
  → audit: append SecretWrite entry
```

### Get a secret (agent, session token)

```
authy get db-url (with AUTHY_TOKEN + AUTHY_KEYFILE)
  → auth: resolve token + keyfile → VaultKey + session scope
  → vault: load_vault(key) → decrypt vault.age
  → session: validate_token() → find matching session → get scope
  → policy: scope.can_read("db-url") → allow/deny
  → if allowed: write secret value to stdout
  → audit: append SecretRead entry (GRANTED or DENIED)
```

### Run subprocess (agent, scoped)

```
authy run --scope deploy -- ./deploy.sh
  → auth: resolve credentials
  → vault: load_vault() → decrypt
  → policy: filter all secrets by scope → allowed set
  → subprocess: Command::new("./deploy.sh")
      .envs(allowed_secrets_as_env_vars)
      .spawn()
  → forward exit code
  → audit: append SubprocessRun entry
```

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Single vault file | MessagePack + age | No metadata leakage (secret names hidden), atomic updates |
| Whole-vault encryption | vs per-secret | Simpler, no info leak about count/size, fast at target scale |
| Policies inside vault | vs separate file | Can't tamper policies without master key |
| HMAC tokens | vs JWT/signed | Simpler, no need for asymmetric crypto, revocable via vault |
| Stateless CLI | vs daemon | Simpler to build/audit/deploy; daemon is a future phase |
| MessagePack | vs JSON/CBOR | Compact, fast, well-supported in Rust via rmp-serde |
| JSONL audit | vs encrypted log | Appendable without decryption; HMAC chain detects tampering |
