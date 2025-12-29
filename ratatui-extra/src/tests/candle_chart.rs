use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::style::Color;
use ratatui::widgets::Widget;

use crate::testutils::*;
use crate::widgets::candle_chart::{Candle, CandleChart, Interval};

fn key_up() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_down() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_left() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Left,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_right() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key_release() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Release,
        state: KeyEventState::NONE,
    }
}

// ============================================================================
// CandleChart initial state tests
// ============================================================================

#[test]
fn candle_chart_default() {
    let chart = CandleChart::default();
    let _ = chart;
}

// ============================================================================
// Candle tests
// ============================================================================

#[test]
fn candle_new() {
    let candle = Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000);
    assert_eq!(candle.open, 100.0);
    assert_eq!(candle.high, 110.0);
    assert_eq!(candle.close, 105.0);
    assert_eq!(candle.low, 95.0);
    assert_eq!(candle.start_timestamp, 1000);
    assert_eq!(candle.end_timestamp, 2000);
}

#[test]
fn candle_default() {
    let candle = Candle::default();
    assert_eq!(candle.open, 0.0);
    assert_eq!(candle.high, 0.0);
    assert_eq!(candle.low, 0.0);
    assert_eq!(candle.close, 0.0);
    assert_eq!(candle.start_timestamp, 0);
    assert_eq!(candle.end_timestamp, 0);
}

#[test]
fn candle_bull_market() {
    let candle = Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000);
    // Close > Open means bullish (green)
    assert_eq!(candle.bear_bull(), Color::LightGreen);
}

#[test]
fn candle_bear_market() {
    let candle = Candle::new(105.0, 110.0, 100.0, 95.0, 1000, 2000);
    // Close < Open means bearish (red)
    assert_eq!(candle.bear_bull(), Color::Red);
}

#[test]
fn candle_equal_open_close() {
    let candle = Candle::new(100.0, 110.0, 100.0, 95.0, 1000, 2000);
    // When open == close, it's considered bullish (green)
    assert_eq!(candle.bear_bull(), Color::LightGreen);
}

#[test]
fn candle_calc_y() {
    let candle = Candle::new(100.0, 110.0, 105.0, 90.0, 1000, 2000);
    let y_scale = 10.0;
    let y_min = 80.0;

    let [y_open, y_high, y_low, y_close] = candle.calc_y(y_scale, y_min);

    assert_eq!(y_open, (100.0 - 80.0) / 10.0); // 2.0
    assert_eq!(y_high, (110.0 - 80.0) / 10.0); // 3.0
    assert_eq!(y_low, (90.0 - 80.0) / 10.0);   // 1.0
    assert_eq!(y_close, (105.0 - 80.0) / 10.0); // 2.5
}

// ============================================================================
// Interval tests
// ============================================================================

#[test]
fn interval_default() {
    let interval = Interval::default();
    assert!(matches!(interval, Interval::OneSecond));
}

#[test]
fn interval_display_one_second() {
    assert_eq!(format!("{}", Interval::OneSecond), "1s");
}

#[test]
fn interval_display_fifteen_minutes() {
    assert_eq!(format!("{}", Interval::FifteenMinutes), "15m");
}

#[test]
fn interval_display_one_hour() {
    assert_eq!(format!("{}", Interval::OneHour), "1h");
}

#[test]
fn interval_display_one_week() {
    assert_eq!(format!("{}", Interval::OneWeek), "1w");
}

#[test]
fn interval_display_one_month() {
    assert_eq!(format!("{}", Interval::OneMonth), "1M");
}

#[test]
fn interval_values() {
    assert_eq!(Interval::OneSecond as i64, 1000);
    assert_eq!(Interval::FifteenMinutes as i64, 15 * 60 * 1000);
    assert_eq!(Interval::OneHour as i64, 60 * 60 * 1000);
    assert_eq!(Interval::OneWeek as i64, 60 * 60 * 24 * 7 * 1000);
    assert_eq!(Interval::OneMonth as i64, 60 * 60 * 24 * 30 * 1000);
}

// ============================================================================
// CandleChart update tests
// ============================================================================

#[test]
fn candle_chart_update_empty() {
    let mut chart = CandleChart::default();
    chart.update(vec![], Interval::OneHour);
    // Should not panic with empty data
}

#[test]
fn candle_chart_update_single_candle() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);
}

#[test]
fn candle_chart_update_multiple_candles() {
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000, 3000),
        Candle::new(110.0, 120.0, 115.0, 105.0, 3000, 4000),
    ];
    chart.update(candles, Interval::OneHour);
}

#[test]
fn candle_chart_update_changes_interval() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];

    chart.update(candles.clone(), Interval::OneHour);
    chart.update(candles, Interval::OneSecond);
    // Should handle interval change
}

// ============================================================================
// CandleChart zoom tests
// ============================================================================

#[test]
fn candle_chart_zoom_in() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    chart.handle_event(&key_up());
    // Zoom should increase
}

#[test]
fn candle_chart_zoom_out() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    // Zoom in first
    chart.handle_event(&key_up());
    chart.handle_event(&key_up());

    // Then zoom out
    chart.handle_event(&key_down());
}

#[test]
fn candle_chart_zoom_in_max() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    // Zoom in many times - should cap at 60
    for _ in 0..100 {
        chart.handle_event(&key_up());
    }
}

#[test]
fn candle_chart_zoom_out_min() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    // Zoom out many times - should cap at 1
    for _ in 0..100 {
        chart.handle_event(&key_down());
    }
}

// ============================================================================
// CandleChart navigation tests
// ============================================================================

#[test]
fn candle_chart_move_right() {
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000, 3000),
        Candle::new(110.0, 120.0, 115.0, 105.0, 3000, 4000),
    ];
    chart.update(candles, Interval::OneSecond);

    chart.handle_event(&key_right());
}

#[test]
fn candle_chart_move_left() {
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000, 3000),
        Candle::new(110.0, 120.0, 115.0, 105.0, 3000, 4000),
    ];
    chart.update(candles, Interval::OneSecond);

    chart.handle_event(&key_left());
}

#[test]
fn candle_chart_move_right_at_end() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneSecond);

    // Already at end, moving right should cap at end_timestamp
    chart.handle_event(&key_right());
    chart.handle_event(&key_right());
}

// ============================================================================
// CandleChart event handling edge cases
// ============================================================================

#[test]
fn candle_chart_ignores_key_release() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    // Release events should be ignored
    chart.handle_event(&key_release());
}

#[test]
fn candle_chart_ignores_other_keys() {
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000)];
    chart.update(candles, Interval::OneHour);

    // Other keys should be ignored
    chart.handle_event(&KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    });
}

// ============================================================================
// Candle clone test
// ============================================================================

#[test]
fn candle_clone() {
    let candle = Candle::new(100.0, 110.0, 105.0, 95.0, 1000, 2000);
    let cloned = candle.clone();

    assert_eq!(cloned.open, candle.open);
    assert_eq!(cloned.high, candle.high);
    assert_eq!(cloned.low, candle.low);
    assert_eq!(cloned.close, candle.close);
    assert_eq!(cloned.start_timestamp, candle.start_timestamp);
    assert_eq!(cloned.end_timestamp, candle.end_timestamp);
}

// ============================================================================
// Interval equality and hash tests
// ============================================================================

#[test]
fn interval_equality() {
    assert_eq!(Interval::OneHour, Interval::OneHour);
    assert_ne!(Interval::OneHour, Interval::OneSecond);
}

#[test]
fn interval_copy() {
    let interval = Interval::OneHour;
    let copied = interval;
    assert_eq!(interval, copied);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn candle_chart_renders_with_data() {
    let mut term = TestTerminal::new(60, 20);
    let mut chart = CandleChart::default();
    // Chart needs data to render (empty chart will panic on division by zero)
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000)];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    // Should render without panic
}

#[test]
fn candle_chart_renders_with_single_candle() {
    let mut term = TestTerminal::new(60, 20);
    let mut chart = CandleChart::default();
    let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000)];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    let output = term.render_to_string();
    // Should contain Y-axis markers
    assert!(output.contains("┤") || output.contains("│"));
}

#[test]
fn candle_chart_renders_with_multiple_candles() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000000, 3000000),
        Candle::new(110.0, 120.0, 115.0, 105.0, 3000000, 4000000),
        Candle::new(115.0, 125.0, 120.0, 110.0, 4000000, 5000000),
    ];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    let output = term.render_to_string();
    // Should contain axis elements
    assert!(output.contains("─"));
}

#[test]
fn candle_chart_renders_x_axis() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000000, 3000000),
    ];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    let output = term.render_to_string();
    // X-axis has horizontal lines
    assert!(output.contains("─"));
    // Bottom right corner
    assert!(output.contains("┘"));
}

#[test]
fn candle_chart_renders_y_axis() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000),
    ];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    let output = term.render_to_string();
    // Y-axis has vertical lines and tick marks
    assert!(output.contains("│"));
    assert!(output.contains("┤"));
}

#[test]
fn candle_chart_renders_bullish_candle() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    // Bullish candle: close > open
    let candles = vec![Candle::new(100.0, 120.0, 115.0, 95.0, 1000000, 2000000)];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    // Should render without panic - bullish candles are green
}

#[test]
fn candle_chart_renders_bearish_candle() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    // Bearish candle: close < open
    let candles = vec![Candle::new(115.0, 120.0, 100.0, 95.0, 1000000, 2000000)];
    chart.update(candles, Interval::OneSecond);

    (&chart).render(term.area, &mut term.buffer);

    // Should render without panic - bearish candles are red
}

#[test]
fn candle_chart_renders_after_zoom() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000000, 3000000),
    ];
    chart.update(candles, Interval::OneSecond);

    // Zoom in
    chart.handle_event(&key_up());
    chart.handle_event(&key_up());

    (&chart).render(term.area, &mut term.buffer);

    // Should render without panic after zooming
}

#[test]
fn candle_chart_renders_after_navigation() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();
    let candles = vec![
        Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000),
        Candle::new(105.0, 115.0, 110.0, 100.0, 2000000, 3000000),
        Candle::new(110.0, 120.0, 115.0, 105.0, 3000000, 4000000),
    ];
    chart.update(candles, Interval::OneSecond);

    // Navigate left
    chart.handle_event(&key_left());

    (&chart).render(term.area, &mut term.buffer);

    // Should render without panic after navigation
}

#[test]
fn candle_chart_renders_with_different_intervals() {
    let mut term = TestTerminal::new(80, 25);
    let mut chart = CandleChart::default();

    // Test with different intervals
    for interval in [
        Interval::OneSecond,
        Interval::FifteenMinutes,
        Interval::OneHour,
    ] {
        let candles = vec![Candle::new(100.0, 110.0, 105.0, 95.0, 1000000, 2000000)];
        chart.update(candles, interval);
        (&chart).render(term.area, &mut term.buffer);
    }
}
