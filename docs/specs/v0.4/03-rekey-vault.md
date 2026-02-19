# 05 — Passphrase Change / Re-key Vault

## Summary

Add `authy rekey` to change the vault's passphrase or switch between passphrase and keyfile auth.

## Motivation

Users who initialized with a passphrase may want to switch to a keyfile (or vice versa). Users who suspect their passphrase is compromised need to re-encrypt without losing data. There's currently no way to change credentials without manually decrypting and re-encrypting.

## `authy rekey`

### Usage

```bash
# Change passphrase
authy rekey

# Switch from passphrase to keyfile
authy rekey --generate-keyfile ~/.authy/keys/new.key

# Switch from keyfile to passphrase
authy rekey --to-passphrase

# Re-encrypt with a new keyfile
authy rekey --new-keyfile ~/.authy/keys/new.key
```

### Behavior

1. Authenticate with current credentials (passphrase/keyfile)
2. Decrypt vault into memory
3. Prompt for new credentials:
   - Default: prompt for new passphrase (if current is passphrase, prompt for new one)
   - `--generate-keyfile <path>`: generate new keyfile, re-encrypt with it
   - `--to-passphrase`: prompt for new passphrase (when switching from keyfile)
   - `--new-keyfile <path>`: re-encrypt with existing keyfile at path
4. Re-encrypt vault with new credentials
5. Atomic write (write to .tmp, rename)
6. Audit log entry: "vault_rekey" event
7. Print confirmation to stderr

### CLI Definition

```rust
/// Change vault passphrase or switch auth method
Rekey {
    /// Generate a new keyfile at this path
    #[arg(long)]
    generate_keyfile: Option<String>,
    /// Switch to passphrase auth
    #[arg(long)]
    to_passphrase: bool,
    /// Re-encrypt with an existing keyfile
    #[arg(long)]
    new_keyfile: Option<String>,
},
```

### Validation

- `--generate-keyfile`, `--to-passphrase`, and `--new-keyfile` are mutually exclusive
- If none specified: re-prompt for passphrase (same auth method, new passphrase)
- `--generate-keyfile` path must not already exist (safety check)
- `--new-keyfile` path must exist and be a valid age identity

### Session Tokens After Rekey

Session tokens are HMAC'd with a key derived from the master key. After rekey, the master key changes, so all existing tokens become invalid. This is correct behavior — after a credential change, existing sessions should be invalidated.

Document this clearly in the command output: "All existing session tokens have been invalidated."

## Tests

- Rekey passphrase to new passphrase
- Rekey passphrase to keyfile
- Rekey keyfile to passphrase
- Rekey keyfile to new keyfile
- Vault contents preserved after rekey
- Old passphrase no longer works after rekey
- Audit log entry written
- Existing sessions invalidated after rekey
