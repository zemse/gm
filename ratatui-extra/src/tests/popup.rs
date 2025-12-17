use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::extensions::ThemedWidget;
use crate::testutils::*;
use crate::widgets::popup::{Popup, PopupWidget};

fn key_esc() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

// ============================================================================
// Open/Close tests
// ============================================================================

#[test]
fn popup_starts_closed_by_default() {
    let popup = Popup::default();
    assert!(!popup.is_open());
}

#[test]
fn popup_can_be_opened() {
    let mut popup = Popup::default();
    popup.open();
    assert!(popup.is_open());
}

#[test]
fn popup_can_be_closed() {
    let mut popup = Popup::default().with_open(true);
    popup.close();
    assert!(!popup.is_open());
}

#[test]
fn popup_with_open_builder() {
    let popup = Popup::default().with_open(true);
    assert!(popup.is_open());
}

// ============================================================================
// Title tests
// ============================================================================

#[test]
fn popup_with_title_builder() {
    let popup = Popup::default().with_title("My Title");
    // Title is set (we can verify via rendering)
    assert!(popup.get_base_popup().get_areas(ratatui::layout::Rect::new(0, 0, 30, 20)).title_area.height > 0);
}

#[test]
fn popup_without_title_has_no_title_area() {
    let popup = Popup::default();
    let areas = popup.get_areas(ratatui::layout::Rect::new(0, 0, 30, 20));
    assert_eq!(areas.title_area.height, 0);
}

// ============================================================================
// Event handling tests
// ============================================================================

#[test]
fn esc_closes_open_popup() {
    let mut popup = Popup::default().with_open(true);
    let mut actions = TestAct::default();

    popup.handle_event(Some(&key_esc()), &mut actions);

    assert!(!popup.is_open());
    assert!(actions.esc_ignored);
}

#[test]
fn esc_on_closed_popup_does_nothing() {
    let mut popup = Popup::default();
    let mut actions = TestAct::default();

    popup.handle_event(Some(&key_esc()), &mut actions);

    assert!(!popup.is_open());
    assert!(!actions.esc_ignored);
}

#[test]
fn open_popup_ignores_esc_for_parent() {
    let mut popup = Popup::default().with_open(true);
    let mut actions = TestAct::default();

    popup.handle_event(None, &mut actions);

    // Even without an event, open popup marks esc as ignored
    assert!(actions.esc_ignored);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn render_popup_boxed_no_title() {
    let mut term = TestTerminal::new(20, 10);
    let popup = Popup::default();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let expected = "\
┌──────────────────┐
│                  │
│                  │
│                  │
│                  │
│                  │
│                  │
│                  │
│                  │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_popup_boxed_with_title() {
    let mut term = TestTerminal::new(20, 10);
    let popup = Popup::default().with_title("Hello");
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    // Title appears on first inner line
    let expected = "\
┌──────────────────┐
│ Hello            │
│                  │
│                  │
│                  │
│                  │
│                  │
│                  │
│                  │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_popup_unboxed_no_title() {
    let mut term = TestTerminal::new(20, 10);
    let popup = Popup::default();
    let theme = TestTheme::unboxed();

    popup.render(term.area, &mut term.buffer, &theme);

    // Unboxed popup is just empty (no border)
    let expected = "";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_popup_unboxed_with_title() {
    let mut term = TestTerminal::new(20, 10);
    let popup = Popup::default().with_title("Hello");
    let theme = TestTheme::unboxed();

    popup.render(term.area, &mut term.buffer, &theme);

    // Unboxed popup shows title with margin
    let expected = "\
\n  Hello";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Body area tests
// ============================================================================

#[test]
fn body_area_without_title() {
    let popup = Popup::default();
    let popup_area = ratatui::layout::Rect::new(0, 0, 30, 20);

    let body_area = popup.body_area(popup_area);

    // Body area is inner area (margin 2 horizontal, 1 vertical)
    assert_eq!(body_area.x, 2);
    assert_eq!(body_area.y, 1);
    assert_eq!(body_area.width, 26);
    assert_eq!(body_area.height, 18);
}

#[test]
fn body_area_with_title() {
    let popup = Popup::default().with_title("Title");
    let popup_area = ratatui::layout::Rect::new(0, 0, 30, 20);

    let body_area = popup.body_area(popup_area);

    // Body area starts below title (title height + 1 for spacing)
    assert_eq!(body_area.x, 2);
    assert!(body_area.y > 1); // Below title
    assert_eq!(body_area.width, 26);
}

#[test]
fn body_area_with_long_title() {
    let popup = Popup::default().with_title("This is a very long title that will wrap");
    let popup_area = ratatui::layout::Rect::new(0, 0, 20, 20);

    let body_area = popup.body_area(popup_area);

    // Body area should account for wrapped title
    assert!(body_area.y > 2); // Title wraps to multiple lines
}
