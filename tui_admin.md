# TUI Admin — Design & Implementation Plan

## Problem

Two gaps in the current design:

1. **Shell history leakage.** Admin operations like `echo "secret" | authy store` record secret values in `~/.bash_history`. Any agent with full bash access can read it. The SECURITY.md claim "secrets never enter shell history" is only true for `authy get` (stdout), not for the admin store workflow.

2. **CLI-only integration.** All modules are private to the binary (`mod` in `main.rs`, no `lib.rs`). Agent developers who want to integrate programmatically must shell out to the CLI. There's no reusable Rust API.

## Solution

Add `authy admin` — a password-gated TUI for all admin operations. This cleanly separates the two personas:

| Surface | Who | Auth | Capabilities |
|---|---|---|---|
| `authy admin` (TUI) | Human admin | Passphrase or keyfile, entered in TUI | Full CRUD: secrets, policies, sessions, audit |
| `authy get/run` (CLI) | Agents | Keyfile + session token (env vars) | Read-only, policy-scoped |

Secrets typed into TUI input fields never touch shell history, process argv, or parent environment. The vault is decrypted once at TUI launch and held in memory for the session — no repeated decrypt per operation.

## Dependencies

Add to `Cargo.toml`:

```toml
ratatui = "0.29"
crossterm = "0.28"
```

`ratatui` + `crossterm` is the standard Rust TUI stack. Single-binary, no runtime dependency. `crossterm` provides cross-platform terminal I/O (raw mode, events, alternate screen).

## Architecture

### New modules

```
src/
  tui/
    mod.rs          TuiApp struct, run() entry point, event loop
    auth.rs         Auth screen — passphrase/keyfile input before entering main UI
    dashboard.rs    Main layout: sidebar navigation + content area + status bar
    secrets.rs      Secrets view — list, store, reveal, rotate, remove
    policies.rs     Policies view — list, create, edit, remove, test
    sessions.rs     Sessions view — list, create, revoke
    audit.rs        Audit view — scrollable log, verify, filter
    widgets.rs      Shared widgets: masked input, confirmation dialog, popup, table
```

### New CLI command

Add `Admin` variant to `Commands` in `src/cli/mod.rs`:

```rust
/// Launch admin TUI (interactive vault management)
Admin {
    /// Keyfile path (alternative to passphrase prompt)
    #[arg(long, env = "AUTHY_KEYFILE")]
    keyfile: Option<String>,
},
```

Add `src/cli/admin.rs` handler that calls `tui::run()`.

### State model

```rust
// src/tui/mod.rs

pub struct TuiApp {
    // Auth (held for session lifetime, zeroized on drop)
    key: VaultKey,
    auth_ctx: AuthContext,

    // In-memory vault (reload on external change via mtime check)
    vault: Vault,

    // UI state
    screen: Screen,
    section: Section,        // Secrets | Policies | Sessions | Audit
    input_mode: InputMode,   // Normal | Editing
    // per-section state (cursor position, scroll offset, filter text)
}

enum Screen {
    Auth,       // First screen — passphrase/keyfile input
    Main,       // Dashboard with sidebar + content
    Popup(PopupKind),  // Modal overlay (reveal secret, confirm delete, show token)
}
```

The vault is loaded once after auth. Write operations mutate the in-memory vault and call `vault::save_vault()` atomically. A periodic or on-focus mtime check can detect external vault changes (e.g., another admin session).

### Relationship to existing modules

The TUI calls the same underlying modules the CLI handlers use:

```
TUI auth screen  →  vault::crypto (decrypt)
TUI store        →  vault::load_vault / save_vault, audit::log_event
TUI policy CRUD  →  policy::Policy, vault::save_vault
TUI session      →  session::generate_token, vault::save_vault
TUI audit view   →  audit::read_entries, audit::verify_chain
```

No changes needed to `vault/`, `policy/`, `session/`, `audit/`, or `subprocess/`. The TUI is a new frontend that reuses existing core logic.

### Event loop

```
┌─ Terminal raw mode on, alternate screen ─┐
│                                           │
│  loop {                                   │
│    1. crossterm::event::poll(250ms)       │
│    2. handle key event → update state     │
│    3. ratatui: terminal.draw(|f| { ... }) │
│    4. if quit requested → break           │
│  }                                        │
│                                           │
│  Zeroize VaultKey, restore terminal       │
└───────────────────────────────────────────┘
```

## TUI Screens

### 1. Auth screen

```
┌─────────────────────────────────────┐
│           authy admin               │
│                                     │
│  Vault: ~/.authy/vault.age          │
│                                     │
│  Passphrase: ●●●●●●●●●●            │
│                                     │
│  [Enter] Unlock    [Esc] Quit       │
└─────────────────────────────────────┘
```

- Masked password input (toggle visibility with Ctrl+R)
- If `--keyfile` was given, skip prompt and decrypt directly
- On failure: show error inline, allow retry
- On success: load vault → transition to Main screen

### 2. Main dashboard

```
┌──────────┬──────────────────────────────────────────────┐
│          │ Secrets (4)                                   │
│ Secrets  │──────────────────────────────────────────────│
│ Policies │ NAME              CREATED      VERSION  TAGS  │
│ Sessions │ db-url            2026-02-15   1        prod  │
│ Audit    │ openai-api-key    2026-02-15   1             │
│          │ github-token      2026-02-15   2        ci    │
│          │ aws-secret-key    2026-02-16   1        prod  │
│          │                                               │
│          │                                               │
│          │                                               │
├──────────┴──────────────────────────────────────────────┤
│ [s]tore  [Enter]reveal  [r]otate  [d]elete    [q]uit    │
│ vault: ~/.authy/vault.age  auth: passphrase  modified: … │
└─────────────────────────────────────────────────────────┘
```

- Sidebar: Tab/Shift+Tab or number keys (1-4) to switch sections
- Content: j/k or arrow keys to navigate, Enter to select
- Status bar: vault path, auth method, last modified timestamp

### 3. Secret input (store/rotate)

```
┌────────────────────────────────────┐
│ Store new secret                   │
│                                    │
│ Name:  new-api-key                 │
│ Value: ●●●●●●●●●●●●●●●            │
│ Tags:  prod, api                   │
│                                    │
│ [Ctrl+R] reveal  [Enter] save     │
│ [Esc] cancel                       │
└────────────────────────────────────┘
```

- Value field is masked by default (Ctrl+R toggles)
- Multiline value support (Ctrl+Enter for newline, Enter to submit)
- Secret value goes directly into `SecretEntry` — never touches shell

### 4. Reveal secret (popup)

```
┌────────────────────────────────────┐
│ db-url (v1)                        │
│                                    │
│ postgres://user:pass@db:5432/prod  │
│                                    │
│ [Esc] close   auto-close: 30s     │
└────────────────────────────────────┘
```

- Auto-close timer (configurable, default 30s)
- Value is zeroized from display buffer on close

### 5. Policy management

```
┌──────────┬──────────────────────────────────────────────┐
│          │ Policies (2)                                  │
│ Secrets  │──────────────────────────────────────────────│
│>Policies │ NAME            ALLOW   DENY   DESCRIPTION    │
│ Sessions │ deploy-agent    2       1      Deploy scope   │
│ Audit    │ ci-agent        3       0      CI pipeline    │
│          │                                               │
├──────────┴──────────────────────────────────────────────┤
│ [c]reate  [e]dit  [d]elete  [t]est              [q]uit  │
└─────────────────────────────────────────────────────────┘
```

- Create/Edit opens a form with: name, description, allow patterns (one per line), deny patterns
- Test: enter a secret name → shows ALLOWED/DENIED result inline

### 6. Session management

```
┌──────────┬──────────────────────────────────────────────┐
│          │ Sessions (3)                                  │
│ Secrets  │──────────────────────────────────────────────│
│ Policies │ ID        SCOPE          STATUS   EXPIRES     │
│>Sessions │ a3f2c1    deploy-agent   active   2h left     │
│ Audit    │ b7d4e9    ci-agent       active   45m left    │
│          │ c1a8f3    deploy-agent   revoked  —           │
│          │                                               │
├──────────┴──────────────────────────────────────────────┤
│ [c]reate  [r]evoke  [R]evoke all                [q]uit  │
└─────────────────────────────────────────────────────────┘
```

- Create: select scope from existing policies (dropdown/list), enter TTL
- Token shown once in a popup (same auto-close pattern as secret reveal)
- Copy-to-clipboard hint if terminal supports OSC 52

### 7. Audit log viewer

```
┌──────────┬──────────────────────────────────────────────┐
│          │ Audit Log (47 entries) ✓ chain valid          │
│ Secrets  │──────────────────────────────────────────────│
│ Policies │ 2026-02-16 10:32  store    db-url      ok    │
│ Sessions │ 2026-02-16 10:33  policy   deploy-agt  ok    │
│>Audit    │ 2026-02-16 11:01  get      db-url      ok    │
│          │ 2026-02-16 11:01  get      openai-key  DENY  │
│          │ 2026-02-16 11:05  session  —           ok    │
│          │                                               │
├──────────┴──────────────────────────────────────────────┤
│ [v]erify chain  [/]filter                       [q]uit  │
└─────────────────────────────────────────────────────────┘
```

- Scrollable with j/k, Page Up/Down
- `/` to filter by operation, actor, or outcome
- `v` to verify HMAC chain (shows result inline)

## Key Bindings

| Key | Context | Action |
|---|---|---|
| `Tab` / `1-4` | Global | Switch sidebar section |
| `j/k` or `↑/↓` | List | Navigate items |
| `Enter` | List | Select / reveal |
| `s` | Secrets | Store new secret |
| `r` | Secrets | Rotate selected |
| `d` | Any list | Delete selected (with confirmation) |
| `c` | Policies/Sessions | Create new |
| `e` | Policies | Edit selected |
| `t` | Policies | Test policy |
| `Ctrl+R` | Input field | Toggle value visibility |
| `/` | Audit | Open filter |
| `v` | Audit | Verify chain |
| `?` | Global | Help overlay |
| `q` / `Ctrl+C` | Global | Quit |
| `Esc` | Popup/Input | Cancel / close |

## Implementation Phases

### Phase 1: TUI shell + auth screen
**Files:** `Cargo.toml`, `src/cli/mod.rs`, `src/cli/admin.rs`, `src/tui/mod.rs`, `src/tui/auth.rs`, `src/tui/widgets.rs`

- Add `ratatui` + `crossterm` dependencies
- Add `Admin` command to CLI
- Implement event loop skeleton (raw mode, alternate screen, cleanup on panic)
- Auth screen with masked passphrase input
- Vault decrypt + hold `VaultKey` in `TuiApp` state
- Zeroize on drop

### Phase 2: Dashboard + secrets view (read-only)
**Files:** `src/tui/dashboard.rs`, `src/tui/secrets.rs`

- Sidebar navigation (section switching)
- Secrets table (name, created, version, tags)
- Reveal secret popup (masked, toggle, auto-close timer)
- Status bar

### Phase 3: Secret write operations
**Files:** `src/tui/secrets.rs`, `src/tui/widgets.rs`

- Store: name + masked value input form → `vault.secrets.insert()` + `save_vault()`
- Rotate: masked value input → update entry + bump version + save
- Remove: confirmation dialog → `vault.secrets.remove()` + save
- Audit logging for each operation

### Phase 4: Policy management
**Files:** `src/tui/policies.rs`

- Policy list table
- Create form: name, description, allow/deny pattern lists
- Edit form: modify existing policy
- Remove with confirmation
- Test: inline secret name input → show ALLOWED/DENIED

### Phase 5: Sessions + audit
**Files:** `src/tui/sessions.rs`, `src/tui/audit.rs`

- Session list with status display (active/expired/revoked, time remaining)
- Create: scope selector from policy list, TTL input → show token once in popup
- Revoke / revoke-all
- Audit log: scrollable entry list, filter, chain verify

## Testing

- **Unit tests:** Widget rendering (masked input, popup layout), key binding dispatch, name transform edge cases
- **Integration tests:** Spawn `authy admin` with `--keyfile`, send simulated key events via crossterm's event injection, verify vault state after operations
- **Manual verification:**
  1. `cargo build && ./target/debug/authy init --generate-keyfile /tmp/test.key`
  2. `./target/debug/authy admin --keyfile /tmp/test.key`
  3. Store a secret via TUI → verify `grep -r "secret-value" ~/.bash_history` finds nothing
  4. `authy list` from CLI confirms secret was stored
  5. Create policy + session in TUI → use token from agent CLI to verify scoped access

## Future: `lib.rs` extraction

The TUI work naturally motivates extracting a `lib.rs` — the TUI and CLI handlers share the same core operations (`vault::load_vault`, `vault::save_vault`, `policy::can_read`, `session::generate_token`, etc.). Once the TUI is stable, refactoring `main.rs` from `mod` to `pub mod` re-exports through a `lib.rs` gives external crates a Rust API. This unblocks:

- MCP server mode (`authy serve --mcp`)
- Language bindings via FFI / WASM
- Embedding authy as a library in agent frameworks

This is a follow-up, not a prerequisite. The TUI can reuse existing modules as-is since it lives in the same crate.
