use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

use crate::extensions::ThemedWidget;
use crate::testutils::*;
use crate::widgets::popup::PopupWidget;
use crate::widgets::text_popup::{TextPopup, TextPopupEvent};

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

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn text_popup_starts_closed() {
    let popup = TextPopup::default();
    assert!(!popup.is_open());
}

#[test]
fn text_popup_with_text_opens_automatically() {
    let popup = TextPopup::default().with_text("Hello".to_string());
    assert!(popup.is_open());
}

#[test]
fn text_popup_with_empty_text_stays_closed() {
    let popup = TextPopup::default().with_text(String::new());
    assert!(!popup.is_open());
}

// ============================================================================
// Text manipulation tests
// ============================================================================

#[test]
fn text_popup_set_text_opens_popup() {
    let mut popup = TextPopup::default();
    popup.set_text("Hello".to_string(), false);
    assert!(popup.is_open());
}

#[test]
fn text_popup_set_empty_text_closes_popup() {
    let mut popup = TextPopup::default().with_text("Hello".to_string());
    popup.set_text(String::new(), false);
    assert!(!popup.is_open());
}

#[test]
fn text_popup_text_getter() {
    let popup = TextPopup::default().with_text("Hello World".to_string());
    assert_eq!(popup.text(), "Hello World");
}

// ============================================================================
// Builder tests
// ============================================================================

#[test]
fn text_popup_with_title() {
    let popup = TextPopup::default()
        .with_title("My Title")
        .with_text("Content".to_string());

    assert!(popup.is_open());
}

#[test]
fn text_popup_with_note() {
    let popup = TextPopup::default()
        .with_text("Main content".to_string())
        .with_note("Press Enter to close");

    assert!(popup.is_open());
}

// ============================================================================
// Event handling tests
// ============================================================================

#[test]
fn text_popup_esc_closes_and_returns_event() {
    let mut popup = TextPopup::default().with_text("Hello".to_string());
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_esc()), area, &mut actions);

    assert!(!popup.is_open());
    assert!(matches!(result, Some(TextPopupEvent::Closed)));
}

#[test]
fn text_popup_enter_closes_and_returns_event() {
    let mut popup = TextPopup::default().with_text("Hello".to_string());
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(!popup.is_open());
    assert!(matches!(result, Some(TextPopupEvent::Closed)));
}

#[test]
fn text_popup_ignores_esc_for_parent() {
    let mut popup = TextPopup::default().with_text("Hello".to_string());
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    popup.handle_event(None, area, &mut actions);

    assert!(actions.esc_ignored);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn text_popup_renders_when_open() {
    let mut term = TestTerminal::new(30, 10);
    let popup = TextPopup::default().with_text("Hello World".to_string());
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌────────────────────────────┐
│ Hello World                │
│                            │
│                            │
│                            │
│                            │
│                            │
│                            │
│                            │
└────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn text_popup_does_not_render_when_closed() {
    let mut term = TestTerminal::new(30, 15);
    let popup = TextPopup::default();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert_eq!(output, "");
}

#[test]
fn text_popup_renders_with_title() {
    let mut term = TestTerminal::new(30, 10);
    let popup = TextPopup::default()
        .with_title("Info")
        .with_text("Details here".to_string());
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌────────────────────────────┐
│ Info                       │
│                            │
│ Details here               │
│                            │
│                            │
│                            │
│                            │
│                            │
└────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// PopupWidget trait tests
// ============================================================================

#[test]
fn text_popup_open_close() {
    let mut popup = TextPopup::default().with_text("Test".to_string());

    assert!(popup.is_open());

    popup.close();
    assert!(!popup.is_open());

    popup.open();
    assert!(popup.is_open());
}

#[test]
fn text_popup_body_area() {
    let popup = TextPopup::default().with_text("Test".to_string());
    let popup_area = ratatui::layout::Rect::new(0, 0, 30, 20);

    let body_area = popup.body_area(popup_area);

    // Body area should be inside the popup
    assert!(body_area.x >= popup_area.x);
    assert!(body_area.y >= popup_area.y);
    assert!(body_area.width <= popup_area.width);
    assert!(body_area.height <= popup_area.height);
}
