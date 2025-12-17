//! Test utilities for TUI component testing.
//!
//! Provides a test terminal with fixed dimensions to render components
//! and compare the actual rendered text output.

use ratatui::{
    buffer::Buffer,
    crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
        MouseEvent, MouseEventKind,
    },
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    widgets::BorderType,
};
use url::Url;

use crate::{act::Act, thematize::Thematize};

/// A fixed-size test terminal for rendering components and comparing output.
pub struct TestTerminal {
    pub buffer: Buffer,
    pub area: Rect,
}

impl TestTerminal {
    /// Create a test terminal with fixed width and height.
    pub fn new(width: u16, height: u16) -> Self {
        let area = Rect::new(0, 0, width, height);
        let buffer = Buffer::empty(area);
        Self { buffer, area }
    }

    /// Reset the buffer to empty state.
    pub fn clear(&mut self) {
        self.buffer = Buffer::empty(self.area);
    }

    /// Get the rendered terminal output as a string.
    /// Returns exactly what would appear on screen - each row is a line.
    pub fn render_to_string(&self) -> String {
        let mut lines = Vec::new();
        for y in 0..self.area.height {
            let mut line = String::new();
            for x in 0..self.area.width {
                let cell = self.buffer.cell(Position::new(x, y)).unwrap();
                let symbol = cell.symbol();
                // Empty cells are represented as space
                if symbol.is_empty() {
                    line.push(' ');
                } else {
                    line.push_str(symbol);
                }
            }
            // Trim trailing spaces for cleaner comparison
            lines.push(line.trim_end().to_string());
        }
        // Remove trailing empty lines
        while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.pop();
        }
        lines.join("\n")
    }

    /// Check if a position has cursor styling (REVERSED modifier).
    pub fn has_cursor_at(&self, x: u16, y: u16) -> bool {
        if let Some(cell) = self.buffer.cell(Position::new(x, y)) {
            cell.style().add_modifier.contains(Modifier::REVERSED)
        } else {
            false
        }
    }

    /// Find the cursor position (x, y) and return the character at that position.
    /// Returns None if no cursor is found.
    pub fn find_cursor(&self) -> Option<(u16, u16, char)> {
        for y in 0..self.area.height {
            for x in 0..self.area.width {
                if let Some(cell) = self.buffer.cell(Position::new(x, y)) {
                    if cell.style().add_modifier.contains(Modifier::REVERSED) {
                        let ch = cell.symbol().chars().next().unwrap_or(' ');
                        return Some((x, y, ch));
                    }
                }
            }
        }
        None
    }
}

/// A simple theme for testing with predictable styling.
#[derive(Default, Clone)]
pub struct TestTheme {
    pub boxed: bool,
}

impl TestTheme {
    pub fn boxed() -> Self {
        Self { boxed: true }
    }

    pub fn unboxed() -> Self {
        Self { boxed: false }
    }
}

impl Thematize for TestTheme {
    fn cursor(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    fn cursor_cancelled(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    fn toast(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    fn popup(&self) -> Self {
        self.clone()
    }

    fn error_popup(&self) -> Self {
        self.clone()
    }

    fn style(&self) -> Style {
        Style::default()
    }

    fn style_dim(&self) -> Style {
        Style::default().fg(Color::DarkGray)
    }

    fn border_type(&self) -> BorderType {
        BorderType::Plain
    }

    fn button_focused(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    }

    fn button_notfocused(&self) -> Style {
        Style::default()
    }

    fn select_focused(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
    }

    fn select_active(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    fn select_inactive(&self) -> Style {
        Style::default().fg(Color::Gray)
    }

    fn boxed(&self) -> bool {
        self.boxed
    }
}

/// Test implementation of Act trait.
#[derive(Default)]
pub struct TestAct {
    pub esc_ignored: bool,
    pub left_ignored: bool,
    pub right_ignored: bool,
}

impl Act for TestAct {
    fn ignore_esc(&mut self) {
        self.esc_ignored = true;
    }
    fn ignore_left(&mut self) {
        self.left_ignored = true;
    }
    fn ignore_right(&mut self) {
        self.right_ignored = true;
    }
    fn is_esc_ignored(&self) -> bool {
        self.esc_ignored
    }
    fn merge(&mut self, other: Self) {
        self.esc_ignored |= other.esc_ignored;
        self.left_ignored |= other.left_ignored;
        self.right_ignored |= other.right_ignored;
    }
    fn copy_to_clipboard(&mut self, _: String, _: Option<Position>) {}
    fn open_url(&mut self, _: Url, _: Option<Position>) {}
}

// ============================================================================
// Event helpers for simulating keyboard and mouse input
// ============================================================================

/// Create a key press event for a character.
pub fn key(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create a key press with control modifier.
pub fn key_ctrl(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create a key press with alt modifier.
pub fn key_alt(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::ALT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create a backspace key event.
pub fn backspace() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Backspace,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create a backspace with alt modifier (word delete).
pub fn backspace_alt() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Backspace,
        modifiers: KeyModifiers::ALT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create left arrow key event.
pub fn left() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create left arrow with alt modifier (word left).
pub fn left_alt() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::ALT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create right arrow key event.
pub fn right() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create right arrow with alt modifier (word right).
pub fn right_alt() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::ALT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

/// Create a mouse click event at position.
pub fn mouse_click(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}
