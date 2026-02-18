# 06 — Structured Errors

## Summary

Define typed exit codes and optional JSON error output so agents can programmatically handle failures.

## Motivation

Today, all errors exit with code 1 and print a human-readable message to stderr. An agent calling `authy get db-url` that fails cannot distinguish between "secret not found" (maybe try a different name), "access denied" (policy problem), "auth failed" (wrong credentials), or "vault not initialized" (setup problem). It just sees "non-zero exit" and a string it has to parse.

## Current Behavior

```rust
// src/main.rs
match result {
    Ok(()) => {}
    Err(e) => {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

All errors → exit 1. Message is the `Display` impl of `AuthyError`.

## Proposed Behavior

### Typed exit codes

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Operation completed |
| 1 | General error | Unexpected failures, I/O errors, serialization errors |
| 2 | Auth error | No credentials, wrong passphrase, invalid keyfile |
| 3 | Not found | Secret, policy, or session doesn't exist |
| 4 | Access denied | Policy check failed |
| 5 | Already exists | Secret or policy already exists (without `--force`) |
| 6 | Token error | Token expired, revoked, or invalid |
| 7 | Vault error | Vault not initialized or corrupted |
| 10 | Child process error | `authy run` child exited with non-zero (exit code forwarded if possible) |

### Mapping from `AuthyError`

```
VaultNotInitialized      → 7
VaultAlreadyExists       → 5
SecretNotFound           → 3
SecretAlreadyExists      → 5
PolicyNotFound           → 3
PolicyAlreadyExists      → 5
AccessDenied             → 4
AuthFailed               → 2
InvalidToken             → 6
TokenExpired             → 6
TokenRevoked             → 6
SessionNotFound          → 3
TokenReadOnly            → 4
Encryption               → 1
Decryption               → 2
InvalidKeyfile           → 2
Serialization            → 1
AuditChainBroken         → 1
Io                       → 1
Other                    → 1
```

### JSON error output

When `--json` is set (see [01-json-output.md](01-json-output.md)), errors are emitted as JSON to stderr:

```json
{
  "error": {
    "code": "secret_not_found",
    "message": "Secret 'db-url' not found",
    "exit_code": 3
  }
}
```

Error code strings:

| String | Exit Code |
|--------|-----------|
| `general_error` | 1 |
| `auth_failed` | 2 |
| `not_found` | 3 |
| `access_denied` | 4 |
| `already_exists` | 5 |
| `token_error` | 6 |
| `vault_error` | 7 |
| `child_process_error` | 10 |

### Without `--json`

Plain text error to stderr (unchanged from today, but with typed exit codes):

```
Error: Secret 'db-url' not found
```

Exit code: 3

### `authy run` exit code forwarding

When `authy run -- cmd` fails because the child process exits non-zero, forward the child's exit code directly. If the child exits with code 42, `authy run` exits with code 42.

If `authy run` itself fails (auth error, policy error, etc.), use the typed exit codes above.

## Edge Cases

- Child process killed by signal: exit with 128 + signal number (Unix convention)
- Multiple errors in one operation (e.g., `import` with some failures): exit with the code of the most severe error
- Exit codes > 125: avoid conflict with shell special codes (126 = not executable, 127 = not found, 128+ = signals). Our codes 1-10 are safe.

## Acceptance Criteria

- [ ] Each `AuthyError` variant maps to a specific exit code
- [ ] Exit codes are stable and documented
- [ ] `--json` mode emits JSON error objects to stderr
- [ ] Without `--json`, error messages are unchanged (backward compatible)
- [ ] `authy run` forwards child process exit codes
- [ ] Exit code 0 is only used on success
