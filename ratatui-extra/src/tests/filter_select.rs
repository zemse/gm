use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use crate::testutils::*;
use crate::widgets::filter_select::FilterSelect;

fn key_char(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_backspace() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Backspace,
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

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn filter_select_default_has_no_items() {
    let fs: FilterSelect<String> = FilterSelect::default();
    assert_eq!(fs.list_len(), 0);
}

#[test]
fn filter_select_with_items() {
    let fs = FilterSelect::default().with_items(vec!["Apple", "Banana", "Cherry"]);
    assert_eq!(fs.list_len(), 3);
}

#[test]
fn filter_select_search_string_starts_empty() {
    let fs: FilterSelect<String> = FilterSelect::default();
    assert!(fs.search_string.is_empty());
}

// ============================================================================
// Builder tests
// ============================================================================

#[test]
fn filter_select_with_empty_text() {
    let mut term = TestTerminal::new(20, 10);
    let fs: FilterSelect<String> = FilterSelect::default()
        .with_items(vec![])
        .with_empty_text("Nothing found");
    let theme = TestTheme::boxed();

    fs.render(term.area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Nothing found"));
}

#[test]
fn filter_select_with_focus() {
    let fs: FilterSelect<String> = FilterSelect::default().with_focus(true);
    // Focus is set on internal select
    assert!(fs.select.cursor() == 0); // Select is initialized
}

// ============================================================================
// Filtering tests
// ============================================================================

#[test]
fn filter_select_typing_filters_list() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Type 'A' to filter
    let _ = fs.handle_event(Some(&key_char('A')), area);

    // Should filter to items containing 'A'
    assert_eq!(fs.search_string, "A");
    assert_eq!(fs.list_len(), 2); // Apple, Apricot
}

#[test]
fn filter_select_typing_more_narrows_filter() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    let _ = fs.handle_event(Some(&key_char('A')), area);
    let _ = fs.handle_event(Some(&key_char('p')), area);
    let _ = fs.handle_event(Some(&key_char('p')), area);

    assert_eq!(fs.search_string, "App");
    assert_eq!(fs.list_len(), 1); // Only Apple
}

#[test]
fn filter_select_backspace_widens_filter() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Type "App"
    let _ = fs.handle_event(Some(&key_char('A')), area);
    let _ = fs.handle_event(Some(&key_char('p')), area);
    let _ = fs.handle_event(Some(&key_char('p')), area);
    assert_eq!(fs.list_len(), 1);

    // Backspace
    let _ = fs.handle_event(Some(&key_backspace()), area);
    assert_eq!(fs.search_string, "Ap");
    assert_eq!(fs.list_len(), 2); // Apple, Apricot
}

#[test]
fn filter_select_empty_filter_shows_all() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Type and then delete
    let _ = fs.handle_event(Some(&key_char('A')), area);
    let _ = fs.handle_event(Some(&key_backspace()), area);

    assert!(fs.search_string.is_empty());
    assert_eq!(fs.list_len(), 3);
}

#[test]
fn filter_select_no_matches() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    let _ = fs.handle_event(Some(&key_char('X')), area);

    assert_eq!(fs.list_len(), 0);
}

// ============================================================================
// Navigation tests
// ============================================================================

#[test]
fn filter_select_down_navigates_filtered_list() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Filter to A items
    let _ = fs.handle_event(Some(&key_char('A')), area);

    // Navigate down
    let _ = fs.handle_event(Some(&key_down()), area);

    assert_eq!(fs.select.cursor(), 1);
}

// ============================================================================
// Selection tests
// ============================================================================

#[test]
fn filter_select_get_focussed_item() {
    let fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);

    let item = fs.get_focussed_item();
    assert!(item.is_ok());
    assert_eq!(item.unwrap().as_str(), "Apple");
}

#[test]
fn filter_select_get_focussed_item_after_filter() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Filter to B
    let _ = fs.handle_event(Some(&key_char('B')), area);

    let item = fs.get_focussed_item();
    assert!(item.is_ok());
    assert_eq!(item.unwrap().as_str(), "Banana");
}

// ============================================================================
// Reset tests
// ============================================================================

#[test]
fn filter_select_reset_clears_search_and_cursor() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Type and navigate
    let _ = fs.handle_event(Some(&key_char('A')), area);
    let _ = fs.handle_event(Some(&key_down()), area);

    fs.reset();

    assert!(fs.search_string.is_empty());
    assert_eq!(fs.select.cursor(), 0);
}

// ============================================================================
// Set items tests
// ============================================================================

#[test]
fn filter_select_set_items() {
    let mut fs: FilterSelect<String> = FilterSelect::default();

    fs.set_items(Some(vec!["A".to_string(), "B".to_string()]));

    assert_eq!(fs.list_len(), 2);
}

#[test]
fn filter_select_set_items_none() {
    let mut fs = FilterSelect::default().with_items(vec!["A".to_string()]);

    fs.set_items(None);

    assert_eq!(fs.list_len(), 0);
}

#[test]
fn filter_select_set_items_preserves_filter() {
    let mut fs = FilterSelect::default().with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    // Set filter
    let _ = fs.handle_event(Some(&key_char('A')), area);
    assert_eq!(fs.list_len(), 1);

    // Update items
    fs.set_items(Some(vec![
        "Apple".to_string(),
        "Apricot".to_string(),
        "Banana".to_string(),
    ]));

    // Filter should still apply
    assert_eq!(fs.list_len(), 2); // Apple, Apricot
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn filter_select_renders_items() {
    let mut term = TestTerminal::new(30, 6);
    let fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
    ]);
    let theme = TestTheme::boxed();

    fs.render(term.area, &mut term.buffer, &theme);

    let expected = "\
Type to filter

Apple
Banana
Cherry";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn filter_select_renders_filter_prompt() {
    let mut term = TestTerminal::new(30, 4);
    let fs = FilterSelect::default().with_items(vec!["Apple".to_string()]);
    let theme = TestTheme::boxed();

    fs.render(term.area, &mut term.buffer, &theme);

    let expected = "\
Type to filter

Apple";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn filter_select_renders_current_filter() {
    let mut term = TestTerminal::new(30, 5);
    let mut fs = FilterSelect::default().with_items(vec!["Apple".to_string(), "Banana".to_string()]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 5);
    let theme = TestTheme::boxed();

    let _ = fs.handle_event(Some(&key_char('A')), area);

    fs.render(term.area, &mut term.buffer, &theme);

    let expected = "\
Filter: A

Apple";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Case sensitivity test
// ============================================================================

#[test]
fn filter_select_is_case_sensitive() {
    let mut fs = FilterSelect::default().with_items(vec![
        "Apple".to_string(),
        "apple".to_string(),
        "APPLE".to_string(),
    ]);
    let area = ratatui::layout::Rect::new(0, 0, 30, 15);

    let _ = fs.handle_event(Some(&key_char('a')), area);

    // Only lowercase 'a' matches "apple"
    assert_eq!(fs.list_len(), 1);
}
