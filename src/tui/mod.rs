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
use crate::policy::Policy;
use crate::session;
use crate::vault::{self, secret::SecretEntry, Vault, VaultKey};

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

/// The kind of popup overlay currently shown.
#[derive(Debug, Clone)]
pub enum PopupKind {
    /// Reveal a secret value.
    RevealSecret {
        name: String,
        value: String,
        masked: bool,
        auto_close_at: Instant,
    },
    /// Store a new secret form.
    StoreForm {
        name_input: widgets::TextInput,
        value_input: widgets::TextInput,
        tags_input: widgets::TextInput,
        focused_field: usize, // 0=name, 1=value, 2=tags
        error: Option<String>,
    },
    /// Rotate an existing secret (new value form).
    RotateForm {
        name: String,
        value_input: widgets::TextInput,
        error: Option<String>,
    },
    /// Confirm deletion dialog.
    ConfirmDelete {
        name: String,
    },
    /// Status message popup (auto-close).
    StatusMessage {
        message: String,
        is_error: bool,
        auto_close_at: Instant,
    },
    /// Create or edit a policy form.
    PolicyForm {
        name_input: widgets::TextInput,
        desc_input: widgets::TextInput,
        allow_input: widgets::TextInput,
        deny_input: widgets::TextInput,
        focused_field: usize, // 0=name, 1=desc, 2=allow, 3=deny
        error: Option<String>,
        editing: bool, // true if editing existing policy
    },
    /// Confirm policy deletion.
    ConfirmDeletePolicy {
        name: String,
    },
    /// Test a policy against a secret name.
    PolicyTest {
        scope: String,
        name_input: widgets::TextInput,
        result: Option<String>,
    },
    /// Create a session token form.
    SessionForm {
        scope_index: usize, // index into policies list
        policy_names: Vec<String>,
        ttl_input: widgets::TextInput,
        focused_field: usize, // 0=scope, 1=ttl
        error: Option<String>,
    },
    /// Show a newly created session token (one-time display).
    ShowToken {
        token: String,
        session_id: String,
        auto_close_at: Instant,
    },
    /// Confirm session revoke.
    ConfirmRevokeSession {
        session_id: String,
    },
    /// Confirm revoke all sessions.
    ConfirmRevokeAllSessions,
    /// Audit chain verification result.
    AuditVerifyResult {
        message: String,
        is_ok: bool,
    },
    /// Audit filter input.
    AuditFilter {
        filter_input: widgets::TextInput,
    },
    /// Help overlay.
    Help,
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

    // Popup overlay (rendered on top of main screen)
    pub popup: Option<PopupKind>,

    // Audit entries cache (loaded on demand)
    pub audit_entries: Vec<audit::AuditEntry>,
    pub audit_filter: String,
    pub audit_scroll: usize,
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
            popup: None,
            audit_entries: Vec::new(),
            audit_filter: String::new(),
            audit_scroll: 0,
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

    /// Save the vault to disk.
    pub fn save_vault(&self) -> Result<()> {
        if let (Some(v), Some(k)) = (&self.vault, &self.key) {
            vault::save_vault(v, k)?;
        }
        Ok(())
    }

    /// Get the auth actor name for audit logging.
    pub fn actor_name(&self) -> String {
        self.auth_ctx
            .as_ref()
            .map(|ctx| ctx.actor_name())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Derive the audit key from the current vault key.
    pub fn audit_key(&self) -> Option<Vec<u8>> {
        self.key.as_ref().map(|k| {
            let material = audit::key_material(k);
            audit::derive_audit_key(&material)
        })
    }

    /// Log an audit event.
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

    /// Load audit entries from disk.
    pub fn load_audit_entries(&mut self) {
        match audit::read_entries(&vault::audit_path()) {
            Ok(entries) => self.audit_entries = entries,
            Err(_) => self.audit_entries = Vec::new(),
        }
    }

    /// Get the filtered audit entries.
    pub fn filtered_audit_entries(&self) -> Vec<&audit::AuditEntry> {
        if self.audit_filter.is_empty() {
            self.audit_entries.iter().collect()
        } else {
            let filter = self.audit_filter.to_lowercase();
            self.audit_entries
                .iter()
                .filter(|e| {
                    e.operation.to_lowercase().contains(&filter)
                        || e.actor.to_lowercase().contains(&filter)
                        || e.outcome.to_lowercase().contains(&filter)
                        || e.secret.as_deref().unwrap_or("").to_lowercase().contains(&filter)
                })
                .collect()
        }
    }

    /// Derive session HMAC key from vault key.
    pub fn session_hmac_key(&self) -> Option<Vec<u8>> {
        self.key.as_ref().map(|k| {
            let material = audit::key_material(k);
            crate::vault::crypto::derive_key(&material, b"session-hmac", 32)
        })
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

        // Auto-close popup if timer expired
        let should_close = match &app.popup {
            Some(PopupKind::RevealSecret { auto_close_at, .. })
            | Some(PopupKind::StatusMessage { auto_close_at, .. })
            | Some(PopupKind::ShowToken { auto_close_at, .. }) => Instant::now() >= *auto_close_at,
            _ => false,
        };
        if should_close {
            app.popup = None;
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

/// Handle key input on the main dashboard screen.
fn handle_main_input(app: &mut TuiApp, key: event::KeyEvent) {
    // If a popup is active, handle popup input first
    if app.popup.is_some() {
        handle_popup_input(app, key);
        return;
    }

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
            if app.section == Section::Audit { app.load_audit_entries(); }
        }
        KeyCode::BackTab => {
            app.section = app.section.prev();
            if app.section == Section::Audit { app.load_audit_entries(); }
        }
        KeyCode::Char('1') => app.section = Section::Secrets,
        KeyCode::Char('2') => app.section = Section::Policies,
        KeyCode::Char('3') => app.section = Section::Sessions,
        KeyCode::Char('4') => {
            app.section = Section::Audit;
            app.load_audit_entries();
        }
        // List navigation
        KeyCode::Char('j') | KeyCode::Down => {
            if app.section == Section::Audit {
                let max = app.filtered_audit_entries().len();
                if app.audit_scroll + 1 < max {
                    app.audit_scroll += 1;
                }
            } else {
                let max = list_len(app);
                if max > 0 {
                    let pos = (app.cursor_pos() + 1).min(max - 1);
                    app.set_cursor_pos(pos);
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.section == Section::Audit {
                app.audit_scroll = app.audit_scroll.saturating_sub(1);
            } else {
                let pos = app.cursor_pos().saturating_sub(1);
                app.set_cursor_pos(pos);
            }
        }
        KeyCode::PageDown if app.section == Section::Audit => {
            let max = app.filtered_audit_entries().len();
            app.audit_scroll = (app.audit_scroll + 20).min(max.saturating_sub(1));
        }
        KeyCode::PageUp if app.section == Section::Audit => {
            app.audit_scroll = app.audit_scroll.saturating_sub(20);
        }
        // Reveal secret on Enter (Secrets section)
        KeyCode::Enter => {
            if app.section == Section::Secrets {
                open_reveal_popup(app);
            }
        }
        // Store new secret
        KeyCode::Char('s') if app.section == Section::Secrets => {
            app.popup = Some(PopupKind::StoreForm {
                name_input: widgets::TextInput::new(false),
                value_input: widgets::TextInput::new(true),
                tags_input: widgets::TextInput::new(false),
                focused_field: 0,
                error: None,
            });
        }
        // Rotate secret
        KeyCode::Char('r') if app.section == Section::Secrets => {
            if let Some(vault) = &app.vault {
                if let Some((name, _)) = vault.secrets.iter().nth(app.cursor_pos()) {
                    app.popup = Some(PopupKind::RotateForm {
                        name: name.clone(),
                        value_input: widgets::TextInput::new(true),
                        error: None,
                    });
                }
            }
        }
        // Delete secret
        KeyCode::Char('d') if app.section == Section::Secrets => {
            if let Some(vault) = &app.vault {
                if let Some((name, _)) = vault.secrets.iter().nth(app.cursor_pos()) {
                    app.popup = Some(PopupKind::ConfirmDelete {
                        name: name.clone(),
                    });
                }
            }
        }
        // Create policy
        KeyCode::Char('c') if app.section == Section::Policies => {
            app.popup = Some(PopupKind::PolicyForm {
                name_input: widgets::TextInput::new(false),
                desc_input: widgets::TextInput::new(false),
                allow_input: widgets::TextInput::new(false),
                deny_input: widgets::TextInput::new(false),
                focused_field: 0,
                error: None,
                editing: false,
            });
        }
        // Edit policy
        KeyCode::Char('e') if app.section == Section::Policies => {
            if let Some(vault) = &app.vault {
                if let Some((name, policy)) = vault.policies.iter().nth(app.cursor_pos()) {
                    let mut name_input = widgets::TextInput::new(false);
                    name_input.value = name.clone();
                    name_input.cursor_pos = name.len();
                    let mut desc_input = widgets::TextInput::new(false);
                    desc_input.value = policy.description.clone().unwrap_or_default();
                    desc_input.cursor_pos = desc_input.value.len();
                    let mut allow_input = widgets::TextInput::new(false);
                    allow_input.value = policy.allow.join(", ");
                    allow_input.cursor_pos = allow_input.value.len();
                    let mut deny_input = widgets::TextInput::new(false);
                    deny_input.value = policy.deny.join(", ");
                    deny_input.cursor_pos = deny_input.value.len();
                    app.popup = Some(PopupKind::PolicyForm {
                        name_input,
                        desc_input,
                        allow_input,
                        deny_input,
                        focused_field: 2, // Focus allow patterns
                        error: None,
                        editing: true,
                    });
                }
            }
        }
        // Delete policy
        KeyCode::Char('d') if app.section == Section::Policies => {
            if let Some(vault) = &app.vault {
                if let Some((name, _)) = vault.policies.iter().nth(app.cursor_pos()) {
                    app.popup = Some(PopupKind::ConfirmDeletePolicy {
                        name: name.clone(),
                    });
                }
            }
        }
        // Test policy
        KeyCode::Char('t') if app.section == Section::Policies => {
            if let Some(vault) = &app.vault {
                if let Some((name, _)) = vault.policies.iter().nth(app.cursor_pos()) {
                    app.popup = Some(PopupKind::PolicyTest {
                        scope: name.clone(),
                        name_input: widgets::TextInput::new(false),
                        result: None,
                    });
                }
            }
        }
        // Create session
        KeyCode::Char('c') if app.section == Section::Sessions => {
            if let Some(vault) = &app.vault {
                let policy_names: Vec<String> = vault.policies.keys().cloned().collect();
                if policy_names.is_empty() {
                    app.popup = Some(PopupKind::StatusMessage {
                        message: "No policies defined. Create a policy first.".into(),
                        is_error: true,
                        auto_close_at: Instant::now() + Duration::from_secs(3),
                    });
                } else {
                    let mut ttl_input = widgets::TextInput::new(false);
                    ttl_input.value = "1h".to_string();
                    ttl_input.cursor_pos = 2;
                    app.popup = Some(PopupKind::SessionForm {
                        scope_index: 0,
                        policy_names,
                        ttl_input,
                        focused_field: 0,
                        error: None,
                    });
                }
            }
        }
        // Revoke session
        KeyCode::Char('r') if app.section == Section::Sessions => {
            if let Some(vault) = &app.vault {
                if let Some(s) = vault.sessions.get(app.cursor_pos()) {
                    if !s.revoked {
                        app.popup = Some(PopupKind::ConfirmRevokeSession {
                            session_id: s.id.clone(),
                        });
                    }
                }
            }
        }
        // Revoke all sessions
        KeyCode::Char('R') if app.section == Section::Sessions => {
            app.popup = Some(PopupKind::ConfirmRevokeAllSessions);
        }
        // Audit: verify chain
        KeyCode::Char('v') if app.section == Section::Audit => {
            app.load_audit_entries();
            if let Some(audit_key) = app.audit_key() {
                match audit::verify_chain(&vault::audit_path(), &audit_key) {
                    Ok((count, _)) => {
                        app.popup = Some(PopupKind::AuditVerifyResult {
                            message: format!("Chain valid ({} entries)", count),
                            is_ok: true,
                        });
                    }
                    Err(e) => {
                        app.popup = Some(PopupKind::AuditVerifyResult {
                            message: format!("{}", e),
                            is_ok: false,
                        });
                    }
                }
            }
        }
        // Audit: filter
        KeyCode::Char('/') if app.section == Section::Audit => {
            let mut filter_input = widgets::TextInput::new(false);
            filter_input.value = app.audit_filter.clone();
            filter_input.cursor_pos = filter_input.value.len();
            app.popup = Some(PopupKind::AuditFilter { filter_input });
        }
        // Help overlay
        KeyCode::Char('?') => {
            app.popup = Some(PopupKind::Help);
        }
        _ => {}
    }
}

/// Open the reveal-secret popup for the currently selected secret.
fn open_reveal_popup(app: &mut TuiApp) {
    let vault = match &app.vault {
        Some(v) => v,
        None => return,
    };

    let pos = app.cursor_pos();
    if let Some((name, entry)) = vault.secrets.iter().nth(pos) {
        app.popup = Some(PopupKind::RevealSecret {
            name: name.clone(),
            value: entry.value.clone(),
            masked: true,
            auto_close_at: Instant::now() + Duration::from_secs(30),
        });
    }
}

/// Handle key input when a popup is active.
fn handle_popup_input(app: &mut TuiApp, key: event::KeyEvent) {
    // Take ownership of the popup temporarily
    let popup = match app.popup.take() {
        Some(p) => p,
        None => return,
    };

    match popup {
        PopupKind::RevealSecret { mut masked, name, value, auto_close_at } => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    // Close popup (already taken)
                }
                _ => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
                        masked = !masked;
                    }
                    app.popup = Some(PopupKind::RevealSecret { name, value, masked, auto_close_at });
                }
            }
        }
        PopupKind::StoreForm { mut name_input, mut value_input, mut tags_input, mut focused_field, .. } => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel
                }
                KeyCode::Tab => {
                    focused_field = (focused_field + 1) % 3;
                    app.popup = Some(PopupKind::StoreForm { name_input, value_input, tags_input, focused_field, error: None });
                }
                KeyCode::BackTab => {
                    focused_field = if focused_field == 0 { 2 } else { focused_field - 1 };
                    app.popup = Some(PopupKind::StoreForm { name_input, value_input, tags_input, focused_field, error: None });
                }
                KeyCode::Enter => {
                    // Submit the form
                    let name = name_input.value.trim().to_string();
                    let value = value_input.value.clone();
                    let tags_str = tags_input.value.trim().to_string();

                    if name.is_empty() {
                        app.popup = Some(PopupKind::StoreForm {
                            name_input, value_input, tags_input, focused_field,
                            error: Some("Name cannot be empty".into()),
                        });
                        return;
                    }
                    if value.is_empty() {
                        app.popup = Some(PopupKind::StoreForm {
                            name_input, value_input, tags_input, focused_field,
                            error: Some("Value cannot be empty".into()),
                        });
                        return;
                    }

                    if let Some(ref mut vault) = app.vault {
                        if vault.secrets.contains_key(&name) {
                            app.popup = Some(PopupKind::StoreForm {
                                name_input, value_input, tags_input, focused_field,
                                error: Some(format!("Secret '{}' already exists", name)),
                            });
                            return;
                        }

                        let mut entry = SecretEntry::new(value);
                        if !tags_str.is_empty() {
                            entry.metadata.tags = tags_str.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
                        }
                        vault.secrets.insert(name.clone(), entry);
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("store", Some(&name), "success", None);

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Secret '{}' stored.", name),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    // Forward key to the focused input
                    match focused_field {
                        0 => { name_input.handle_input(key); }
                        1 => { value_input.handle_input(key); }
                        2 => { tags_input.handle_input(key); }
                        _ => {}
                    }
                    app.popup = Some(PopupKind::StoreForm { name_input, value_input, tags_input, focused_field, error: None });
                }
            }
        }
        PopupKind::RotateForm { name, mut value_input, .. } => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel
                }
                KeyCode::Enter => {
                    let new_value = value_input.value.clone();
                    if new_value.is_empty() {
                        app.popup = Some(PopupKind::RotateForm {
                            name, value_input,
                            error: Some("Value cannot be empty".into()),
                        });
                        return;
                    }

                    if let Some(ref mut vault) = app.vault {
                        if let Some(entry) = vault.secrets.get_mut(&name) {
                            entry.value = new_value;
                            entry.metadata.bump_version();
                            vault.touch();
                        }
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("rotate", Some(&name), "success", None);

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Secret '{}' rotated.", name),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    value_input.handle_input(key);
                    app.popup = Some(PopupKind::RotateForm { name, value_input, error: None });
                }
            }
        }
        PopupKind::ConfirmDelete { name } => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(ref mut vault) = app.vault {
                        vault.secrets.remove(&name);
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("remove", Some(&name), "success", None);

                    // Adjust cursor if it was at the end
                    let len = app.vault.as_ref().map(|v| v.secrets.len()).unwrap_or(0);
                    if app.cursor_pos() >= len && len > 0 {
                        app.set_cursor_pos(len - 1);
                    }

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Secret '{}' deleted.", name),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    // Any other key cancels
                }
            }
        }
        PopupKind::PolicyForm { mut name_input, mut desc_input, mut allow_input, mut deny_input, mut focused_field, editing, .. } => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel
                }
                KeyCode::Tab => {
                    focused_field = (focused_field + 1) % 4;
                    app.popup = Some(PopupKind::PolicyForm { name_input, desc_input, allow_input, deny_input, focused_field, error: None, editing });
                }
                KeyCode::BackTab => {
                    focused_field = if focused_field == 0 { 3 } else { focused_field - 1 };
                    app.popup = Some(PopupKind::PolicyForm { name_input, desc_input, allow_input, deny_input, focused_field, error: None, editing });
                }
                KeyCode::Enter => {
                    let name = name_input.value.trim().to_string();
                    let desc = desc_input.value.trim().to_string();
                    let allow_str = allow_input.value.trim().to_string();
                    let deny_str = deny_input.value.trim().to_string();

                    if name.is_empty() {
                        app.popup = Some(PopupKind::PolicyForm {
                            name_input, desc_input, allow_input, deny_input, focused_field,
                            error: Some("Name cannot be empty".into()), editing,
                        });
                        return;
                    }
                    if allow_str.is_empty() {
                        app.popup = Some(PopupKind::PolicyForm {
                            name_input, desc_input, allow_input, deny_input, focused_field,
                            error: Some("At least one allow pattern required".into()), editing,
                        });
                        return;
                    }

                    let allow: Vec<String> = allow_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    let deny: Vec<String> = deny_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

                    if let Some(ref mut vault) = app.vault {
                        if editing {
                            if let Some(policy) = vault.policies.get_mut(&name) {
                                policy.allow = allow;
                                policy.deny = deny;
                                policy.description = if desc.is_empty() { None } else { Some(desc) };
                                policy.modified_at = chrono::Utc::now();
                            }
                        } else {
                            if vault.policies.contains_key(&name) {
                                app.popup = Some(PopupKind::PolicyForm {
                                    name_input, desc_input, allow_input, deny_input, focused_field,
                                    error: Some(format!("Policy '{}' already exists", name)), editing,
                                });
                                return;
                            }
                            let mut policy = Policy::new(name.clone(), allow, deny);
                            policy.description = if desc.is_empty() { None } else { Some(desc) };
                            vault.policies.insert(name.clone(), policy);
                        }
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let op = if editing { "policy.update" } else { "policy.create" };
                    let _ = app.log_audit(op, None, "success", Some(&format!("policy={}", name)));

                    let verb = if editing { "updated" } else { "created" };
                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Policy '{}' {}.", name, verb),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    match focused_field {
                        0 => { name_input.handle_input(key); }
                        1 => { desc_input.handle_input(key); }
                        2 => { allow_input.handle_input(key); }
                        3 => { deny_input.handle_input(key); }
                        _ => {}
                    }
                    app.popup = Some(PopupKind::PolicyForm { name_input, desc_input, allow_input, deny_input, focused_field, error: None, editing });
                }
            }
        }
        PopupKind::ConfirmDeletePolicy { name } => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(ref mut vault) = app.vault {
                        vault.policies.remove(&name);
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("policy.remove", None, "success", Some(&format!("policy={}", name)));

                    let len = app.vault.as_ref().map(|v| v.policies.len()).unwrap_or(0);
                    if app.cursor_pos() >= len && len > 0 {
                        app.set_cursor_pos(len - 1);
                    }

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Policy '{}' deleted.", name),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    // Cancel
                }
            }
        }
        PopupKind::PolicyTest { scope, mut name_input, .. } => {
            match key.code {
                KeyCode::Esc => {
                    // Close
                }
                KeyCode::Enter => {
                    let secret_name = name_input.value.trim().to_string();
                    if secret_name.is_empty() {
                        app.popup = Some(PopupKind::PolicyTest {
                            scope, name_input,
                            result: Some("Enter a secret name to test".into()),
                        });
                        return;
                    }

                    let result = if let Some(vault) = &app.vault {
                        if let Some(policy) = vault.policies.get(&scope) {
                            match policy.can_read(&secret_name) {
                                Ok(true) => format!("ALLOWED: '{}' can read '{}'", scope, secret_name),
                                Ok(false) => format!("DENIED: '{}' cannot read '{}'", scope, secret_name),
                                Err(e) => format!("Error: {}", e),
                            }
                        } else {
                            "Policy not found".into()
                        }
                    } else {
                        "No vault".into()
                    };

                    app.popup = Some(PopupKind::PolicyTest {
                        scope, name_input,
                        result: Some(result),
                    });
                }
                _ => {
                    name_input.handle_input(key);
                    app.popup = Some(PopupKind::PolicyTest { scope, name_input, result: None });
                }
            }
        }
        PopupKind::SessionForm { mut scope_index, policy_names, mut ttl_input, mut focused_field, .. } => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel
                }
                KeyCode::Tab => {
                    focused_field = (focused_field + 1) % 2;
                    app.popup = Some(PopupKind::SessionForm { scope_index, policy_names, ttl_input, focused_field, error: None });
                }
                KeyCode::BackTab => {
                    focused_field = if focused_field == 0 { 1 } else { 0 };
                    app.popup = Some(PopupKind::SessionForm { scope_index, policy_names, ttl_input, focused_field, error: None });
                }
                // Arrow keys to cycle scope when scope field is focused
                KeyCode::Up | KeyCode::Left if focused_field == 0 => {
                    scope_index = if scope_index == 0 { policy_names.len().saturating_sub(1) } else { scope_index - 1 };
                    app.popup = Some(PopupKind::SessionForm { scope_index, policy_names, ttl_input, focused_field, error: None });
                }
                KeyCode::Down | KeyCode::Right if focused_field == 0 => {
                    scope_index = (scope_index + 1) % policy_names.len().max(1);
                    app.popup = Some(PopupKind::SessionForm { scope_index, policy_names, ttl_input, focused_field, error: None });
                }
                KeyCode::Enter => {
                    let scope = policy_names.get(scope_index).cloned().unwrap_or_default();
                    let ttl_str = ttl_input.value.trim().to_string();

                    if scope.is_empty() {
                        app.popup = Some(PopupKind::SessionForm {
                            scope_index, policy_names, ttl_input, focused_field,
                            error: Some("No scope selected".into()),
                        });
                        return;
                    }

                    let duration = match session::parse_ttl(&ttl_str) {
                        Ok(d) => d,
                        Err(e) => {
                            app.popup = Some(PopupKind::SessionForm {
                                scope_index, policy_names, ttl_input, focused_field,
                                error: Some(format!("Invalid TTL: {}", e)),
                            });
                            return;
                        }
                    };

                    let hmac_key = match app.session_hmac_key() {
                        Some(k) => k,
                        None => {
                            app.popup = Some(PopupKind::StatusMessage {
                                message: "No vault key available".into(),
                                is_error: true,
                                auto_close_at: Instant::now() + Duration::from_secs(3),
                            });
                            return;
                        }
                    };

                    let now = chrono::Utc::now();
                    let expires_at = now + duration;
                    let (token, token_hmac) = session::generate_token(&hmac_key);
                    let session_id = session::generate_session_id();

                    let record = session::SessionRecord {
                        id: session_id.clone(),
                        scope: scope.clone(),
                        token_hmac,
                        created_at: now,
                        expires_at,
                        revoked: false,
                        label: None,
                        run_only: false,
                    };

                    if let Some(ref mut vault) = app.vault {
                        vault.sessions.push(record);
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit(
                        "session.create", None, "success",
                        Some(&format!("session={}, scope={}, ttl={}", session_id, scope, ttl_str)),
                    );

                    app.popup = Some(PopupKind::ShowToken {
                        token,
                        session_id,
                        auto_close_at: Instant::now() + Duration::from_secs(60),
                    });
                }
                _ => {
                    if focused_field == 1 {
                        ttl_input.handle_input(key);
                    }
                    app.popup = Some(PopupKind::SessionForm { scope_index, policy_names, ttl_input, focused_field, error: None });
                }
            }
        }
        PopupKind::ShowToken { .. } => {
            // Any key closes
        }
        PopupKind::ConfirmRevokeSession { session_id } => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    if let Some(ref mut vault) = app.vault {
                        if let Some(s) = vault.sessions.iter_mut().find(|s| s.id == session_id) {
                            s.revoked = true;
                        }
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("session.revoke", None, "success", Some(&format!("session={}", session_id)));

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("Session '{}' revoked.", session_id),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    // Cancel
                }
            }
        }
        PopupKind::ConfirmRevokeAllSessions => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let mut count = 0;
                    if let Some(ref mut vault) = app.vault {
                        for s in vault.sessions.iter_mut() {
                            if !s.revoked {
                                s.revoked = true;
                                count += 1;
                            }
                        }
                        vault.touch();
                    }

                    if let Err(e) = app.save_vault() {
                        app.popup = Some(PopupKind::StatusMessage {
                            message: format!("Save failed: {}", e),
                            is_error: true,
                            auto_close_at: Instant::now() + Duration::from_secs(3),
                        });
                        return;
                    }

                    let _ = app.log_audit("session.revoke_all", None, "success", Some(&format!("count={}", count)));

                    app.popup = Some(PopupKind::StatusMessage {
                        message: format!("{} session(s) revoked.", count),
                        is_error: false,
                        auto_close_at: Instant::now() + Duration::from_secs(2),
                    });
                }
                _ => {
                    // Cancel
                }
            }
        }
        PopupKind::AuditVerifyResult { .. } => {
            // Any key closes
        }
        PopupKind::AuditFilter { mut filter_input } => {
            match key.code {
                KeyCode::Esc => {
                    // Cancel, keep old filter
                }
                KeyCode::Enter => {
                    app.audit_filter = filter_input.value.trim().to_string();
                    app.audit_scroll = 0;
                }
                _ => {
                    filter_input.handle_input(key);
                    app.popup = Some(PopupKind::AuditFilter { filter_input });
                }
            }
        }
        PopupKind::Help => {
            // Any key closes help
        }
        PopupKind::StatusMessage { .. } => {
            // Any key closes the status message
        }
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
        Section::Audit => app.filtered_audit_entries().len(),
    }
}

/// Root draw function — dispatches to auth or main screen.
fn draw(frame: &mut Frame, app: &TuiApp) {
    match app.screen {
        Screen::Auth => auth::draw(frame, app),
        Screen::Main => {
            draw_main(frame, app);
            // Draw popup overlay on top if active
            if let Some(ref popup) = app.popup {
                draw_popup(frame, popup);
            }
        }
    }
}

/// Draw a popup overlay.
fn draw_popup(frame: &mut Frame, popup: &PopupKind) {
    match popup {
        PopupKind::RevealSecret {
            name,
            value,
            masked,
            auto_close_at,
        } => {
            let display_value = if *masked {
                "\u{2022}".repeat(value.len().min(40))
            } else {
                value.clone()
            };

            let remaining = auto_close_at
                .checked_duration_since(Instant::now())
                .unwrap_or_default();

            let title = name.to_string();
            let footer = format!(
                "[Esc] close  [Ctrl+R] {}  auto-close: {}s",
                if *masked { "reveal" } else { "mask" },
                remaining.as_secs()
            );

            let p = widgets::Popup {
                title: &title,
                content: &display_value,
                footer: &footer,
            };
            p.render(frame);
        }
        PopupKind::StoreForm {
            name_input,
            value_input,
            tags_input,
            focused_field,
            error,
        } => {
            let area = widgets::centered_rect(60, 12, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Store new secret ")
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut y = inner.y;
            let w = inner.width.saturating_sub(2);
            let x = inner.x + 1;

            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, name_input, "Name ", *focused_field == 0);
            y += 1;
            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, value_input, "Value", *focused_field == 1);
            y += 1;
            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, tags_input, "Tags ", *focused_field == 2);
            y += 2;

            if let Some(err) = error {
                let p = Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
                frame.render_widget(p, Rect { x, y, width: w, height: 1 });
                y += 1;
            }

            let hint = Paragraph::new(Span::styled(
                "[Tab] next field  [Enter] save  [Ctrl+R] toggle mask  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(hint, Rect { x, y, width: w, height: 1 });
        }
        PopupKind::RotateForm {
            name,
            value_input,
            error,
        } => {
            let area = widgets::centered_rect(60, 9, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rotate: {} ", name))
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut y = inner.y;
            let w = inner.width.saturating_sub(2);
            let x = inner.x + 1;

            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, value_input, "New value", true);
            y += 2;

            if let Some(err) = error {
                let p = Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
                frame.render_widget(p, Rect { x, y, width: w, height: 1 });
                y += 1;
            }

            let hint = Paragraph::new(Span::styled(
                "[Enter] save  [Ctrl+R] toggle mask  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(hint, Rect { x, y, width: w, height: 1 });
        }
        PopupKind::ConfirmDelete { name } => {
            let dialog = widgets::ConfirmDialog {
                title: "Delete secret",
                message: &format!("Delete '{}'?", name),
            };
            dialog.render(frame);
        }
        PopupKind::PolicyForm {
            name_input,
            desc_input,
            allow_input,
            deny_input,
            focused_field,
            error,
            editing,
        } => {
            let title = if *editing { " Edit policy " } else { " Create policy " };
            let area = widgets::centered_rect(60, 14, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut y = inner.y;
            let w = inner.width.saturating_sub(2);
            let x = inner.x + 1;

            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, name_input, "Name ", *focused_field == 0);
            y += 1;
            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, desc_input, "Desc ", *focused_field == 1);
            y += 1;
            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, allow_input, "Allow", *focused_field == 2);
            y += 1;
            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, deny_input, "Deny ", *focused_field == 3);
            y += 2;

            if let Some(err) = error {
                let p = Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
                frame.render_widget(p, Rect { x, y, width: w, height: 1 });
                y += 1;
            }

            let hint_text = if *editing {
                "[Tab] next field  [Enter] save  [Esc] cancel  (comma-separated patterns)"
            } else {
                "[Tab] next field  [Enter] create  [Esc] cancel  (comma-separated patterns)"
            };
            let hint = Paragraph::new(Span::styled(hint_text, Style::default().fg(Color::DarkGray)));
            frame.render_widget(hint, Rect { x, y, width: w, height: 1 });
        }
        PopupKind::ConfirmDeletePolicy { name } => {
            let dialog = widgets::ConfirmDialog {
                title: "Delete policy",
                message: &format!("Delete policy '{}'?", name),
            };
            dialog.render(frame);
        }
        PopupKind::PolicyTest {
            scope,
            name_input,
            result,
        } => {
            let area = widgets::centered_rect(60, 9, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Test policy: {} ", scope))
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut y = inner.y;
            let w = inner.width.saturating_sub(2);
            let x = inner.x + 1;

            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, name_input, "Secret name", true);
            y += 2;

            if let Some(res) = result {
                let color = if res.starts_with("ALLOWED") {
                    Color::Green
                } else if res.starts_with("DENIED") {
                    Color::Yellow
                } else {
                    Color::Red
                };
                let p = Paragraph::new(Span::styled(res.as_str(), Style::default().fg(color)));
                frame.render_widget(p, Rect { x, y, width: w, height: 1 });
                y += 1;
            }

            let hint = Paragraph::new(Span::styled(
                "[Enter] test  [Esc] close",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(hint, Rect { x, y, width: w, height: 1 });
        }
        PopupKind::SessionForm {
            scope_index,
            policy_names,
            ttl_input,
            focused_field,
            error,
        } => {
            let area = widgets::centered_rect(60, 10, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Create session ")
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let mut y = inner.y;
            let w = inner.width.saturating_sub(2);
            let x = inner.x + 1;

            // Scope selector
            let scope_name = policy_names.get(*scope_index).cloned().unwrap_or_default();
            let scope_style = if *focused_field == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            let scope_text = format!("Scope: < {} >  ({}/{})", scope_name, scope_index + 1, policy_names.len());
            let p = Paragraph::new(Span::styled(scope_text, scope_style));
            frame.render_widget(p, Rect { x, y, width: w, height: 1 });
            y += 1;

            widgets::render_input(frame, Rect { x, y, width: w, height: 1 }, ttl_input, "TTL  ", *focused_field == 1);
            y += 2;

            if let Some(err) = error {
                let p = Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
                frame.render_widget(p, Rect { x, y, width: w, height: 1 });
                y += 1;
            }

            let hint = Paragraph::new(Span::styled(
                "[Tab] next  [</>] change scope  [Enter] create  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(hint, Rect { x, y, width: w, height: 1 });
        }
        PopupKind::ShowToken {
            token,
            session_id,
            auto_close_at,
        } => {
            let remaining = auto_close_at
                .checked_duration_since(Instant::now())
                .unwrap_or_default();
            let title = format!("Session: {}", session_id);
            let footer = format!("[any key] close  auto-close: {}s  (copy this token now!)", remaining.as_secs());
            let p = widgets::Popup {
                title: &title,
                content: token,
                footer: &footer,
            };
            p.render(frame);
        }
        PopupKind::ConfirmRevokeSession { session_id } => {
            let dialog = widgets::ConfirmDialog {
                title: "Revoke session",
                message: &format!("Revoke session '{}'?", session_id),
            };
            dialog.render(frame);
        }
        PopupKind::ConfirmRevokeAllSessions => {
            let dialog = widgets::ConfirmDialog {
                title: "Revoke all",
                message: "Revoke ALL active sessions?",
            };
            dialog.render(frame);
        }
        PopupKind::AuditVerifyResult { message, is_ok } => {
            let color = if *is_ok { Color::Green } else { Color::Red };
            let area = widgets::centered_rect(50, 5, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Chain Verification ")
                .border_style(Style::default().fg(color));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let p = Paragraph::new(message.as_str()).alignment(Alignment::Center);
            frame.render_widget(p, inner);
        }
        PopupKind::AuditFilter { filter_input } => {
            let area = widgets::centered_rect(50, 6, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Filter audit log ")
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let x = inner.x + 1;
            let w = inner.width.saturating_sub(2);
            widgets::render_input(frame, Rect { x, y: inner.y, width: w, height: 1 }, filter_input, "Filter", true);

            let hint = Paragraph::new(Span::styled(
                "[Enter] apply  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(hint, Rect { x, y: inner.y + 2, width: w, height: 1 });
        }
        PopupKind::Help => {
            let help_text = "\
Tab/1-4    Switch section
j/k ↑/↓    Navigate list
Enter      Select / reveal

Secrets:
  s        Store new secret
  r        Rotate secret
  d        Delete secret

Policies:
  c        Create policy
  e        Edit policy
  d        Delete policy
  t        Test policy

Sessions:
  c        Create session
  r        Revoke session
  R        Revoke all

Audit:
  /        Filter log
  v        Verify chain

Ctrl+R     Toggle mask
Esc/q      Close / quit
?          This help";

            let area = widgets::centered_rect(50, 30.min(frame.area().height.saturating_sub(2)), frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Key Bindings ")
                .border_style(Style::default().fg(Color::Cyan));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let p = Paragraph::new(help_text);
            frame.render_widget(p, inner);
        }
        PopupKind::StatusMessage {
            message,
            is_error,
            ..
        } => {
            let color = if *is_error { Color::Red } else { Color::Green };
            let area = widgets::centered_rect(50, 5, frame.area());
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let p = Paragraph::new(message.as_str()).alignment(Alignment::Center);
            frame.render_widget(p, inner);
        }
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
            // Header
            let header = format!(
                " {:<20} {:<12} {:<12} {:<5} {}",
                "NAME", "CREATED", "MODIFIED", "VER", "TAGS"
            );
            let header_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 };
            let header_p = Paragraph::new(Span::styled(header, Style::default().add_modifier(Modifier::BOLD)));
            frame.render_widget(header_p, header_area);

            let list_area = Rect {
                x: inner.x,
                y: inner.y + 1,
                width: inner.width,
                height: inner.height.saturating_sub(1),
            };

            let items: Vec<String> = vault
                .secrets
                .iter()
                .map(|(name, entry)| {
                    format!(
                        " {:<20} {:<12} {:<12} {:<5} {}",
                        name,
                        entry.metadata.created_at.format("%Y-%m-%d"),
                        entry.metadata.modified_at.format("%Y-%m-%d"),
                        format!("v{}", entry.metadata.version),
                        entry.metadata.tags.join(", ")
                    )
                })
                .collect();

            draw_list(frame, list_area, &items, app.cursor_pos());
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
            let filter_info = if app.audit_filter.is_empty() {
                String::new()
            } else {
                format!("  filter: \"{}\"", app.audit_filter)
            };

            let header = format!(
                " {:<20} {:<12} {:<16} {:<8} {}",
                "TIMESTAMP", "OPERATION", "SECRET", "STATUS", filter_info
            );
            let header_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: 1 };
            let header_p = Paragraph::new(Span::styled(header, Style::default().add_modifier(Modifier::BOLD)));
            frame.render_widget(header_p, header_area);

            let list_area = Rect {
                x: inner.x,
                y: inner.y + 1,
                width: inner.width,
                height: inner.height.saturating_sub(1),
            };

            let filtered = app.filtered_audit_entries();
            let items: Vec<String> = filtered
                .iter()
                .rev() // Most recent first
                .map(|e| {
                    let secret = e.secret.as_deref().unwrap_or("-");
                    format!(
                        " {:<20} {:<12} {:<16} {}",
                        e.timestamp.format("%m-%d %H:%M:%S"),
                        e.operation,
                        secret,
                        e.outcome,
                    )
                })
                .collect();

            // Apply scroll offset
            let visible: Vec<String> = items
                .iter()
                .skip(app.audit_scroll)
                .cloned()
                .collect();

            draw_list(frame, list_area, &visible, 0);
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
