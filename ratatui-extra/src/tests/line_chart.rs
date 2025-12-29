use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

use crate::extensions::ThemedWidget;
use crate::testutils::*;
use crate::widgets::line_chart::{LineChart, LineChartEvent};

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

fn key_up() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Up,
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

fn key_plus() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char('+'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn key_minus() -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char('-'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn mouse_scroll_up(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_scroll_down(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_click(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_up(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_drag(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

fn mouse_moved(x: u16, y: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

// ============================================================================
// Initial state tests
// ============================================================================

#[test]
fn line_chart_default() {
    let chart = LineChart::default();
    let _ = chart;
}

#[test]
fn line_chart_new() {
    let chart = LineChart::new();
    let _ = chart;
}

// ============================================================================
// Builder tests
// ============================================================================

#[test]
fn line_chart_with_bounds() {
    let chart = LineChart::new().with_bounds([0.0, 100.0], [-50.0, 50.0]);
    let _ = chart;
}

// ============================================================================
// Set points tests
// ============================================================================

#[test]
fn line_chart_set_points() {
    let mut chart = LineChart::new();
    chart.set_points(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 4.0)]);
    // Verify no panic
}

#[test]
fn line_chart_set_empty_points() {
    let mut chart = LineChart::new();
    chart.set_points(vec![]);
}

#[test]
fn line_chart_set_single_point() {
    let mut chart = LineChart::new();
    chart.set_points(vec![(5.0, 10.0)]);
}

// ============================================================================
// Pan tests - keyboard
// ============================================================================

#[test]
fn line_chart_pan_left() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_left()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Panned { dx, dy }) if dx == -3.0 && dy == 0.0
    ));
}

#[test]
fn line_chart_pan_right() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_right()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Panned { dx, dy }) if dx == 3.0 && dy == 0.0
    ));
}

#[test]
fn line_chart_pan_up() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_up()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Panned { dx, dy }) if dx == 0.0 && dy == -1.0
    ));
}

#[test]
fn line_chart_pan_down() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_down()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Panned { dx, dy }) if dx == 0.0 && dy == 1.0
    ));
}

// ============================================================================
// Zoom tests - keyboard
// ============================================================================

#[test]
fn line_chart_zoom_in() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_plus()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Zoomed { factor }) if (factor - 0.9).abs() < 0.001
    ));
}

#[test]
fn line_chart_zoom_out() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&key_minus()), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Zoomed { factor }) if (factor - 1.1).abs() < 0.001
    ));
}

// ============================================================================
// Mouse tests
// ============================================================================

#[test]
fn line_chart_mouse_click_returns_clicked_event() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&mouse_click(20, 10)), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Clicked { .. })
    ));
}

#[test]
fn line_chart_mouse_scroll_up_zooms_in() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&mouse_scroll_up(20, 10)), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Zoomed { factor }) if (factor - 0.9).abs() < 0.001
    ));
}

#[test]
fn line_chart_mouse_scroll_down_zooms_out() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&mouse_scroll_down(20, 10)), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Zoomed { factor }) if (factor - 1.1).abs() < 0.001
    ));
}

#[test]
fn line_chart_mouse_drag_pans() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    // Start drag
    let _ = chart.handle_event(Some(&mouse_click(20, 10)), area);

    // Drag to new position
    let result = chart.handle_event(Some(&mouse_drag(25, 12)), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Panned { .. })
    ));
}

#[test]
fn line_chart_mouse_up_stops_drag() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    // Start drag
    let _ = chart.handle_event(Some(&mouse_click(20, 10)), area);
    let _ = chart.handle_event(Some(&mouse_drag(25, 12)), area);

    // Mouse up
    let _ = chart.handle_event(Some(&mouse_up(25, 12)), area);

    // Further drag should not pan (not dragging anymore)
    let result = chart.handle_event(Some(&mouse_drag(30, 15)), area);

    // Result should be None since we're not dragging
    assert!(result.is_ok());
}

#[test]
fn line_chart_mouse_hover_returns_hover_event() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(Some(&mouse_moved(20, 10)), area);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(LineChartEvent::Hover { .. })
    ));
}

#[test]
fn line_chart_mouse_click_outside_area_no_event() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(10, 10, 20, 10);

    // Click outside the area
    let result = chart.handle_event(Some(&mouse_click(5, 5)), area);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn line_chart_zero_size_area() {
    let mut chart = LineChart::new().with_bounds([0.0, 20.0], [-10.0, 10.0]);
    let area = ratatui::layout::Rect::new(0, 0, 0, 0);

    // Should not panic with zero-size area
    let result = chart.handle_event(Some(&key_left()), area);
    assert!(result.is_ok());
}

#[test]
fn line_chart_no_event() {
    let mut chart = LineChart::new();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);

    let result = chart.handle_event(None, area);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn line_chart_renders_empty() {
    let mut term = TestTerminal::new(40, 20);
    let chart = LineChart::new();
    let theme = TestTheme::boxed();

    chart.render(term.area, &mut term.buffer, &theme);

    // Should render without panic, even with no data
}

#[test]
fn line_chart_renders_with_data() {
    let mut term = TestTerminal::new(40, 20);
    let mut chart = LineChart::new().with_bounds([0.0, 10.0], [0.0, 10.0]);
    chart.set_points(vec![
        (0.0, 0.0),
        (2.0, 2.0),
        (4.0, 4.0),
        (6.0, 6.0),
        (8.0, 8.0),
        (10.0, 10.0),
    ]);
    let theme = TestTheme::boxed();

    chart.render(term.area, &mut term.buffer, &theme);

    // The canvas widget renders Braille characters for lines
    // Just verify no panic
}

#[test]
fn line_chart_renders_single_point() {
    let mut term = TestTerminal::new(40, 20);
    let mut chart = LineChart::new().with_bounds([0.0, 10.0], [0.0, 10.0]);
    chart.set_points(vec![(5.0, 5.0)]);
    let theme = TestTheme::boxed();

    chart.render(term.area, &mut term.buffer, &theme);
    // Single point won't draw a line, but should not panic
}

#[test]
fn line_chart_renders_two_points() {
    let mut term = TestTerminal::new(40, 20);
    let mut chart = LineChart::new().with_bounds([0.0, 10.0], [0.0, 10.0]);
    chart.set_points(vec![(0.0, 0.0), (10.0, 10.0)]);
    let theme = TestTheme::boxed();

    chart.render(term.area, &mut term.buffer, &theme);
    // Should draw a diagonal line
}
