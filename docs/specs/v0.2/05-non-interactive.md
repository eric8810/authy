# 05 — Non-Interactive Mode

## Summary

Ensure every Authy command can run without human interaction. Agents don't type passphrases, confirm prompts, or read stderr instructions.

## Motivation

AI agents call `authy` via bash. They cannot:
- Respond to interactive passphrase prompts
- Press `y/n` for confirmations
- Read "Enter passphrase:" messages on stderr and act on them

Today, Authy can be used non-interactively via env vars (`AUTHY_KEYFILE`, `AUTHY_PASSPHRASE`, `AUTHY_TOKEN`), but the behavior is implicit. There's no explicit signal that says "fail fast instead of prompting." An agent calling `authy get db-url` without credentials will hang waiting for a passphrase prompt.

## Current Behavior

- `authy init` without `--passphrase` or `--generate-keyfile`: prompts for passphrase interactively
- `authy store/rotate`: reads stdin — if stdin is a TTY, it waits for user input + Ctrl+D
- `authy get/list` without credentials: prompts for passphrase via `dialoguer`
- All commands: fall through to interactive passphrase prompt if no env vars set

When called by an agent, this means the process hangs indefinitely waiting for input that will never come.

## Proposed Behavior

### Detect non-interactive context automatically

When stdin is not a TTY (i.e., the process is being called from a script or agent), Authy should behave non-interactively by default:

- **No passphrase prompts.** If no credentials are provided via env vars or flags, fail immediately with a clear error.
- **No confirmation prompts.** Operations that would normally ask "are you sure?" proceed without asking (the caller is a script, it already decided).

### `AUTHY_NON_INTERACTIVE` env var

For cases where stdin IS a TTY but the caller still wants non-interactive behavior:

```bash
export AUTHY_NON_INTERACTIVE=1
authy get db-url  # fails immediately if no credentials available
```

### Behavior matrix

| Scenario | Current | Proposed |
|----------|---------|----------|
| `authy get` with no credentials, stdin is TTY | Prompts for passphrase | Prompts for passphrase (unchanged) |
| `authy get` with no credentials, stdin is NOT TTY | Prompts (hangs) | Error: "No credentials provided" (exit 2) |
| `authy get` with no credentials, `AUTHY_NON_INTERACTIVE=1` | Prompts | Error: "No credentials provided" (exit 2) |
| `authy get` with `AUTHY_KEYFILE` set | Works | Works (unchanged) |
| `authy get` with `AUTHY_TOKEN` set | Works | Works (unchanged) |
| `authy init` with no flags, stdin is NOT TTY | Prompts (hangs) | Error: "Use --generate-keyfile or --passphrase" (exit 2) |

### Error messages for non-interactive failures

Clear, actionable messages:

```
Error: No credentials provided. Set AUTHY_KEYFILE, AUTHY_PASSPHRASE, or AUTHY_TOKEN.
```

```
Error: Cannot prompt for passphrase in non-interactive mode. Use --passphrase or --generate-keyfile.
```

### stdin detection

Use `std::io::stdin().is_terminal()` (available in Rust's `std::io::IsTerminal` trait, stabilized in Rust 1.70). No additional crate needed.

## Edge Cases

- Piped stdin for `authy store`: `echo "secret" | authy store name` — stdin is not a TTY, but the command legitimately reads from stdin. This is not a conflict: `store` reads the secret value from stdin, and auth comes from env vars. If no auth env vars are set and stdin is not a TTY, fail on auth before attempting to read the value.
- `authy admin` in non-interactive mode: error immediately ("admin TUI requires an interactive terminal")
- `AUTHY_NON_INTERACTIVE=0` or `AUTHY_NON_INTERACTIVE=false`: treated as interactive (only `1` or `true` activates non-interactive)

## Acceptance Criteria

- [ ] When stdin is not a TTY, Authy never prompts for input
- [ ] When `AUTHY_NON_INTERACTIVE=1` is set, Authy never prompts for input
- [ ] Missing credentials produce a clear error message with remediation steps
- [ ] `authy store` and `authy rotate` still read secret values from piped stdin
- [ ] `authy admin` errors immediately in non-interactive mode
- [ ] All error messages include the specific env var or flag needed to proceed
- [ ] Exit code 2 for "missing credentials" (distinct from exit code 1 for runtime errors)
