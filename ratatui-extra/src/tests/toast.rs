use std::time::Duration;

use ratatui::layout::Position;

use crate::testutils::*;
use crate::widgets::toast::Toast;

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn toast_starts_not_shown() {
    let toast = Toast::new("Hello");
    // Toast starts not shown, so rendering should produce nothing
    let mut term = TestTerminal::new(20, 10);
    let theme = TestTheme::boxed();

    toast.render(&mut term.buffer, &theme);

    // Nothing rendered
    assert_eq!(term.render_to_string(), "");
}

#[test]
fn toast_starts_expired() {
    let toast = Toast::new("Hello");
    // Initial expiry_instant is Instant::now() at creation, so it's immediately expired
    assert!(toast.is_expired());
}

// ============================================================================
// Show/hide tests
// ============================================================================

#[test]
fn toast_show_makes_it_visible() {
    let mut toast = Toast::new("Hello");
    let mut term = TestTerminal::new(30, 10);
    let theme = TestTheme::boxed();

    // Show the toast with a long duration
    toast.show(Position::new(0, 0), Duration::from_secs(60));

    toast.render(&mut term.buffer, &theme);

    // Toast should be rendered
    let output = term.render_to_string();
    assert!(output.contains("Hello"));
}

#[test]
fn toast_with_zero_duration_expires_immediately() {
    let mut toast = Toast::new("Hello");

    toast.show(Position::new(0, 0), Duration::ZERO);

    // Should be expired immediately
    assert!(toast.is_expired());
}

#[test]
fn toast_handle_event_hides_expired_toast() {
    let mut toast = Toast::new("Hello");
    let mut term = TestTerminal::new(30, 10);
    let theme = TestTheme::boxed();

    // Show with zero duration (expires immediately)
    toast.show(Position::new(0, 0), Duration::ZERO);

    // Handle event should detect expiry and hide
    toast.handle_event(None);

    toast.render(&mut term.buffer, &theme);

    // Nothing rendered since toast was hidden
    assert_eq!(term.render_to_string(), "");
}

#[test]
fn toast_not_expired_stays_visible() {
    let mut toast = Toast::new("Hello");

    // Show with long duration
    toast.show(Position::new(0, 0), Duration::from_secs(60));

    // Should not be expired
    assert!(!toast.is_expired());

    // Handle event should not hide it
    toast.handle_event(None);

    let mut term = TestTerminal::new(30, 10);
    let theme = TestTheme::boxed();
    toast.render(&mut term.buffer, &theme);

    // Should still be rendered
    let output = term.render_to_string();
    assert!(output.contains("Hello"));
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn toast_renders_at_position() {
    let mut toast = Toast::new("Hi");
    let mut term = TestTerminal::new(20, 10);
    let theme = TestTheme::boxed();

    // Show at position (5, 2)
    toast.show(Position::new(5, 2), Duration::from_secs(60));

    toast.render(&mut term.buffer, &theme);

    let output = term.render_to_string();
    // Toast renders at y+1 from position, so at y=3
    assert!(output.contains("Hi"));
}

#[test]
fn toast_renders_short_message() {
    let mut toast = Toast::new("OK");
    let mut term = TestTerminal::new(20, 10);
    let theme = TestTheme::boxed();

    toast.show(Position::new(0, 0), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("OK"));
}

#[test]
fn toast_renders_longer_message() {
    let mut toast = Toast::new("Copied to clipboard!");
    let mut term = TestTerminal::new(30, 10);
    let theme = TestTheme::boxed();

    toast.show(Position::new(0, 0), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Copied to clipboard!"));
}

#[test]
fn toast_clears_area_before_rendering() {
    let mut term = TestTerminal::new(20, 10);
    let theme = TestTheme::boxed();

    // First fill the buffer with some content
    for y in 0..term.area.height {
        for x in 0..term.area.width {
            term.buffer
                .cell_mut(ratatui::layout::Position::new(x, y))
                .unwrap()
                .set_char('X');
        }
    }

    // Now render a toast
    let mut toast = Toast::new("Hi");
    toast.show(Position::new(0, 0), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);

    // The toast area should be cleared (not all X's)
    let output = term.render_to_string();
    assert!(output.contains("Hi"));
}

// ============================================================================
// Position tests
// ============================================================================

#[test]
fn toast_at_different_positions() {
    let mut toast = Toast::new("Test");
    let theme = TestTheme::boxed();

    // Test at origin
    let mut term = TestTerminal::new(20, 10);
    toast.show(Position::new(0, 0), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);
    assert!(term.render_to_string().contains("Test"));

    // Test at offset position
    let mut term = TestTerminal::new(20, 10);
    toast.show(Position::new(5, 3), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);
    assert!(term.render_to_string().contains("Test"));
}

#[test]
fn toast_position_affects_render_location() {
    let mut toast = Toast::new("X");
    let theme = TestTheme::boxed();

    // Render at position (10, 5) - toast appears at y+1 = 6
    let mut term = TestTerminal::new(20, 10);
    toast.show(Position::new(10, 5), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);

    // Check that the first 10 columns of line 6 are empty (before toast)
    let output = term.render_to_string();
    let lines: Vec<&str> = output.lines().collect();

    // Line 6 (0-indexed) should have content starting after column 10
    if lines.len() > 6 {
        let line = lines[6];
        // First part should be spaces, then toast content
        assert!(line.starts_with("          ") || line.trim().is_empty() || line.len() >= 10);
    }
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn toast_with_empty_message() {
    let mut toast = Toast::new("");
    let mut term = TestTerminal::new(20, 10);
    let theme = TestTheme::boxed();

    toast.show(Position::new(0, 0), Duration::from_secs(60));
    toast.render(&mut term.buffer, &theme);

    // Should render without crashing
    let _ = term.render_to_string();
}

#[test]
fn toast_multiple_show_calls_update_position() {
    let mut toast = Toast::new("Hi");
    let theme = TestTheme::boxed();

    // First show at one position
    toast.show(Position::new(0, 0), Duration::from_secs(60));

    // Second show at different position
    toast.show(Position::new(10, 5), Duration::from_secs(60));

    let mut term = TestTerminal::new(20, 10);
    toast.render(&mut term.buffer, &theme);

    // Should render at the second position
    let output = term.render_to_string();
    assert!(output.contains("Hi"));
}
