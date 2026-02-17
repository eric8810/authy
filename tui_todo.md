# TUI Admin — Development Tracker

> Derived from [tui_admin.md](tui_admin.md). Update status as work progresses.

Status key: `[ ]` todo · `[~]` in progress · `[x]` done · `[-]` skipped

---

## Phase 1: TUI shell + auth screen

- [x] Add `ratatui = "0.29"` and `crossterm = "0.28"` to `Cargo.toml`
- [x] Add `Admin` variant to `Commands` enum in `src/cli/mod.rs`
- [x] Create `src/cli/admin.rs` — handler that calls `tui::run()`
- [x] Create `src/tui/mod.rs` — `TuiApp` struct, `run()` entry point, event loop
  - [x] Raw mode + alternate screen setup
  - [x] Graceful cleanup on panic (restore terminal)
  - [x] `TuiApp` holds `VaultKey`, `AuthContext`, `Vault` (zeroize on drop)
- [x] Create `src/tui/auth.rs` — auth screen
  - [x] Masked passphrase input field
  - [x] Toggle visibility (Ctrl+R)
  - [x] Keyfile path bypass (skip prompt if `--keyfile` given)
  - [x] Error display on failed decrypt, allow retry
  - [x] On success: load vault, transition to main screen
- [x] Create `src/tui/widgets.rs` — shared widget primitives
  - [x] Masked text input widget
  - [x] Confirmation dialog widget

## Phase 2: Dashboard + secrets view (read-only)

- [ ] Create `src/tui/dashboard.rs` — main layout
  - [ ] Sidebar with section list (Secrets, Policies, Sessions, Audit)
  - [ ] Tab / number key navigation between sections
  - [ ] Status bar (vault path, auth method, last modified)
- [ ] Create `src/tui/secrets.rs` — secrets list view
  - [ ] Table: name, created, modified, version, tags
  - [ ] j/k / arrow key navigation
  - [ ] Enter → reveal popup
- [ ] Reveal secret popup
  - [ ] Show value (masked by default, Ctrl+R to reveal)
  - [ ] Auto-close timer (30s default)
  - [ ] Zeroize display buffer on close

## Phase 3: Secret write operations

- [ ] Store new secret
  - [ ] Name input field
  - [ ] Masked value input field (Ctrl+R toggle)
  - [ ] Tags input field (comma-separated)
  - [ ] Save → `vault.secrets.insert()` + `save_vault()` + audit log
- [ ] Rotate secret
  - [ ] Masked value input for new value
  - [ ] Save → update entry, bump version + `save_vault()` + audit log
- [ ] Remove secret
  - [ ] Confirmation dialog ("Delete 'db-url'? y/n")
  - [ ] Save → `vault.secrets.remove()` + `save_vault()` + audit log

## Phase 4: Policy management

- [ ] Create `src/tui/policies.rs` — policy list view
  - [ ] Table: name, allow count, deny count, description
  - [ ] j/k navigation
- [ ] Create policy form
  - [ ] Name input
  - [ ] Description input
  - [ ] Allow patterns (multi-line, one per line)
  - [ ] Deny patterns (multi-line, one per line)
  - [ ] Save → `vault.policies.insert()` + `save_vault()` + audit log
- [ ] Edit policy form
  - [ ] Pre-populated from existing policy
  - [ ] Save → update + `save_vault()` + audit log
- [ ] Remove policy
  - [ ] Confirmation dialog
  - [ ] Save → `vault.policies.remove()` + `save_vault()` + audit log
- [ ] Test policy inline
  - [ ] Secret name input → show ALLOWED/DENIED result

## Phase 5: Sessions + audit

### Sessions
- [ ] Create `src/tui/sessions.rs` — session list view
  - [ ] Table: id, scope, status (active/expired/revoked), expires (time remaining)
  - [ ] j/k navigation
- [ ] Create session
  - [ ] Scope selector (list of existing policies)
  - [ ] TTL input
  - [ ] Show token once in popup (auto-close)
  - [ ] Save → `vault.sessions.push()` + `save_vault()` + audit log
- [ ] Revoke session
  - [ ] Confirmation → set `revoked = true` + `save_vault()` + audit log
- [ ] Revoke all sessions
  - [ ] Confirmation → revoke all + `save_vault()` + audit log

### Audit
- [ ] Create `src/tui/audit.rs` — audit log view
  - [ ] Scrollable entry list (timestamp, operation, secret, actor, outcome)
  - [ ] j/k + Page Up/Down navigation
- [ ] Filter (`/` key)
  - [ ] Filter by operation, actor, outcome
- [ ] Verify chain (`v` key)
  - [ ] Run `audit::verify_chain()`, show result inline (valid / broken at entry N)

## Phase 6: Polish + testing

- [ ] Help overlay (`?` key) — show key binding reference
- [ ] Quit confirmation if vault was modified since last save
- [ ] Unit tests — widget rendering, key dispatch, state transitions
- [ ] Integration tests — spawn with `--keyfile`, simulate key events, verify vault state
- [ ] Manual test: store secret via TUI, confirm not in bash_history
- [ ] Update README.md with `authy admin` usage
- [ ] Update SECURITY.md threat model (TUI mitigates history leakage)

---

## Future (post-TUI)

- [ ] Extract `lib.rs` — make core modules public for external crate usage
- [ ] MCP server mode (`authy serve --mcp`)
- [ ] Clipboard support for token copy (OSC 52)
- [ ] External vault change detection (mtime polling, reload prompt)
