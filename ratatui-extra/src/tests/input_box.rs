use crate::event::WidgetEvent;
use crate::testutils::*;
use crate::widgets::input_box::InputBox;

// ============================================================================
// Rendering tests - Small terminal (20x10)
// ============================================================================

#[test]
fn render_empty_input_small_boxed() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Name");
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│                  │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_empty_input_small_unboxed() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Name");
    let theme = TestTheme::unboxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
Name
>";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_with_text_small_boxed() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Alice".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Alice            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_with_placeholder_small() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Email").with_empty_text("Enter email");
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Email─────────────┐
│ Enter email      │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Rendering tests - Large terminal (40x10)
// ============================================================================

#[test]
fn render_empty_input_large_boxed() {
    let mut term = TestTerminal::new(40, 10);
    let input = InputBox::new("Username");
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Username──────────────────────────────┐
│                                      │
└──────────────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_with_text_large_boxed() {
    let mut term = TestTerminal::new(40, 10);
    let mut input = InputBox::new("Address");
    input.set_text("123 Main Street".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Address───────────────────────────────┐
│ 123 Main Street                      │
└──────────────────────────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_long_text_wraps() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Msg");
    input.set_text("This is a very long message".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    // Text wraps at word boundaries when possible
    let expected = "\
┌Msg───────────────┐
│ This is a very   │
│ long message     │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_with_currency() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Amount").with_currency("ETH".to_string());
    input.set_text("1.5".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Amount────────────┐
│ 1.5 ETH          │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Cursor rendering tests
// ============================================================================

#[test]
fn cursor_at_end_of_text() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hi".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hi               │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (4, 1), on space after "Hi"
    assert_eq!(term.find_cursor(), Some((4, 1, ' ')));
}

#[test]
fn cursor_at_start_when_empty() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Name");
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│                  │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (2, 1), on empty space
    assert_eq!(term.find_cursor(), Some((2, 1, ' ')));
}

#[test]
fn cursor_after_moving_left() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Move cursor left twice (from position 5 to position 3)
    input.handle_event(
        Some(&WidgetEvent::InputEvent(left())),
        term.area,
        &mut actions,
    );
    input.handle_event(
        Some(&WidgetEvent::InputEvent(left())),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hello            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (5, 1), on 'l'
    assert_eq!(term.find_cursor(), Some((5, 1, 'l')));
}

#[test]
fn cursor_after_ctrl_a_at_start() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Ctrl+A moves to start
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hello            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (2, 1), on 'H'
    assert_eq!(term.find_cursor(), Some((2, 1, 'H')));
}

#[test]
fn cursor_after_ctrl_e_at_end() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // First move to start
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );

    // Then Ctrl+E to move to end
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('e'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hello            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (7, 1), on space after "Hello"
    assert_eq!(term.find_cursor(), Some((7, 1, ' ')));
}

#[test]
fn cursor_in_unboxed_mode() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hi".to_string());
    let theme = TestTheme::unboxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
Name
> Hi";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (4, 1), on space after "Hi"
    assert_eq!(term.find_cursor(), Some((4, 1, ' ')));
}

#[test]
fn cursor_in_unboxed_mode_empty() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Name");
    let theme = TestTheme::unboxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
Name
>";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (2, 1), on space after "> "
    assert_eq!(term.find_cursor(), Some((2, 1, ' ')));
}

#[test]
fn cursor_after_mouse_click() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Click at x=4 (after "He") - inner area starts at x=2
    input.handle_event(
        Some(&WidgetEvent::InputEvent(mouse_click(4, 1))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hello            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (4, 1), on 'l'
    assert_eq!(term.find_cursor(), Some((4, 1, 'l')));
}

#[test]
fn cursor_on_second_line_for_wrapped_text() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Msg");
    input.set_text("This is a very long message".to_string());
    let theme = TestTheme::boxed();

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Msg───────────────┐
│ This is a very   │
│ long message     │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (13, 2), on 'e' at end of "message"
    assert_eq!(term.find_cursor(), Some((13, 2, 'e')));
}

#[test]
fn cursor_in_middle_of_wrapped_text() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Msg");
    input.set_text("This is a very long message".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Move cursor to start of "long" (position 16)
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );
    for _ in 0..16 {
        input.handle_event(
            Some(&WidgetEvent::InputEvent(right())),
            term.area,
            &mut actions,
        );
    }

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Msg───────────────┐
│ This is a very   │
│ long message     │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    // Cursor at position (2, 2), on 'l' of "long"
    assert_eq!(term.find_cursor(), Some((2, 2, 'l')));
}

// ============================================================================
// Keyboard interaction tests
// ============================================================================

#[test]
fn type_characters() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Type "Hi"
    for event in [key('H'), key('i')] {
        input.handle_event(
            Some(&WidgetEvent::InputEvent(event)),
            term.area,
            &mut actions,
        );
    }

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hi               │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn backspace_deletes_character() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Press backspace twice
    for _ in 0..2 {
        input.handle_event(
            Some(&WidgetEvent::InputEvent(backspace())),
            term.area,
            &mut actions,
        );
    }

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hel              │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn left_arrow_moves_cursor() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("ABC".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Move cursor left twice (from end position 3 to position 1)
    input.handle_event(
        Some(&WidgetEvent::InputEvent(left())),
        term.area,
        &mut actions,
    );
    input.handle_event(
        Some(&WidgetEvent::InputEvent(left())),
        term.area,
        &mut actions,
    );

    // Type 'X' - should insert at position 1
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ AXBC             │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn right_arrow_moves_cursor() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("ABC".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Move cursor to start with Ctrl+A
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );

    // Move right once
    input.handle_event(
        Some(&WidgetEvent::InputEvent(right())),
        term.area,
        &mut actions,
    );

    // Type 'X' - should insert at position 1
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ AXBC             │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn ctrl_a_moves_to_start() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Ctrl+A to move to start
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );

    // Type 'X' at start
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ XHello           │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn ctrl_e_moves_to_end() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Move to start first
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('a'))),
        term.area,
        &mut actions,
    );

    // Ctrl+E to move to end
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('e'))),
        term.area,
        &mut actions,
    );

    // Type 'X' at end
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ HelloX           │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn ctrl_u_clears_to_start() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello World".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Ctrl+U clears everything before cursor (cursor is at end)
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key_ctrl('u'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│                  │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn alt_backspace_deletes_word() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello World".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Alt+Backspace deletes word before cursor
    input.handle_event(
        Some(&WidgetEvent::InputEvent(backspace_alt())),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ Hello            │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn alt_left_moves_word_left() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello World".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Alt+Left moves cursor to the space before current word
    input.handle_event(
        Some(&WidgetEvent::InputEvent(left_alt())),
        term.area,
        &mut actions,
    );

    // Type 'X' - inserts at the space position (before "World")
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ HelloX World     │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Mouse interaction tests
// ============================================================================

#[test]
fn mouse_click_positions_cursor() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    input.set_text("Hello".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Click at position x=4 (after "He") inside the input area
    // Inner text area starts at x=2, so clicking at x=4 means cursor pos 2
    input.handle_event(
        Some(&WidgetEvent::InputEvent(mouse_click(4, 1))),
        term.area,
        &mut actions,
    );

    // Type 'X' - should insert at position 2
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ HeXllo           │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Focus and immutable tests
// ============================================================================

#[test]
fn unfocused_input_has_no_cursor() {
    let mut term = TestTerminal::new(20, 10);
    let input = InputBox::new("Name");
    let theme = TestTheme::boxed();

    // Render without focus
    input.render(term.area, &mut term.buffer, false, &theme);

    // No cursor should be visible (no reversed cells)
    assert!(!term.has_cursor_at(2, 1));
}

#[test]
fn immutable_input_ignores_typing() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name").make_immutable(true);
    input.set_text("ReadOnly".to_string());
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Try to type
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('X'))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    // Text should remain unchanged
    let expected = "\
┌Name──────────────┐
│ ReadOnly         │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn leading_spaces_ignored() {
    let mut term = TestTerminal::new(20, 10);
    let mut input = InputBox::new("Name");
    let mut actions = TestAct::default();
    let theme = TestTheme::boxed();

    // Try to type space when empty
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key(' '))),
        term.area,
        &mut actions,
    );

    assert_eq!(input.get_text(), "");

    // Type a character first
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key('A'))),
        term.area,
        &mut actions,
    );

    // Now space should work
    input.handle_event(
        Some(&WidgetEvent::InputEvent(key(' '))),
        term.area,
        &mut actions,
    );

    input.render(term.area, &mut term.buffer, true, &theme);

    let expected = "\
┌Name──────────────┐
│ A                │
└──────────────────┘";
    assert_eq!(term.render_to_string(), expected);
    assert_eq!(input.get_text(), "A ");
}
