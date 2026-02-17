# Security

## Threat Model

Authy protects secrets against:

1. **Unauthorized agent access** — agents can only read secrets allowed by their scope policy via short-lived session tokens
2. **Secret leakage to disk** — the vault is always encrypted at rest; secrets are decrypted only in memory
3. **Secret leakage via process metadata** — secrets are injected as child-process environment variables, never as CLI arguments (which appear in `/proc/*/cmdline`)
4. **Audit tampering** — the audit log uses an HMAC chain; any modification is detectable
5. **Token theft** — tokens are short-lived with configurable TTL and can be revoked instantly

Authy does **not** protect against:

- An attacker with root access on the same machine (they can read process memory)
- Keyfile theft (if the keyfile is compromised, the vault can be decrypted)
- A compromised agent exfiltrating secrets it has legitimate access to

## Cryptographic Primitives

| Purpose | Primitive | Implementation |
|---|---|---|
| Vault encryption | age (X25519 + ChaCha20-Poly1305) | `age` crate |
| Passphrase KDF | scrypt (via age) | `age` crate |
| Session token HMAC | HMAC-SHA256 | `hmac` + `sha2` crates |
| Key derivation (master → session/audit keys) | HKDF-SHA256 | `hkdf` crate |
| Token comparison | Constant-time equality | `subtle` crate |
| Memory zeroization | Zeroize on drop | `zeroize` crate |
| Random generation | OS CSPRNG | `rand::OsRng` |

## Security Invariants

1. **Secrets never exist unencrypted on disk.** The vault is decrypted into memory and zeroized on drop. `authy run` never writes temporary files.

2. **Secrets never appear in process argument lists.** `authy run --scope deploy -- ./deploy.sh` contains no secret values in argv. Secrets are passed via `std::process::Command::envs()`.

3. **Secrets never enter shell history.** Secret values are read from stdin, not CLI arguments. The `--value` flag (if ever added) would be documented as insecure.

4. **Session tokens are read-only.** No mutation operations (store, remove, rotate, policy changes) are possible with a session token. This is enforced at the auth layer.

5. **Policy evaluation is deny-by-default.** A secret is only accessible if it matches an `allow` pattern and does not match any `deny` pattern.

6. **Policies are tamper-proof.** Policies are stored inside the encrypted vault. Modifying them requires the master key.

## Reporting Vulnerabilities

If you find a security vulnerability, please report it privately. Do not open a public issue.
