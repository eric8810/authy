use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// A text input widget that supports masked (password) mode.
#[derive(Debug, Clone)]
pub struct TextInput {
    pub value: String,
    pub masked: bool,
    pub cursor_pos: usize,
}

impl TextInput {
    pub fn new(masked: bool) -> Self {
        Self {
            value: String::new(),
            masked,
            cursor_pos: 0,
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_pos = 0;
    }

    /// Handle a key event. Returns true if the event was consumed.
    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        // Ctrl+R toggles mask
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
            self.masked = !self.masked;
            return true;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_pos, c);
                self.cursor_pos += c.len_utf8();
                true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    // Find the previous char boundary
                    let prev = self.value[..self.cursor_pos]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.value.drain(prev..self.cursor_pos);
                    self.cursor_pos = prev;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.value.len() {
                    let next = self.value[self.cursor_pos..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.cursor_pos + i)
                        .unwrap_or(self.value.len());
                    self.value.drain(self.cursor_pos..next);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos = self.value[..self.cursor_pos]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.value.len() {
                    self.cursor_pos = self.value[self.cursor_pos..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.cursor_pos + i)
                        .unwrap_or(self.value.len());
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.value.len();
                true
            }
            _ => false,
        }
    }

    /// Get display text (masked or plain).
    pub fn display_text(&self) -> String {
        if self.masked {
            "\u{2022}".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    /// Get the display cursor position (in characters, for masked mode).
    pub fn display_cursor(&self) -> usize {
        if self.masked {
            self.value[..self.cursor_pos].chars().count()
        } else {
            self.cursor_pos
        }
    }
}

/// Render a text input field with an optional label.
pub fn render_input(
    frame: &mut Frame,
    area: Rect,
    input: &TextInput,
    label: &str,
    focused: bool,
) {
    let display = input.display_text();
    let text = format!("{}: {}", label, display);
    let style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let paragraph = Paragraph::new(Span::styled(text, style));
    frame.render_widget(paragraph, area);

    // Place cursor
    if focused {
        let cursor_x = area.x + label.len() as u16 + 2 + input.display_cursor() as u16;
        let cursor_y = area.y;
        if cursor_x < area.x + area.width {
            frame.set_cursor_position(Position::new(cursor_x, cursor_y));
        }
    }
}

/// A confirmation dialog overlay. Used in Phase 3+.
#[allow(dead_code)]
pub struct ConfirmDialog<'a> {
    pub title: &'a str,
    pub message: &'a str,
}

impl<'a> ConfirmDialog<'a> {
    #[allow(dead_code)]
    pub fn render(&self, frame: &mut Frame) {
        let area = centered_rect(50, 7, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let text = format!("{}\n\n[y] Yes  [n] No", self.message);
        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}

/// A popup overlay for showing content. Used in Phase 2+.
#[allow(dead_code)]
pub struct Popup<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub footer: &'a str,
}

impl<'a> Popup<'a> {
    #[allow(dead_code)]
    pub fn render(&self, frame: &mut Frame) {
        let height = (self.content.lines().count() as u16 + 4).min(frame.area().height - 2);
        let area = centered_rect(60, height, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };
        let footer_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };

        let paragraph = Paragraph::new(self.content);
        frame.render_widget(paragraph, content_area);

        let footer = Paragraph::new(Span::styled(
            self.footer,
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(footer, footer_area);
    }
}

/// Helper to create a centered rect of given percentage width and fixed height.
pub fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: popup_width.min(area.width),
        height: height.min(area.height),
    }
}
