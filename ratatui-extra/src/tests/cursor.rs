use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::widgets::cursor::Cursor;

fn key_down() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_up() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ============================================================================
// Basic cursor tests
// ============================================================================

#[test]
fn cursor_initial_position() {
    let cursor = Cursor::new(0);
    assert_eq!(cursor.current, 0);

    let cursor = Cursor::new(5);
    assert_eq!(cursor.current, 5);
}

#[test]
fn cursor_reset() {
    let mut cursor = Cursor::new(5);
    cursor.reset();
    assert_eq!(cursor.current, 0);
}

// ============================================================================
// Down navigation tests
// ============================================================================

#[test]
fn cursor_move_down() {
    let mut cursor = Cursor::new(0);
    cursor.handle(Some(&key_down()), 5);
    assert_eq!(cursor.current, 1);
}

#[test]
fn cursor_move_down_wraps_at_end() {
    let mut cursor = Cursor::new(4);
    cursor.handle(Some(&key_down()), 5);
    // Should wrap to 0
    assert_eq!(cursor.current, 0);
}

#[test]
fn cursor_move_down_multiple() {
    let mut cursor = Cursor::new(0);
    cursor.handle(Some(&key_down()), 5);
    cursor.handle(Some(&key_down()), 5);
    cursor.handle(Some(&key_down()), 5);
    assert_eq!(cursor.current, 3);
}

// ============================================================================
// Up navigation tests
// ============================================================================

#[test]
fn cursor_move_up() {
    let mut cursor = Cursor::new(3);
    cursor.handle(Some(&key_up()), 5);
    assert_eq!(cursor.current, 2);
}

#[test]
fn cursor_move_up_wraps_at_start() {
    let mut cursor = Cursor::new(0);
    cursor.handle(Some(&key_up()), 5);
    // Should wrap to 4 (max - 1)
    assert_eq!(cursor.current, 4);
}

#[test]
fn cursor_move_up_multiple() {
    let mut cursor = Cursor::new(4);
    cursor.handle(Some(&key_up()), 5);
    cursor.handle(Some(&key_up()), 5);
    assert_eq!(cursor.current, 2);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn cursor_with_zero_max_does_nothing() {
    let mut cursor = Cursor::new(0);
    cursor.handle(Some(&key_down()), 0);
    assert_eq!(cursor.current, 0);

    cursor.handle(Some(&key_up()), 0);
    assert_eq!(cursor.current, 0);
}

#[test]
fn cursor_with_single_item() {
    let mut cursor = Cursor::new(0);
    cursor.handle(Some(&key_down()), 1);
    // Should stay at 0 (wraps around)
    assert_eq!(cursor.current, 0);

    cursor.handle(Some(&key_up()), 1);
    assert_eq!(cursor.current, 0);
}

#[test]
fn cursor_handles_none_event() {
    let mut cursor = Cursor::new(2);
    cursor.handle(None, 5);
    // Should remain unchanged
    assert_eq!(cursor.current, 2);
}

#[test]
fn cursor_full_cycle_down() {
    let mut cursor = Cursor::new(0);
    let max = 3;

    cursor.handle(Some(&key_down()), max); // 0 -> 1
    assert_eq!(cursor.current, 1);
    cursor.handle(Some(&key_down()), max); // 1 -> 2
    assert_eq!(cursor.current, 2);
    cursor.handle(Some(&key_down()), max); // 2 -> 0 (wrap)
    assert_eq!(cursor.current, 0);
}

#[test]
fn cursor_full_cycle_up() {
    let mut cursor = Cursor::new(0);
    let max = 3;

    cursor.handle(Some(&key_up()), max); // 0 -> 2 (wrap)
    assert_eq!(cursor.current, 2);
    cursor.handle(Some(&key_up()), max); // 2 -> 1
    assert_eq!(cursor.current, 1);
    cursor.handle(Some(&key_up()), max); // 1 -> 0
    assert_eq!(cursor.current, 0);
}
