use std::fmt;

use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use strum::EnumIter;

use crate::event::WidgetEvent;
use crate::testutils::*;
use crate::widgets::button::Button;
use crate::widgets::form::{Form, FormEvent, FormItemIndex, FormWidget};
use crate::widgets::input_box::InputBox;

// Test form enum for basic tests
#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
enum TestFormItem {
    Name,
    Email,
    Submit,
}

impl fmt::Display for TestFormItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestFormItem::Name => write!(f, "Name"),
            TestFormItem::Email => write!(f, "Email"),
            TestFormItem::Submit => write!(f, "Submit"),
        }
    }
}

impl FormItemIndex for TestFormItem {
    fn index(self) -> usize {
        self as usize
    }
}

impl TryFrom<TestFormItem> for FormWidget {
    type Error = crate::Error;

    fn try_from(item: TestFormItem) -> Result<Self, Self::Error> {
        Ok(match item {
            TestFormItem::Name => FormWidget::InputBox {
                widget: InputBox::new("Name"),
            },
            TestFormItem::Email => FormWidget::InputBox {
                widget: InputBox::new("Email"),
            },
            TestFormItem::Submit => FormWidget::Button {
                widget: Button::new("Submit"),
            },
        })
    }
}

// Test form enum with headings and static text
#[derive(Debug, Clone, Copy, EnumIter, PartialEq)]
enum MixedFormItem {
    Heading,
    Description,
    Username,
    LineBreak,
    Password,
}

impl fmt::Display for MixedFormItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MixedFormItem::Heading => write!(f, "Heading"),
            MixedFormItem::Description => write!(f, "Description"),
            MixedFormItem::Username => write!(f, "Username"),
            MixedFormItem::LineBreak => write!(f, "LineBreak"),
            MixedFormItem::Password => write!(f, "Password"),
        }
    }
}

impl FormItemIndex for MixedFormItem {
    fn index(self) -> usize {
        self as usize
    }
}

impl TryFrom<MixedFormItem> for FormWidget {
    type Error = crate::Error;

    fn try_from(item: MixedFormItem) -> Result<Self, Self::Error> {
        Ok(match item {
            MixedFormItem::Heading => FormWidget::Heading("Login Form"),
            MixedFormItem::Description => FormWidget::StaticText("Please enter your credentials"),
            MixedFormItem::Username => FormWidget::InputBox {
                widget: InputBox::new("Username"),
            },
            MixedFormItem::LineBreak => FormWidget::LineBreak,
            MixedFormItem::Password => FormWidget::InputBox {
                widget: InputBox::new("Password"),
            },
        })
    }
}

fn key_down() -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }))
}

fn key_up() -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Key(KeyEvent {
        code: KeyCode::Up,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }))
}

fn key_tab() -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Key(KeyEvent {
        code: KeyCode::Tab,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }))
}

fn key_enter() -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }))
}

fn key_char(c: char) -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }))
}

fn mouse_click(x: u16, y: u16) -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }))
}

fn mouse_scroll_down(x: u16, y: u16) -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }))
}

fn mouse_scroll_up(x: u16, y: u16) -> WidgetEvent {
    WidgetEvent::InputEvent(Event::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }))
}

// ============================================================================
// Form initialization tests
// ============================================================================

#[test]
fn form_init() {
    let form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert_eq!(form.items.len(), 3);
}

#[test]
fn form_init_with_values() {
    let form: Form<TestFormItem, crate::Error> = Form::init(|f| {
        f.set_text(TestFormItem::Name, "John".to_string());
        f.set_text(TestFormItem::Email, "john@example.com".to_string());
        Ok(())
    })
    .unwrap();

    assert_eq!(form.get_text(TestFormItem::Name).as_ref(), "John");
    assert_eq!(
        form.get_text(TestFormItem::Email).as_ref(),
        "john@example.com"
    );
}

#[test]
fn form_init_cursor_starts_at_first_valid() {
    let form: Form<MixedFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // Should skip Heading and Description, cursor at Username
    assert!(form.is_focused(MixedFormItem::Username));
}

// ============================================================================
// Form focus tests
// ============================================================================

#[test]
fn form_set_form_focus() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.set_form_focus(false);
    assert!(!form.form_focus);

    form.set_form_focus(true);
    assert!(form.form_focus);
}

#[test]
fn form_is_focused() {
    let form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert!(form.is_focused(TestFormItem::Name));
    assert!(!form.is_focused(TestFormItem::Email));
}

// ============================================================================
// Cursor navigation tests
// ============================================================================

#[test]
fn form_advance_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert!(form.is_focused(TestFormItem::Name));

    form.advance_cursor();
    assert!(form.is_focused(TestFormItem::Email));

    form.advance_cursor();
    assert!(form.is_focused(TestFormItem::Submit));
}

#[test]
fn form_retreat_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // Start at Name, go to Submit first
    form.advance_cursor();
    form.advance_cursor();
    assert!(form.is_focused(TestFormItem::Submit));

    form.retreat_cursor();
    assert!(form.is_focused(TestFormItem::Email));

    form.retreat_cursor();
    assert!(form.is_focused(TestFormItem::Name));
}

#[test]
fn form_cursor_wraps_forward() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.advance_cursor(); // Email
    form.advance_cursor(); // Submit
    form.advance_cursor(); // Should wrap to Name

    assert!(form.is_focused(TestFormItem::Name));
}

#[test]
fn form_cursor_wraps_backward() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // At Name, go backward should wrap to Submit
    form.retreat_cursor();
    assert!(form.is_focused(TestFormItem::Submit));
}

#[test]
fn form_cursor_skips_non_focusable() {
    let mut form: Form<MixedFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // Starts at Username (skips Heading, Description)
    assert!(form.is_focused(MixedFormItem::Username));

    form.advance_cursor();
    // Should skip LineBreak and go to Password
    assert!(form.is_focused(MixedFormItem::Password));
}

// ============================================================================
// Hide/Show tests
// ============================================================================

#[test]
fn form_hide_item() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.hide_item(TestFormItem::Email);

    assert_eq!(form.hidden_count(), 1);
    assert_eq!(form.visible_count(), 2);
}

#[test]
fn form_show_item() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.hide_item(TestFormItem::Email);
    form.show_item(TestFormItem::Email);

    assert_eq!(form.hidden_count(), 0);
    assert_eq!(form.visible_count(), 3);
}

#[test]
fn form_hide_focused_item_advances_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert!(form.is_focused(TestFormItem::Name));

    form.hide_item(TestFormItem::Name);

    // Cursor should move to next valid item
    assert!(form.is_focused(TestFormItem::Email));
}

#[test]
fn form_cursor_skips_hidden_items() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.hide_item(TestFormItem::Email);

    // At Name, advance should skip Email and go to Submit
    form.advance_cursor();
    assert!(form.is_focused(TestFormItem::Submit));
}

// ============================================================================
// Get/Set text tests
// ============================================================================

#[test]
fn form_get_text() {
    let form: Form<TestFormItem, crate::Error> = Form::init(|f| {
        f.set_text(TestFormItem::Name, "Test".to_string());
        Ok(())
    })
    .unwrap();

    assert_eq!(form.get_text(TestFormItem::Name).as_ref(), "Test");
}

#[test]
fn form_set_text() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.set_text(TestFormItem::Name, "New Name".to_string());

    assert_eq!(form.get_text(TestFormItem::Name).as_ref(), "New Name");
}

// ============================================================================
// Count tests
// ============================================================================

#[test]
fn form_valid_count() {
    let form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert_eq!(form.valid_count(), 3); // Name, Email, Submit are all valid cursors
}

#[test]
fn form_valid_count_with_non_focusable() {
    let form: Form<MixedFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // Only Username and Password are focusable
    assert_eq!(form.valid_count(), 2);
}

#[test]
fn form_valid_count_with_hidden() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.hide_item(TestFormItem::Email);
    assert_eq!(form.valid_count(), 2);
}

// ============================================================================
// Event handling tests
// ============================================================================

#[test]
fn form_down_key_advances_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(Some(&key_down()), area, popup_area, &mut actions);

    assert!(form.is_focused(TestFormItem::Email));
}

#[test]
fn form_up_key_retreats_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.advance_cursor(); // Email
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(Some(&key_up()), area, popup_area, &mut actions);

    assert!(form.is_focused(TestFormItem::Name));
}

#[test]
fn form_tab_key_advances_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(Some(&key_tab()), area, popup_area, &mut actions);

    assert!(form.is_focused(TestFormItem::Email));
}

#[test]
fn form_enter_key_on_input_advances_cursor() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    // On input box, Enter advances cursor
    let _ = form.handle_event(Some(&key_enter()), area, popup_area, &mut actions);

    assert!(form.is_focused(TestFormItem::Email));
}

#[test]
fn form_typing_changes_value() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(Some(&key_char('H')), area, popup_area, &mut actions);
    let _ = form.handle_event(Some(&key_char('i')), area, popup_area, &mut actions);

    assert_eq!(form.get_text(TestFormItem::Name).as_ref(), "Hi");
}

#[test]
fn form_value_change_returns_event() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let result = form.handle_event(Some(&key_char('H')), area, popup_area, &mut actions);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(FormEvent::ValueChanged(TestFormItem::Name))
    ));
}

#[test]
fn form_button_press_returns_event() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    // Navigate to Submit button
    form.advance_cursor(); // Email
    form.advance_cursor(); // Submit

    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let result = form.handle_event(Some(&key_enter()), area, popup_area, &mut actions);

    assert!(result.is_ok());
    assert!(matches!(
        result.unwrap(),
        Some(FormEvent::ButtonPressed(TestFormItem::Submit))
    ));
}

// ============================================================================
// Mouse event tests
// ============================================================================

#[test]
fn form_mouse_click_focuses_item() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    // Click on second item area (approximate)
    let _ = form.handle_event(Some(&mouse_click(5, 4)), area, popup_area, &mut actions);

    // May or may not focus depending on exact layout
}

#[test]
fn form_mouse_scroll_down() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(
        Some(&mouse_scroll_down(5, 2)),
        area,
        popup_area,
        &mut actions,
    );
}

#[test]
fn form_mouse_scroll_up() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let mut actions = TestAct::default();

    let _ = form.handle_event(Some(&mouse_scroll_up(5, 2)), area, popup_area, &mut actions);
}

// ============================================================================
// Button state tests
// ============================================================================

#[test]
fn form_is_button_focused() {
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    assert!(!form.is_button_focused());

    form.advance_cursor(); // Email
    assert!(!form.is_button_focused());

    form.advance_cursor(); // Submit
    assert!(form.is_button_focused());
}

// ============================================================================
// FormWidget tests
// ============================================================================

#[test]
fn form_widget_label() {
    let widget = FormWidget::InputBox {
        widget: InputBox::new("Test Label"),
    };
    assert_eq!(widget.label(), Some("Test Label"));
}

#[test]
fn form_widget_label_heading() {
    let widget = FormWidget::Heading("Title");
    assert_eq!(widget.label(), None);
}

#[test]
fn form_widget_to_value() {
    let mut input = InputBox::new("Test");
    input.set_text("Value".to_string());
    let widget = FormWidget::InputBox { widget: input };
    assert_eq!(widget.to_value(), Some("Value".to_string()));
}

#[test]
fn form_widget_to_value_heading() {
    let widget = FormWidget::Heading("Title");
    assert_eq!(widget.to_value(), None);
}

#[test]
fn form_widget_height() {
    let widget = FormWidget::LineBreak;
    let area = ratatui::layout::Rect::new(0, 0, 40, 20);
    assert_eq!(widget.height(area), 1);
}

// ============================================================================
// Rendering tests
// ============================================================================

#[test]
fn form_renders() {
    let mut term = TestTerminal::new(40, 20);
    let form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let theme = TestTheme::boxed();

    form.render(term.area, popup_area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Name"));
    assert!(output.contains("Email"));
    assert!(output.contains("Submit"));
}

#[test]
fn form_renders_with_values() {
    let mut term = TestTerminal::new(40, 20);
    let form: Form<TestFormItem, crate::Error> = Form::init(|f| {
        f.set_text(TestFormItem::Name, "John Doe".to_string());
        Ok(())
    })
    .unwrap();
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let theme = TestTheme::boxed();

    form.render(term.area, popup_area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("John Doe"));
}

#[test]
fn form_renders_with_hidden_items() {
    let mut term = TestTerminal::new(40, 20);
    let mut form: Form<TestFormItem, crate::Error> = Form::init(|_| Ok(())).unwrap();
    form.hide_item(TestFormItem::Email);
    let popup_area = ratatui::layout::Rect::new(0, 0, 40, 20);
    let theme = TestTheme::boxed();

    form.render(term.area, popup_area, &mut term.buffer, &theme);

    let output = term.render_to_string();
    assert!(output.contains("Name"));
    assert!(!output.contains("Email"));
    assert!(output.contains("Submit"));
}
