# v0.4 — Task Tracker

Status: `[ ]` todo · `[~]` in progress · `[x]` done

---

## 01 — File placeholders + `authy resolve`

- [ ] Add `Resolve` variant to `Commands` enum in `src/cli/mod.rs`
- [ ] Create `src/cli/resolve.rs` — placeholder resolution logic
- [ ] Implement placeholder regex: `<authy:[a-z0-9][a-z0-9-]*>`
- [ ] Read source file, find all placeholders, look up in vault
- [ ] Replace placeholders with real values
- [ ] Output to `--output` path or stdout
- [ ] Error on missing key, error on access denied
- [ ] Wire up in `src/main.rs`
- [ ] Integration tests: resolve yaml, json, missing key, access denied, stdout, passthrough

## 02 — Safe/sensitive command split

- [ ] Add run-only check to `authy resolve` (allow — values go to file, not stdout)
- [ ] Update `docs/GUIDE.md` with safe/sensitive command table
- [ ] Update skill SKILL.md to mention `resolve`
- [ ] Verify all existing run-only tests still pass

## 03 — Rekey vault

- [ ] Add `Rekey` variant to `Commands` enum in `src/cli/mod.rs`
- [ ] Create `src/cli/rekey.rs`
- [ ] Implement: authenticate → decrypt → prompt new credentials → re-encrypt → atomic write
- [ ] Support `--generate-keyfile`, `--to-passphrase`, `--new-keyfile` (mutually exclusive)
- [ ] Audit log entry for rekey event
- [ ] Print warning that session tokens are invalidated
- [ ] Wire up in `src/main.rs`
- [ ] Integration test: rekey passphrase to passphrase
- [ ] Integration test: rekey passphrase to keyfile
- [ ] Integration test: old passphrase fails after rekey
- [ ] Integration test: vault contents preserved after rekey

---

## Final

- [ ] `cargo clippy -- -D warnings` passes clean
- [ ] `cargo test` — all tests pass
- [ ] Update CHANGELOG.md with v0.4 entries
- [ ] Bump version in Cargo.toml to 0.4.0
