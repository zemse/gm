use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

use crate::testutils::*;
use crate::widgets::button::{Button, ButtonResult};

fn key_enter() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn mouse_move(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_down(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

// ============================================================================
// Rendering tests - Boxed theme
// ============================================================================

#[test]
fn render_button_focused_boxed() {
    let mut term = TestTerminal::new(20, 10);
    let button = Button::new("Submit");
    let theme = TestTheme::boxed();

    button.render(term.area, &mut term.buffer, true, &theme);

    // Button width = label.len() + 4 = 6 + 4 = 10
    let expected = "\
┌────────┐
│Submit  │
└────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_button_unfocused_boxed() {
    let mut term = TestTerminal::new(20, 10);
    let button = Button::new("Submit");
    let theme = TestTheme::boxed();

    button.render(term.area, &mut term.buffer, false, &theme);

    let expected = "\
┌────────┐
│Submit  │
└────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_button_short_label_boxed() {
    let mut term = TestTerminal::new(20, 10);
    let button = Button::new("OK");
    let theme = TestTheme::boxed();

    button.render(term.area, &mut term.buffer, true, &theme);

    // Button width = 2 + 4 = 6
    let expected = "\
┌────┐
│OK  │
└────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_button_long_label_boxed() {
    let mut term = TestTerminal::new(30, 10);
    let button = Button::new("Confirm Action");
    let theme = TestTheme::boxed();

    button.render(term.area, &mut term.buffer, true, &theme);

    // Button width = 14 + 4 = 18
    let expected = "\
┌────────────────┐
│Confirm Action  │
└────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Rendering tests - Unboxed theme
// ============================================================================

#[test]
fn render_button_focused_unboxed() {
    let mut term = TestTerminal::new(20, 10);
    let button = Button::new("Submit");
    let theme = TestTheme::unboxed();

    button.render(term.area, &mut term.buffer, true, &theme);

    // Unboxed renders the label with cursor style when focused
    let expected = "\
\n  Submit";
    assert_eq!(term.render_to_string(), expected);
    // Cursor styling applied to button area (starts at 0,0)
    assert_eq!(term.find_cursor(), Some((0, 0, ' ')));
}

#[test]
fn render_button_unfocused_unboxed() {
    let mut term = TestTerminal::new(20, 10);
    let button = Button::new("Submit");
    let theme = TestTheme::unboxed();

    button.render(term.area, &mut term.buffer, false, &theme);

    let expected = "\
\n  Submit";
    assert_eq!(term.render_to_string(), expected);
    // No cursor when unfocused
    assert_eq!(term.find_cursor(), None);
}

// ============================================================================
// Keyboard interaction tests
// ============================================================================

#[test]
fn enter_key_presses_button_when_focused() {
    let mut button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let result = button.handle_event(Some(&key_enter()), area, true);

    assert!(matches!(result, Some(ButtonResult::Pressed)));
}

#[test]
fn enter_key_does_nothing_when_unfocused() {
    let mut button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    let result = button.handle_event(Some(&key_enter()), area, false);

    assert!(result.is_none());
}

// ============================================================================
// Mouse interaction tests
// ============================================================================

#[test]
fn mouse_click_inside_presses_button() {
    let mut button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    // Click inside button area (button is at 0,0 with width=10, height=3)
    let result = button.handle_event(Some(&mouse_down(5, 1)), area, false);

    assert!(matches!(result, Some(ButtonResult::Pressed)));
}

#[test]
fn mouse_click_outside_does_nothing() {
    let mut button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    // Click outside button area
    let result = button.handle_event(Some(&mouse_down(15, 1)), area, false);

    assert!(result.is_none());
}

#[test]
fn mouse_hover_inside_sets_hover_focus() {
    let mut button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    // Move mouse inside button area
    let result = button.handle_event(Some(&mouse_move(5, 1)), area, false);

    assert!(matches!(result, Some(ButtonResult::HoverIn(true))));
    assert!(button.hover_focus);
}

#[test]
fn mouse_hover_outside_clears_hover_focus() {
    let mut button = Button::new("Submit");
    button.hover_focus = true;
    let area = ratatui::layout::Rect::new(0, 0, 20, 10);

    // Move mouse outside button area
    let result = button.handle_event(Some(&mouse_move(15, 5)), area, false);

    assert!(matches!(result, Some(ButtonResult::HoverIn(false))));
    assert!(!button.hover_focus);
}

#[test]
fn hover_focus_makes_button_appear_focused() {
    let mut term = TestTerminal::new(20, 10);
    let mut button = Button::new("Submit");
    button.hover_focus = true;
    let theme = TestTheme::unboxed();

    // Render with focus=false but hover_focus=true
    button.render(term.area, &mut term.buffer, false, &theme);

    // Should render as focused due to hover_focus (cursor at top-left of button area)
    assert_eq!(term.find_cursor(), Some((0, 0, ' ')));
}

// ============================================================================
// Button area tests
// ============================================================================

#[test]
fn button_area_calculated_correctly() {
    let button = Button::new("Submit");
    let area = ratatui::layout::Rect::new(5, 10, 30, 20);

    let button_area = button.area(area);

    assert_eq!(button_area.x, 5);
    assert_eq!(button_area.y, 10);
    assert_eq!(button_area.width, 10); // "Submit".len() + 4 = 10
    assert_eq!(button_area.height, 3);
}

#[test]
fn button_area_short_label() {
    let button = Button::new("OK");
    let area = ratatui::layout::Rect::new(0, 0, 30, 20);

    let button_area = button.area(area);

    assert_eq!(button_area.width, 6); // "OK".len() + 4 = 6
}

// ============================================================================
// Success kind tests
// ============================================================================

#[test]
fn button_with_success_kind_false() {
    let button = Button::new("Cancel").with_success_kind(false);

    assert!(!button.is_success);
}

#[test]
fn button_default_is_success() {
    let button = Button::new("Submit");

    assert!(button.is_success);
}
