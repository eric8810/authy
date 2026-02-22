# 05 — TUI Vault Change Detection

## Summary

Detect external vault modifications while the TUI is open and prompt the user to reload. Uses filesystem mtime polling (not inotify/kqueue) for simplicity and portability.

## Motivation

If the user runs `authy store` in another terminal while the TUI is open, the TUI shows stale data. Without detection, the TUI could overwrite the external changes on its next save. This is the "lost update" problem.

## Design

### State

Add to `TuiApp`:
```rust
pub last_vault_mtime: Option<std::time::SystemTime>,
```

### Recording mtime

```rust
fn record_vault_mtime(&mut self) {
    self.last_vault_mtime = std::fs::metadata(vault::vault_path())
        .and_then(|m| m.modified())
        .ok();
}
```

Call after:
1. Successful authentication (vault loaded)
2. Every `save_vault()` call (we just wrote the file, record the new mtime)

### Checking for changes

```rust
fn vault_changed_externally(&self) -> bool {
    let current = std::fs::metadata(vault::vault_path())
        .and_then(|m| m.modified())
        .ok();
    match (self.last_vault_mtime, current) {
        (Some(last), Some(now)) => now > last,
        _ => false,
    }
}
```

### Polling

On each tick (250ms), if:
- Screen is `Main`
- No popup is active

Then check `vault_changed_externally()`. If true, show `VaultChanged` popup.

### VaultChanged Popup

Add variant to `PopupKind`:
```rust
PopupKind::VaultChanged
```

Renders a `ConfirmDialog`:
- Title: "Vault changed"
- Message: "The vault was modified externally. Reload?"
- `y` → reload vault from disk, call `record_vault_mtime()`
- `n` → dismiss popup, call `record_vault_mtime()` to suppress repeated prompts

### `save_vault` Signature Change

Change `save_vault(&self)` to `save_vault(&mut self)` so it can call `record_vault_mtime()` after saving. All existing call sites already have `&mut self` access (they're in `handle_popup_input` which takes `&mut TuiApp`).

## File Changes

| File | Change |
|------|--------|
| `src/tui/mod.rs` | Add `last_vault_mtime` field to `TuiApp` |
| `src/tui/mod.rs` | Add `record_vault_mtime()` and `vault_changed_externally()` methods |
| `src/tui/mod.rs` | Change `save_vault(&self)` to `save_vault(&mut self)`, call `record_vault_mtime()` after save |
| `src/tui/mod.rs` | Add mtime check in tick section of event loop |
| `src/tui/mod.rs` | Add `PopupKind::VaultChanged` variant |
| `src/tui/mod.rs` | Add `VaultChanged` input handler (y/n) |
| `src/tui/mod.rs` | Add `VaultChanged` draw handler |
| `src/tui/auth.rs` | Call `record_vault_mtime()` after successful auth |

## Tests

Manual only — requires running TUI and modifying vault externally:
1. Open TUI, `authy store` in another terminal, confirm reload prompt appears
2. Press `y` → verify new secret is visible
3. Press `n` → verify prompt doesn't reappear until another change
