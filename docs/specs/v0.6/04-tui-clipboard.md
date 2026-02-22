# 04 — TUI Clipboard Copy (OSC 52)

## Summary

Add `Ctrl+Y` clipboard copy in the TUI for secret values (reveal popup) and session tokens (show token popup) using the OSC 52 terminal escape sequence. Works in any terminal that supports OSC 52 (iTerm2, Alacritty, kitty, Windows Terminal, tmux with `set-clipboard on`).

## Motivation

Users currently see a secret value or token in a popup and must manually select and copy it. OSC 52 allows programmatic clipboard writes without requiring platform-specific clipboard binaries (`pbcopy`, `xclip`, etc.).

## OSC 52 Protocol

Write to stdout:
```
\x1b]52;c;{base64-encoded-data}\x07
```

- `\x1b]52;` — OSC sequence start, code 52 (clipboard)
- `c` — clipboard selection (as opposed to `p` for primary)
- `;` — separator
- `{base64}` — base64-encoded clipboard content
- `\x07` — ST (String Terminator)

The `base64` crate is already a dependency.

## Implementation

### `copy_to_clipboard` helper

```rust
/// Copy data to the system clipboard via OSC 52 escape sequence.
/// Returns true if the write succeeded (does not guarantee terminal support).
fn copy_to_clipboard(data: &str) -> bool {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    let seq = format!("\x1b]52;c;{}\x07", encoded);
    // Write directly to /dev/tty to bypass ratatui's alternate screen buffer
    if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
        use std::io::Write;
        tty.write_all(seq.as_bytes()).is_ok()
    } else {
        false
    }
}
```

Writing to `/dev/tty` ensures the escape sequence reaches the terminal even while ratatui has an alternate screen buffer active.

### Key Bindings

| Popup | Key | Action |
|-------|-----|--------|
| `RevealSecret` | `Ctrl+Y` | Copy secret value, show "Copied!" status |
| `ShowToken` | `Ctrl+Y` | Copy token, show "Copied!" status |

### Status Feedback

After a successful copy, show a brief `StatusMessage` popup:
```rust
PopupKind::StatusMessage {
    message: "Copied to clipboard.".into(),
    is_error: false,
    auto_close_at: Instant::now() + Duration::from_secs(2),
}
```

## File Changes

| File | Change |
|------|--------|
| `src/tui/mod.rs` | Add `copy_to_clipboard()` helper function |
| `src/tui/mod.rs` | Add `Ctrl+Y` handler in `RevealSecret` popup match arm |
| `src/tui/mod.rs` | Add `Ctrl+Y` handler in `ShowToken` popup match arm |
| `src/tui/mod.rs` | Update popup footer text to show `[Ctrl+Y] copy` |
| `src/tui/mod.rs` | Update help overlay to show `Ctrl+Y` binding |

## Footer Updates

### RevealSecret popup footer
Before: `[Esc] close  [Ctrl+R] reveal/mask  auto-close: Ns`
After:  `[Esc] close  [Ctrl+R] reveal/mask  [Ctrl+Y] copy  auto-close: Ns`

### ShowToken popup footer
Before: `[any key] close  auto-close: Ns  (copy this token now!)`
After:  `[Ctrl+Y] copy  [any key] close  auto-close: Ns`

## Tests

Manual only — OSC 52 requires a real terminal. Verify in a terminal that supports OSC 52:
1. Open TUI, reveal a secret, press Ctrl+Y, paste elsewhere
2. Create a session, press Ctrl+Y on the token popup, paste elsewhere
