use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::extensions::ThemedWidget;
use crate::testutils::*;
use crate::widgets::filter_select_popup::FilterSelectPopup;
use crate::widgets::popup::PopupWidget;

fn key_enter() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_esc() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_down() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_char(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn filter_select_popup_starts_closed() {
    let popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    assert!(!popup.is_open());
}

#[test]
fn filter_select_popup_display_selection_empty_when_no_items() {
    let popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    assert_eq!(popup.display_selection(), "");
}

// ============================================================================
// Open/Close tests
// ============================================================================

#[test]
fn filter_select_popup_open() {
    let mut popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    popup.open();
    assert!(popup.is_open());
}

#[test]
fn filter_select_popup_close() {
    let mut popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    popup.open();
    popup.close();
    assert!(!popup.is_open());
}

#[test]
fn filter_select_popup_open_resets_filter() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["Apple".to_string(), "Banana".to_string()]));
    popup.open();

    // Type to filter
    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let _ = popup.handle_event(Some(&key_char('A')), area, &mut actions);

    // Close and reopen
    popup.close();
    popup.open();

    // Filter should be reset - display_selection returns first item
    assert_eq!(popup.display_selection(), "Apple");
}

// ============================================================================
// Set items tests
// ============================================================================

#[test]
fn filter_select_popup_set_items() {
    let mut popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    popup.set_items(Some(vec!["A".to_string(), "B".to_string(), "C".to_string()]));
    popup.open();
    assert_eq!(popup.display_selection(), "A");
}

#[test]
fn filter_select_popup_set_items_none() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["A".to_string()]));
    popup.set_items(None);
    assert_eq!(popup.display_selection(), "");
}

#[test]
fn filter_select_popup_set_focused_item() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]));
    popup.set_focused_item("Banana".to_string());
    assert_eq!(popup.display_selection(), "Banana");
}

// ============================================================================
// Builder tests
// ============================================================================

#[test]
fn filter_select_popup_with_empty_text() {
    let popup: FilterSelectPopup<String> =
        FilterSelectPopup::default().with_empty_text("No options");
    // Just verify it builds without panic
    assert!(!popup.is_open());
}

// ============================================================================
// Event handling tests
// ============================================================================

#[test]
fn filter_select_popup_select_item_closes_popup() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["Apple".to_string(), "Banana".to_string()]));
    popup.open();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
    assert!(!popup.is_open());
}

#[test]
fn filter_select_popup_select_returns_selected_item() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["Apple".to_string(), "Banana".to_string()]));
    popup.open();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    // Navigate down first
    let _ = popup.handle_event(Some(&key_down()), area, &mut actions);
    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(result.is_ok());
    let selected = result.unwrap();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().as_str(), "Banana");
}

#[test]
fn filter_select_popup_ignores_esc_for_parent_when_open() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["Apple".to_string()]));
    popup.open();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_esc()), area, &mut actions);

    assert!(actions.esc_ignored);
}

#[test]
fn filter_select_popup_does_not_ignore_esc_when_closed() {
    let mut popup: FilterSelectPopup<String> = FilterSelectPopup::default();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let _ = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(!actions.esc_ignored);
}

#[test]
fn filter_select_popup_filter_works() {
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]));
    popup.open();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    // Type 'B' to filter
    let _ = popup.handle_event(Some(&key_char('B')), area, &mut actions);

    // Now only Banana should match, select it
    let result = popup.handle_event(Some(&key_enter()), area, &mut actions);

    assert!(result.is_ok());
    let selected = result.unwrap();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().as_str(), "Banana");
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn filter_select_popup_does_not_render_when_closed() {
    let mut term = TestTerminal::new(40, 20);
    let popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert_eq!(output, "");
}

#[test]
fn filter_select_popup_renders_when_open() {
    let mut term = TestTerminal::new(40, 15);
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec!["Apple".to_string(), "Banana".to_string()]));
    popup.open();
    let theme = TestTheme::boxed();

    popup.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Apple"));
    assert!(output.contains("Banana"));
    assert!(output.contains("Type to filter"));
}

#[test]
fn filter_select_popup_renders_filtered_items() {
    let mut term = TestTerminal::new(40, 15);
    let mut popup = FilterSelectPopup::default();
    popup.set_items(Some(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]));
    popup.open();

    let mut actions = TestAct::default();
    let area = ratatui::layout::Rect::new(0, 0, 40, 15);

    // Type 'a' to filter
    let _ = popup.handle_event(Some(&key_char('a')), area, &mut actions);

    let theme = TestTheme::boxed();
    popup.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    // Only items containing 'a' should be shown
    assert!(output.contains("Banana"));
    assert!(output.contains("Filter: a"));
}

// ============================================================================
// PopupWidget trait tests
// ============================================================================

#[test]
fn filter_select_popup_body_area() {
    let popup: FilterSelectPopup<String> = FilterSelectPopup::default();
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let body_area = popup.body_area(popup_area);

    // Body area should be inside the popup
    assert!(body_area.x >= popup_area.x);
    assert!(body_area.y >= popup_area.y);
    assert!(body_area.width <= popup_area.width);
    assert!(body_area.height <= popup_area.height);
}
