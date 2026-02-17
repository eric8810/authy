mod auth;
mod widgets;

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::audit;
use crate::auth::context::{AuthContext, AuthMethod};
use crate::error::{AuthyError, Result};
use crate::vault::{self, Vault, VaultKey};

/// Which sidebar section is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Secrets,
    Policies,
    Sessions,
    Audit,
}

impl Section {
    pub fn label(&self) -> &str {
        match self {
            Section::Secrets => "Secrets",
            Section::Policies => "Policies",
            Section::Sessions => "Sessions",
            Section::Audit => "Audit",
        }
    }

    pub fn all() -> &'static [Section] {
        &[
            Section::Secrets,
            Section::Policies,
            Section::Sessions,
            Section::Audit,
        ]
    }

    pub fn next(&self) -> Section {
        match self {
            Section::Secrets => Section::Policies,
            Section::Policies => Section::Sessions,
            Section::Sessions => Section::Audit,
            Section::Audit => Section::Secrets,
        }
    }

    pub fn prev(&self) -> Section {
        match self {
            Section::Secrets => Section::Audit,
            Section::Policies => Section::Secrets,
            Section::Sessions => Section::Policies,
            Section::Audit => Section::Sessions,
        }
    }
}

/// Top-level screen state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Auth,
    Main,
}

/// Global input mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// The TUI application state.
pub struct TuiApp {
    // Auth state (None until authenticated)
    pub key: Option<VaultKey>,
    pub auth_ctx: Option<AuthContext>,
    pub vault: Option<Vault>,

    // UI state
    pub screen: Screen,
    pub section: Section,
    pub input_mode: InputMode,
    pub should_quit: bool,

    // Auth screen state
    pub auth_input: widgets::TextInput,
    pub auth_error: Option<String>,
    pub keyfile: Option<String>,

    // List cursor positions per section
    pub cursor: [usize; 4],
}

impl TuiApp {
    pub fn new(keyfile: Option<String>) -> Self {
        Self {
            key: None,
            auth_ctx: None,
            vault: None,
            screen: Screen::Auth,
            section: Section::Secrets,
            input_mode: InputMode::Normal,
            should_quit: false,
            auth_input: widgets::TextInput::new(true),
            auth_error: None,
            keyfile,
            cursor: [0; 4],
        }
    }

    /// Get current section cursor index.
    pub fn section_idx(&self) -> usize {
        match self.section {
            Section::Secrets => 0,
            Section::Policies => 1,
            Section::Sessions => 2,
            Section::Audit => 3,
        }
    }

    /// Get the current cursor position for the active section.
    pub fn cursor_pos(&self) -> usize {
        self.cursor[self.section_idx()]
    }

    /// Set the cursor position for the active section.
    pub fn set_cursor_pos(&mut self, pos: usize) {
        self.cursor[self.section_idx()] = pos;
    }

    /// Save the vault to disk. Used by write operations in Phase 3+.
    #[allow(dead_code)]
    pub fn save_vault(&self) -> Result<()> {
        if let (Some(v), Some(k)) = (&self.vault, &self.key) {
            vault::save_vault(v, k)?;
        }
        Ok(())
    }

    /// Get the auth actor name for audit logging. Used by audit in Phase 3+.
    #[allow(dead_code)]
    pub fn actor_name(&self) -> String {
        self.auth_ctx
            .as_ref()
            .map(|ctx| ctx.actor_name())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Derive the audit key from the current vault key. Used in Phase 3+.
    #[allow(dead_code)]
    pub fn audit_key(&self) -> Option<Vec<u8>> {
        self.key.as_ref().map(|k| {
            let material = audit::key_material(k);
            audit::derive_audit_key(&material)
        })
    }

    /// Log an audit event. Used in Phase 3+.
    #[allow(dead_code)]
    pub fn log_audit(
        &self,
        operation: &str,
        secret: Option<&str>,
        outcome: &str,
        detail: Option<&str>,
    ) -> Result<()> {
        if let Some(audit_key) = self.audit_key() {
            audit::log_event(
                &vault::audit_path(),
                operation,
                secret,
                &self.actor_name(),
                outcome,
                detail,
                &audit_key,
            )?;
        }
        Ok(())
    }

    /// Try to authenticate with the current auth input or keyfile.
    pub fn try_auth(&mut self) -> Result<()> {
        auth::try_authenticate(self)
    }
}

/// Main entry point — set up terminal, run event loop, restore terminal.
pub fn run(keyfile: Option<String>) -> Result<()> {
    // Check vault exists
    if !vault::is_initialized() {
        return Err(AuthyError::VaultNotInitialized);
    }

    let mut app = TuiApp::new(keyfile.clone());

    // If keyfile provided, try to auth immediately (skip auth screen)
    if keyfile.is_some() {
        match app.try_auth() {
            Ok(()) => app.screen = Screen::Main,
            Err(e) => {
                app.auth_error = Some(format!("{}", e));
            }
        }
    }

    // Set up terminal
    terminal::enable_raw_mode()
        .map_err(|e| AuthyError::Other(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| AuthyError::Other(format!("Failed to enter alternate screen: {}", e)))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| AuthyError::Other(format!("Failed to create terminal: {}", e)))?;

    // Run event loop (catch panics to restore terminal)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        event_loop(&mut terminal, &mut app)
    }));

    // Restore terminal
    terminal::disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    match result {
        Ok(inner) => inner,
        Err(_) => Err(AuthyError::Other("TUI panicked".into())),
    }
}

/// The main event loop.
fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        // Draw
        terminal
            .draw(|frame| draw(frame, app))
            .map_err(|e| AuthyError::Other(format!("Draw error: {}", e)))?;

        // Poll events
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)
            .map_err(|e| AuthyError::Other(format!("Event poll error: {}", e)))?
        {
            if let Event::Key(key_event) = event::read()
                .map_err(|e| AuthyError::Other(format!("Event read error: {}", e)))?
            {
                // Global quit: Ctrl+C
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && key_event.code == KeyCode::Char('c')
                {
                    app.should_quit = true;
                }

                match app.screen {
                    Screen::Auth => auth::handle_input(app, key_event),
                    Screen::Main => handle_main_input(app, key_event),
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle key input on the main dashboard screen.
fn handle_main_input(app: &mut TuiApp, key: event::KeyEvent) {
    if app.input_mode == InputMode::Editing {
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        // Section navigation
        KeyCode::Tab => {
            app.section = app.section.next();
        }
        KeyCode::BackTab => {
            app.section = app.section.prev();
        }
        KeyCode::Char('1') => app.section = Section::Secrets,
        KeyCode::Char('2') => app.section = Section::Policies,
        KeyCode::Char('3') => app.section = Section::Sessions,
        KeyCode::Char('4') => app.section = Section::Audit,
        // List navigation
        KeyCode::Char('j') | KeyCode::Down => {
            let max = list_len(app);
            if max > 0 {
                let pos = (app.cursor_pos() + 1).min(max - 1);
                app.set_cursor_pos(pos);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let pos = app.cursor_pos().saturating_sub(1);
            app.set_cursor_pos(pos);
        }
        _ => {}
    }
}

/// Get the number of items in the current section list.
fn list_len(app: &TuiApp) -> usize {
    let vault = match &app.vault {
        Some(v) => v,
        None => return 0,
    };
    match app.section {
        Section::Secrets => vault.secrets.len(),
        Section::Policies => vault.policies.len(),
        Section::Sessions => vault.sessions.len(),
        Section::Audit => 0, // Will be populated in Phase 5
    }
}

/// Root draw function — dispatches to auth or main screen.
fn draw(frame: &mut Frame, app: &TuiApp) {
    match app.screen {
        Screen::Auth => auth::draw(frame, app),
        Screen::Main => draw_main(frame, app),
    }
}

/// Draw the main dashboard layout.
fn draw_main(frame: &mut Frame, app: &TuiApp) {
    let area = frame.area();

    // Main layout: sidebar (14 cols) | content
    let h_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(40)])
        .split(area);

    // Sidebar + status bar
    let sidebar_area = h_layout[0];
    let content_area = h_layout[1];

    // Content area: main content + status bar
    let v_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(content_area);

    let main_area = v_layout[0];
    let status_area = v_layout[1];

    // Draw sidebar
    draw_sidebar(frame, sidebar_area, app);

    // Draw section content (placeholder for now, will be filled in later phases)
    draw_section_content(frame, main_area, app);

    // Draw status bar
    draw_status_bar(frame, status_area, app);
}

/// Draw the sidebar with section navigation.
fn draw_sidebar(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .title(" authy ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    for (i, section) in Section::all().iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let style = if *section == app.section {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default()
        };

        let label = format!(" {} {}", i + 1, section.label());
        let span = Span::styled(label, style);
        let paragraph = Paragraph::new(Line::from(span));
        let item_area = Rect {
            x: inner.x,
            y: inner.y + i as u16,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(paragraph, item_area);
    }
}

/// Draw the content area for the current section.
fn draw_section_content(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let vault = match &app.vault {
        Some(v) => v,
        None => {
            let p = Paragraph::new("No vault loaded.")
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(p, area);
            return;
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ({}) ", app.section.label(), list_len(app)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match app.section {
        Section::Secrets => {
            let items: Vec<String> = vault
                .secrets
                .iter()
                .map(|(name, entry)| {
                    format!(
                        " {:<20} v{:<4} {}",
                        name,
                        entry.metadata.version,
                        entry.metadata.tags.join(", ")
                    )
                })
                .collect();

            draw_list(frame, inner, &items, app.cursor_pos());
        }
        Section::Policies => {
            let items: Vec<String> = vault
                .policies
                .iter()
                .map(|(name, policy)| {
                    let desc = policy.description.as_deref().unwrap_or("");
                    format!(
                        " {:<20} allow:{} deny:{} {}",
                        name,
                        policy.allow.len(),
                        policy.deny.len(),
                        desc
                    )
                })
                .collect();

            draw_list(frame, inner, &items, app.cursor_pos());
        }
        Section::Sessions => {
            let now = chrono::Utc::now();
            let items: Vec<String> = vault
                .sessions
                .iter()
                .map(|s| {
                    let status = if s.revoked {
                        "revoked".to_string()
                    } else if now > s.expires_at {
                        "expired".to_string()
                    } else {
                        let remaining = s.expires_at - now;
                        format!("{}m left", remaining.num_minutes())
                    };
                    format!(
                        " {:<16} {:<16} {}",
                        s.id, s.scope, status
                    )
                })
                .collect();

            draw_list(frame, inner, &items, app.cursor_pos());
        }
        Section::Audit => {
            let p = Paragraph::new(" Audit log viewer (Phase 5)");
            frame.render_widget(p, inner);
        }
    }
}

/// Draw a simple selectable list.
fn draw_list(frame: &mut Frame, area: Rect, items: &[String], selected: usize) {
    for (i, item) in items.iter().enumerate() {
        if i as u16 >= area.height {
            break;
        }
        let style = if i == selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        let paragraph = Paragraph::new(Span::styled(item.as_str(), style));
        let item_area = Rect {
            x: area.x,
            y: area.y + i as u16,
            width: area.width,
            height: 1,
        };
        frame.render_widget(paragraph, item_area);
    }
}

/// Draw the status bar at the bottom.
fn draw_status_bar(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let vault_path = vault::vault_path().display().to_string();
    let auth_method = app
        .auth_ctx
        .as_ref()
        .map(|ctx| match &ctx.method {
            AuthMethod::Passphrase => "passphrase",
            AuthMethod::Keyfile => "keyfile",
            AuthMethod::SessionToken { .. } => "token",
        })
        .unwrap_or("none");

    let modified = app
        .vault
        .as_ref()
        .map(|v| v.modified_at.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default();

    // Hint bar with section-specific keys
    let hints = match app.section {
        Section::Secrets => "[s]tore [Enter]reveal [r]otate [d]elete [q]uit",
        Section::Policies => "[c]reate [e]dit [d]elete [t]est [q]uit",
        Section::Sessions => "[c]reate [r]evoke [R]evoke all [q]uit",
        Section::Audit => "[v]erify [/]filter [q]uit",
    };

    let top = Paragraph::new(Span::styled(
        format!(" {}  ", hints),
        Style::default().fg(Color::DarkGray),
    ));
    let bottom = Paragraph::new(Span::styled(
        format!(" vault: {}  auth: {}  modified: {}", vault_path, auth_method, modified),
        Style::default().fg(Color::DarkGray),
    ));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    frame.render_widget(top, rows[0]);
    frame.render_widget(bottom, rows[1]);
}
