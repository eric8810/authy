use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use authy::auth::context::AuthContext;
use authy::error::AuthyError;
use authy::vault::{self, VaultKey};

use super::widgets;
use super::{Screen, TuiApp};

/// Handle key input on the auth screen.
pub fn handle_input(app: &mut TuiApp, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.auth_error = None;
            match app.try_auth() {
                Ok(()) => {
                    app.record_vault_mtime();
                    app.screen = Screen::Main;
                }
                Err(e) => {
                    app.auth_error = Some(format!("{}", e));
                    app.auth_input.clear();
                }
            }
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {
            app.auth_input.handle_input(key);
        }
    }
}

/// Try to authenticate using the app's current state.
pub fn try_authenticate(app: &mut TuiApp) -> authy::error::Result<()> {
    if let Some(ref keyfile_path) = app.keyfile {
        // Keyfile auth
        let content = std::fs::read_to_string(keyfile_path)
            .map_err(|e| AuthyError::InvalidKeyfile(format!("Cannot read {}: {}", keyfile_path, e)))?;

        let identity: age::x25519::Identity = content
            .trim()
            .parse()
            .map_err(|e: &str| AuthyError::InvalidKeyfile(e.to_string()))?;

        let pubkey = identity.to_public().to_string();
        let key = VaultKey::Keyfile {
            identity: content.trim().to_string(),
            pubkey,
        };

        let vault_data = vault::load_vault(&key)?;
        app.key = Some(key);
        app.auth_ctx = Some(AuthContext::master_keyfile());
        app.vault = Some(vault_data);
        Ok(())
    } else {
        // Passphrase auth
        let passphrase = app.auth_input.value.clone();
        if passphrase.is_empty() {
            return Err(AuthyError::AuthFailed("Passphrase cannot be empty".into()));
        }

        let key = VaultKey::Passphrase(passphrase);
        let vault_data = vault::load_vault(&key)?;
        app.key = Some(key);
        app.auth_ctx = Some(AuthContext::master_passphrase());
        app.vault = Some(vault_data);
        Ok(())
    }
}

/// Draw the auth screen.
pub fn draw(frame: &mut Frame, app: &TuiApp) {
    let area = frame.area();

    // Center the auth form
    let form_width = 50u16.min(area.width.saturating_sub(4));
    let form_height = if app.auth_error.is_some() { 10 } else { 8 };
    let form = widgets::centered_rect(
        (form_width * 100 / area.width.max(1)).max(40),
        form_height,
        area,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" authy admin ")
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(form);
    frame.render_widget(block, form);

    let vault_path = vault::vault_path().display().to_string();

    let mut y = inner.y;

    // Vault path
    let vault_line = Paragraph::new(Span::styled(
        format!("Vault: {}", vault_path),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(
        vault_line,
        Rect { x: inner.x + 1, y, width: inner.width.saturating_sub(2), height: 1 },
    );
    y += 2;

    // Passphrase input or keyfile indicator
    if app.keyfile.is_some() {
        let kf_line = Paragraph::new(Span::styled(
            "Auth: keyfile (provided via --keyfile)",
            Style::default().fg(Color::Green),
        ));
        frame.render_widget(
            kf_line,
            Rect { x: inner.x + 1, y, width: inner.width.saturating_sub(2), height: 1 },
        );
    } else {
        let input_area = Rect {
            x: inner.x + 1,
            y,
            width: inner.width.saturating_sub(2),
            height: 1,
        };
        widgets::render_input(frame, input_area, &app.auth_input, "Passphrase", true);
    }
    y += 2;

    // Error message
    if let Some(ref err) = app.auth_error {
        let err_line = Paragraph::new(Span::styled(
            err.as_str(),
            Style::default().fg(Color::Red),
        ));
        frame.render_widget(
            err_line,
            Rect { x: inner.x + 1, y, width: inner.width.saturating_sub(2), height: 1 },
        );
        y += 2;
    }

    // Hint
    let hint = if app.keyfile.is_some() {
        "[Enter] Retry  [Esc] Quit"
    } else {
        "[Enter] Unlock  [Ctrl+R] Toggle visibility  [Esc] Quit"
    };
    let hint_line = Paragraph::new(Span::styled(
        hint,
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(
        hint_line,
        Rect { x: inner.x + 1, y, width: inner.width.saturating_sub(2), height: 1 },
    );
}
