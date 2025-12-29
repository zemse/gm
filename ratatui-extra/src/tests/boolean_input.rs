use ratatui::crossterm::event::Event;

use crate::testutils::*;
use crate::widgets::boolean_input::BooleanInput;

// ============================================================================
// Rendering tests - Boxed theme
// ============================================================================

#[test]
fn render_boolean_off_focused_boxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", false);
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ◉── On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "Off" (cancelled style)
    assert_eq!(term.find_cursor(), Some((2, 1, 'O')));
}

#[test]
fn render_boolean_on_focused_boxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", true);
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ──◉ On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "On"
    assert_eq!(term.find_cursor(), Some((10, 1, 'O')));
}

#[test]
fn render_boolean_off_unfocused_boxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", false);
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, false, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ◯── On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // No cursor when unfocused
    assert_eq!(term.find_cursor(), None);
}

#[test]
fn render_boolean_on_unfocused_boxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", true);
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, false, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ──◯ On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // No cursor when unfocused
    assert_eq!(term.find_cursor(), None);
}

// ============================================================================
// Rendering tests - Unboxed theme
// ============================================================================

#[test]
fn render_boolean_off_focused_unboxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", false);
    let theme = TestTheme::unboxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
Enabled
  Off ◉── On";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "Off"
    assert_eq!(term.find_cursor(), Some((2, 1, 'O')));
}

#[test]
fn render_boolean_on_focused_unboxed() {
    let mut term = TestTerminal::new(25, 10);
    let input = BooleanInput::new("Enabled", true);
    let theme = TestTheme::unboxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
Enabled
  Off ──◉ On";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "On"
    assert_eq!(term.find_cursor(), Some((10, 1, 'O')));
}

// ============================================================================
// Keyboard interaction tests
// ============================================================================

#[test]
fn right_key_turns_on() {
    let mut term = TestTerminal::new(25, 10);
    let mut input = BooleanInput::new("Enabled", false);
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    input.handle_event(Some(&right()), term.area, &mut actions);

    assert!(input.value);
    assert!(actions.right_ignored);

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ──◉ On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "On"
    assert_eq!(term.find_cursor(), Some((10, 1, 'O')));
}

#[test]
fn left_key_turns_off() {
    let mut term = TestTerminal::new(25, 10);
    let mut input = BooleanInput::new("Enabled", true);
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    input.handle_event(Some(&left()), term.area, &mut actions);

    assert!(!input.value);
    assert!(actions.left_ignored);

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Enabled────────────────┐
│ Off ◉── On            │
└───────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor on "Off"
    assert_eq!(term.find_cursor(), Some((2, 1, 'O')));
}

#[test]
fn right_key_when_already_on_does_nothing() {
    let mut input = BooleanInput::new("Enabled", true);
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 25, 10);

    input.handle_event(Some(&right()), area, &mut actions);

    assert!(input.value);
    // Right key not consumed since already on
    assert!(!actions.right_ignored);
}

#[test]
fn left_key_when_already_off_does_nothing() {
    let mut input = BooleanInput::new("Enabled", false);
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 25, 10);

    input.handle_event(Some(&left()), area, &mut actions);

    assert!(!input.value);
    // Left key not consumed since already off
    assert!(!actions.left_ignored);
}

// ============================================================================
// Toggle tests
// ============================================================================

#[test]
fn toggle_changes_value() {
    let mut input = BooleanInput::new("Enabled", false);

    input.toggle();
    assert!(input.value);

    input.toggle();
    assert!(!input.value);
}

// ============================================================================
// Mouse interaction tests
// ============================================================================

#[test]
fn mouse_click_toggles_value() {
    let mut input = BooleanInput::new("Enabled", false);
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 25, 10);

    // Click inside the switch area (inner area after block_inner)
    let click = mouse_click(2, 1);
    input.handle_event(
        Some(&Event::Mouse(match click {
            Event::Mouse(m) => m,
            _ => panic!(),
        })),
        area,
        &mut actions,
    );

    assert!(input.value);
}

#[test]
fn mouse_click_outside_does_nothing() {
    let mut input = BooleanInput::new("Enabled", false);
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 25, 10);

    // Click outside the switch area
    let click = mouse_click(20, 1);
    input.handle_event(
        Some(&Event::Mouse(match click {
            Event::Mouse(m) => m,
            _ => panic!(),
        })),
        area,
        &mut actions,
    );

    assert!(!input.value);
}
