use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

use crate::testutils::*;
use crate::widgets::confirm_popup::{ConfirmPopup, ConfirmResult};
use crate::widgets::popup::PopupWidget;

fn key_esc() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_enter() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_left() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_right() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn confirm_popup_starts_closed() {
    let popup = ConfirmPopup::new("OK", "Cancel", true);
    assert!(!popup.is_open());
}

#[test]
fn confirm_popup_initial_focus_on_confirm() {
    let popup = ConfirmPopup::new("OK", "Cancel", true);
    assert!(popup.is_confirm_focused);
}

#[test]
fn confirm_popup_initial_focus_on_cancel() {
    let popup = ConfirmPopup::new("OK", "Cancel", false);
    assert!(!popup.is_confirm_focused);
}

// ============================================================================
// Open/Close tests
// ============================================================================

#[test]
fn confirm_popup_open() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    assert!(popup.is_open());
}

#[test]
fn confirm_popup_close() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    popup.close();
    assert!(!popup.is_open());
}

#[test]
fn confirm_popup_open_resets_focus() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    popup.is_confirm_focused = false; // Change focus

    popup.close();
    popup.open(); // Should reset to initial

    assert!(popup.is_confirm_focused);
}

#[test]
fn confirm_popup_open_resets_focus_to_cancel() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", false);
    popup.open();
    popup.is_confirm_focused = true; // Change focus

    popup.close();
    popup.open(); // Should reset to initial

    assert!(!popup.is_confirm_focused);
}

// ============================================================================
// Text tests
// ============================================================================

#[test]
fn confirm_popup_with_text() {
    let popup = ConfirmPopup::new("OK", "Cancel", true).with_text("Are you sure?".to_string());
    assert_eq!(popup.text_ref(), "Are you sure?");
}

#[test]
fn confirm_popup_set_text() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.set_text("New message".to_string(), false);
    assert_eq!(popup.text_ref(), "New message");
}

// ============================================================================
// Navigation tests
// ============================================================================

#[test]
fn confirm_popup_left_focuses_cancel() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_left()), area, &mut actions);

    assert!(!popup.is_confirm_focused);
}

#[test]
fn confirm_popup_right_focuses_confirm() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", false);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_right()), area, &mut actions);

    assert!(popup.is_confirm_focused);
}

// ============================================================================
// Confirm/Cancel tests
// ============================================================================

#[test]
fn confirm_popup_enter_confirms_when_confirm_focused() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(matches!(result, Ok(Some(ConfirmResult::Confirmed))));
    assert!(!popup.is_open());
}

#[test]
fn confirm_popup_enter_cancels_when_cancel_focused() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", false);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(matches!(result, Ok(Some(ConfirmResult::Canceled))));
    assert!(!popup.is_open());
}

#[test]
fn confirm_popup_esc_cancels() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_esc()), area, &mut actions);

    assert!(matches!(result, Ok(Some(ConfirmResult::Canceled))));
    assert!(!popup.is_open());
}

// ============================================================================
// Action tests
// ============================================================================

#[test]
fn confirm_popup_ignores_left_right_for_parent() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_left()), area, &mut actions);

    assert!(actions.left_ignored);
    assert!(actions.right_ignored);
}

#[test]
fn confirm_popup_ignores_esc_for_parent() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(actions.esc_ignored);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn confirm_popup_renders_when_open() {
    let mut term = TestTerminal::new(40, 10);
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    popup.open();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌──────────────────────────────────────┐
│                                      │
│                                      │
│                                      │
│                                      │
│                                      │
│      ┌────────┐          ┌────┐      │
│      │Cancel  │          │OK  │      │
│      └────────┘          └────┘      │
└──────────────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn confirm_popup_does_not_render_when_closed() {
    let mut term = TestTerminal::new(40, 20);
    let popup = ConfirmPopup::new("OK", "Cancel", true);
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert_eq!(output, "");
}

#[test]
fn confirm_popup_renders_with_text() {
    let mut term = TestTerminal::new(40, 10);
    let mut popup =
        ConfirmPopup::new("Delete", "Keep", true).with_text("Delete this item?".to_string());
    popup.open();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌──────────────────────────────────────┐
│ Delete this item?                    │
│                                      │
│                                      │
│                                      │
│                                      │
│       ┌──────┐         ┌────────┐    │
│       │Keep  │         │Delete  │    │
│       └──────┘         └────────┘    │
└──────────────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn confirm_popup_renders_with_title() {
    let mut term = TestTerminal::new(40, 10);
    let mut popup = ConfirmPopup::new("Yes", "No", true).with_title("Confirm");
    popup.open();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌──────────────────────────────────────┐
│ Confirm                              │
│                                      │
│                                      │
│                                      │
│                                      │
│        ┌────┐           ┌─────┐      │
│        │No  │           │Yes  │      │
│        └────┘           └─────┘      │
└──────────────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Closed popup tests
// ============================================================================

#[test]
fn confirm_popup_closed_does_not_handle_events() {
    let mut popup = ConfirmPopup::new("OK", "Cancel", true);
    // Don't open
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(matches!(result, Ok(None)));
    assert!(!actions.esc_ignored);
}

// ============================================================================
// Conversion tests
// ============================================================================

#[test]
fn confirm_popup_into_text() {
    let popup = ConfirmPopup::new("OK", "Cancel", true).with_text("Hello".to_string());
    let text = popup.into_text();
    assert_eq!(text.text(), "Hello");
}

#[test]
fn confirm_popup_into_text_popup() {
    let popup = ConfirmPopup::new("OK", "Cancel", true).with_text("Hello".to_string());
    let text_popup = popup.into_text_popup();
    assert_eq!(text_popup.text(), "Hello");
}
