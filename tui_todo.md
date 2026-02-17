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

- [x] Create `src/tui/dashboard.rs` — main layout (integrated in `src/tui/mod.rs`)
  - [x] Sidebar with section list (Secrets, Policies, Sessions, Audit)
  - [x] Tab / number key navigation between sections
  - [x] Status bar (vault path, auth method, last modified)
- [x] Create `src/tui/secrets.rs` — secrets list view (integrated in `src/tui/mod.rs`)
  - [x] Table: name, created, modified, version, tags
  - [x] j/k / arrow key navigation
  - [x] Enter → reveal popup
- [x] Reveal secret popup
  - [x] Show value (masked by default, Ctrl+R to reveal)
  - [x] Auto-close timer (30s default)
  - [x] Zeroize display buffer on close

## Phase 3: Secret write operations

- [x] Store new secret
  - [x] Name input field
  - [x] Masked value input field (Ctrl+R toggle)
  - [x] Tags input field (comma-separated)
  - [x] Save → `vault.secrets.insert()` + `save_vault()` + audit log
- [x] Rotate secret
  - [x] Masked value input for new value
  - [x] Save → update entry, bump version + `save_vault()` + audit log
- [x] Remove secret
  - [x] Confirmation dialog ("Delete 'db-url'? y/n")
  - [x] Save → `vault.secrets.remove()` + `save_vault()` + audit log

## Phase 4: Policy management

- [x] Create `src/tui/policies.rs` — policy list view (integrated in `src/tui/mod.rs`)
  - [x] Table: name, allow count, deny count, description
  - [x] j/k navigation
- [x] Create policy form
  - [x] Name input
  - [x] Description input
  - [x] Allow patterns (comma-separated)
  - [x] Deny patterns (comma-separated)
  - [x] Save → `vault.policies.insert()` + `save_vault()` + audit log
- [x] Edit policy form
  - [x] Pre-populated from existing policy
  - [x] Save → update + `save_vault()` + audit log
- [x] Remove policy
  - [x] Confirmation dialog
  - [x] Save → `vault.policies.remove()` + `save_vault()` + audit log
- [x] Test policy inline
  - [x] Secret name input → show ALLOWED/DENIED result

## Phase 5: Sessions + audit

### Sessions
- [x] Create `src/tui/sessions.rs` — session list view (integrated in `src/tui/mod.rs`)
  - [x] Table: id, scope, status (active/expired/revoked), expires (time remaining)
  - [x] j/k navigation
- [x] Create session
  - [x] Scope selector (list of existing policies, arrow keys to cycle)
  - [x] TTL input
  - [x] Show token once in popup (60s auto-close)
  - [x] Save → `vault.sessions.push()` + `save_vault()` + audit log
- [x] Revoke session
  - [x] Confirmation → set `revoked = true` + `save_vault()` + audit log
- [x] Revoke all sessions
  - [x] Confirmation → revoke all + `save_vault()` + audit log

### Audit
- [x] Create `src/tui/audit.rs` — audit log view (integrated in `src/tui/mod.rs`)
  - [x] Scrollable entry list (timestamp, operation, secret, outcome) — most recent first
  - [x] j/k + Page Up/Down navigation
- [x] Filter (`/` key)
  - [x] Filter by operation, actor, outcome, secret name
- [x] Verify chain (`v` key)
  - [x] Run `audit::verify_chain()`, show result inline (valid / broken at entry N)

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
