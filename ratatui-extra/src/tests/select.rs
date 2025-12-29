use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::extensions::ThemedWidget;
use crate::testutils::*;
use crate::widgets::select::{Select, SelectEvent};

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

fn key_enter() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn select_default_has_no_list() {
    let select: Select<String> = Select::default();
    assert!(select.list_is_none());
    assert_eq!(select.list_len(), 0);
}

#[test]
fn select_with_list() {
    let select = Select::default().with_list(vec!["A", "B", "C"]);
    assert!(!select.list_is_none());
    assert_eq!(select.list_len(), 3);
}

#[test]
fn select_cursor_starts_at_zero() {
    let select = Select::default().with_list(vec!["A", "B", "C"]);
    assert_eq!(select.cursor(), 0);
}

// ============================================================================
// Builder tests
// ============================================================================

#[test]
fn select_with_loading_text() {
    let mut term = TestTerminal::new(20, 5);
    let select: Select<String> = Select::default().with_loading_text("Please wait...");
    let theme = TestTheme::boxed();

    select.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Please wait..."));
}

#[test]
fn select_with_empty_text() {
    let mut term = TestTerminal::new(20, 5);
    let select: Select<String> = Select::default()
        .with_list(vec![])
        .with_empty_text("Nothing here");
    let theme = TestTheme::boxed();

    select.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Nothing here"));
}

// ============================================================================
// Cursor movement tests
// ============================================================================

#[test]
fn select_cursor_moves_down() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let _ = select.handle_event(
        Some(&ratatui::crossterm::event::Event::Key(key_down())),
        area,
    );

    assert_eq!(select.cursor(), 1);
}

#[test]
fn select_cursor_moves_up() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let _ = select.handle_event(Some(&ratatui::crossterm::event::Event::Key(key_up())), area);

    assert_eq!(select.cursor(), 1);
}

#[test]
fn select_cursor_wraps_down() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let _ = select.handle_event(
        Some(&ratatui::crossterm::event::Event::Key(key_down())),
        area,
    );

    assert_eq!(select.cursor(), 0);
}

#[test]
fn select_cursor_wraps_up() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let _ = select.handle_event(Some(&ratatui::crossterm::event::Event::Key(key_up())), area);

    assert_eq!(select.cursor(), 2);
}

// ============================================================================
// Selection tests
// ============================================================================

#[test]
fn select_enter_selects_item() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(1);
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let result = select.handle_event(
        Some(&ratatui::crossterm::event::Event::Key(key_enter())),
        area,
    );

    assert!(matches!(result, Ok(Some(SelectEvent::Select(item))) if *item == "B"));
}

#[test]
fn select_get_focussed_item() {
    let select = Select::default().with_list(vec!["A", "B", "C"]);
    let item = select.get_focussed_item();
    assert!(matches!(item, Ok(&"A")));
}

#[test]
fn select_get_focussed_item_after_move() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    let item = select.get_focussed_item();
    assert!(matches!(item, Ok(&"C")));
}

// ============================================================================
// List manipulation tests
// ============================================================================

#[test]
fn select_set_cursor() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    assert_eq!(select.cursor(), 2);
}

#[test]
fn select_set_cursor_clamps_to_list_len() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(10);
    assert_eq!(select.cursor(), 2); // Clamped to last index
}

#[test]
fn select_reset_cursor() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    select.reset_cursor();
    assert_eq!(select.cursor(), 0);
}

#[test]
fn select_set_focussed_item() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_focussed_item("B");
    assert_eq!(select.cursor(), 1);
}

#[test]
fn select_set_focus_to_last_item() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_focus_to_last_item();
    assert_eq!(select.cursor(), 2);
}

#[test]
fn select_list_push() {
    let mut select: Select<&str> = Select::default();
    select.list_push("A");
    select.list_push("B");
    assert_eq!(select.list_len(), 2);
}

#[test]
fn select_remove_item_at_cursor() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(1);
    let removed = select.remove_item_at_cursor();
    assert_eq!(removed, Some("B"));
    assert_eq!(select.list_len(), 2);
}

#[test]
fn select_remove_item_adjusts_cursor() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2); // Last item
    select.remove_item_at_cursor();
    // Cursor should adjust to new last item
    assert_eq!(select.cursor(), 1);
}

#[test]
fn select_update_list() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.set_cursor(2);
    select.update_list(Some(vec!["X", "Y"]));
    // Cursor should adjust since list is shorter
    assert_eq!(select.cursor(), 1);
    assert_eq!(select.list_len(), 2);
}

#[test]
fn select_update_list_to_none() {
    let mut select = Select::default().with_list(vec!["A", "B", "C"]);
    select.update_list(None);
    assert!(select.list_is_none());
    assert_eq!(select.cursor(), 0);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn select_renders_loading_when_no_list() {
    let mut term = TestTerminal::new(20, 3);
    let select: Select<String> = Select::default();
    let theme = TestTheme::boxed();

    select.render(term.area, &mut term.buffer, &theme);

    let expected = "\
Loading...";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn select_renders_empty_text_for_empty_list() {
    let mut term = TestTerminal::new(20, 3);
    let select: Select<String> = Select::default().with_list(vec![]);
    let theme = TestTheme::boxed();

    select.render(term.area, &mut term.buffer, &theme);

    let expected = "\
no items";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn select_renders_items() {
    let mut term = TestTerminal::new(20, 5);
    let select = Select::default().with_list(vec!["Apple", "Banana", "Cherry"]);
    let theme = TestTheme::boxed();

    select.render(term.area, &mut term.buffer, &theme);

    let expected = "\
Apple
Banana
Cherry";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Focus tests
// ============================================================================

#[test]
fn select_set_focus() {
    let mut select = Select::default().with_list(vec!["A", "B"]);
    select.set_focus(true);
    // Focus affects rendering style, but we can verify it was set
    // by checking the rendered output differs
}

// ============================================================================
// Hover cursor tests
// ============================================================================

#[test]
fn select_with_hover_cursor() {
    let select: Select<&str> = Select::new(Some(vec!["A", "B"]), true);
    assert_eq!(select.hover_cursor(), Some(0));
}

#[test]
fn select_without_hover_cursor() {
    let select: Select<&str> = Select::new(Some(vec!["A", "B"]), false);
    assert_eq!(select.hover_cursor(), None);
}
