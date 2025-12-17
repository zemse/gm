use crate::testutils::*;
use crate::widgets::scroll_bar::CustomScrollBar;

// ============================================================================
// Basic rendering tests
// ============================================================================

#[test]
fn render_scrollbar_at_top() {
    let mut term = TestTerminal::new(5, 10);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 0,
        total_items: 20,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    // At top: scroll indicator at top, rest is track
    let expected = "\
█
║
║
║
║
║
║
║
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_at_bottom() {
    let mut term = TestTerminal::new(5, 10);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 10,
        total_items: 20,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    // Cursor 10 out of 20 items with height 10 maps to middle area
    let expected = "\
║
║
║
║
║
█
║
║
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_at_middle() {
    let mut term = TestTerminal::new(5, 10);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 5,
        total_items: 20,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    // Cursor 5 out of 20 items maps to position 2
    let expected = "\
║
║
█
║
║
║
║
║
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_few_items() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 0,
        total_items: 5,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    // When total_items <= height, each item gets its own line
    let expected = "\
█
║
║
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_few_items_middle() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 2,
        total_items: 5,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    let expected = "\
║
║
█
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_few_items_last() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 4,
        total_items: 5,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    let expected = "\
║
║
║
║
█";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Paginate mode tests
// ============================================================================

#[test]
fn render_scrollbar_paginate_first_page() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    // 15 items with height 5 = 3 pages
    CustomScrollBar {
        cursor: 0,
        total_items: 15,
        paginate: true,
    }
    .render(term.area, &mut term.buffer, &theme);

    // First page indicator
    let expected = "\
█
█
║
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_paginate_second_page() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    // cursor 5 = page 1 (second page, 0-indexed)
    CustomScrollBar {
        cursor: 5,
        total_items: 15,
        paginate: true,
    }
    .render(term.area, &mut term.buffer, &theme);

    // Page 1 indicator spans 2 lines
    let expected = "\
║
║
█
█
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_paginate_last_page() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    // cursor 10 = page 2 (third/last page)
    CustomScrollBar {
        cursor: 10,
        total_items: 15,
        paginate: true,
    }
    .render(term.area, &mut term.buffer, &theme);

    // Last page indicator
    let expected = "\
║
║
║
║
█";
    assert_eq!(term.render_to_string(), expected);
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn render_scrollbar_single_item() {
    let mut term = TestTerminal::new(5, 5);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 0,
        total_items: 1,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    // Single item fills the entire scrollbar
    let expected = "\
█
█
█
█
█";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_two_items() {
    let mut term = TestTerminal::new(5, 4);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 0,
        total_items: 2,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    let expected = "\
█
█
║
║";
    assert_eq!(term.render_to_string(), expected);
}

#[test]
fn render_scrollbar_two_items_second() {
    let mut term = TestTerminal::new(5, 4);
    let theme = TestTheme::boxed();

    CustomScrollBar {
        cursor: 1,
        total_items: 2,
        paginate: false,
    }
    .render(term.area, &mut term.buffer, &theme);

    let expected = "\
║
║
█
█";
    assert_eq!(term.render_to_string(), expected);
}
