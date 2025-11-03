use std::borrow::Cow;

use gm_utils::text_wrap::text_wrap;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    layout::{Position, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Widget, WidgetRef},
};

use crate::thematize::Thematize;

pub trait ThemedWidget {
    fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize);
}

pub trait BorderedWidget<A> {
    fn render_with_block(
        &self,
        area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        leave_horizontal_space: bool,
        args: A,
    ) where
        Self: Sized;
}

impl<T: ThemedWidget, Theme: Thematize> BorderedWidget<&Theme> for T {
    fn render_with_block(
        &self,
        area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        leave_horizontal_space: bool,
        theme: &Theme,
    ) where
        Self: Sized,
    {
        let inner_area = block
            .inner(area)
            .margin_h(if leave_horizontal_space { 1 } else { 0 });
        block.render_ref(area, buf);
        self.render(inner_area, buf, theme);
    }
}

impl<T: WidgetRef> BorderedWidget<()> for T {
    fn render_with_block(
        &self,
        area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        leave_horizontal_space: bool,
        _: (),
    ) where
        Self: Sized,
    {
        let inner_area = block
            .inner(area)
            .margin_h(if leave_horizontal_space { 1 } else { 0 });
        block.render_ref(area, buf);
        self.render_ref(inner_area, buf);
    }
}

pub trait RenderTextWrapped {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer);
}

fn render_wrapped_lines(
    lines: Vec<Cow<'_, str>>,
    style: Option<Style>,
    area: Rect,
    buf: &mut Buffer,
) {
    let mut text = Text::default();
    for line in lines {
        let mut line = Line::raw(line);
        if let Some(style) = style {
            line = line.style(style);
        }
        text.push_line(line);
    }
    text.render(area, buf);
}

impl RenderTextWrapped for &str {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = text_wrap(self, area.width);

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl RenderTextWrapped for String {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = text_wrap(self, area.width);

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl RenderTextWrapped for Vec<&str> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = self
            .iter()
            .flat_map(|str| text_wrap(str, area.width))
            .collect();

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl RenderTextWrapped for Vec<String> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = self
            .iter()
            .flat_map(|str| text_wrap(str, area.width))
            .collect();

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl<'a> RenderTextWrapped for Span<'a> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = text_wrap(&self.content, area.width);

        render_wrapped_lines(lines, Some(self.style), area, buf);
    }
}

impl<'a> RenderTextWrapped for Cow<'a, str> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = text_wrap(self, area.width);

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl<const N: usize> RenderTextWrapped for [&str; N] {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = self
            .iter()
            .flat_map(|str| text_wrap(str, area.width))
            .collect();

        render_wrapped_lines(lines, None, area, buf);
    }
}

impl<const N: usize> RenderTextWrapped for [String; N] {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let lines = self
            .iter()
            .flat_map(|str| text_wrap(str, area.width))
            .collect();

        render_wrapped_lines(lines, None, area, buf);
    }
}

pub trait WidgetHeight {
    fn height_used(&self, area: Rect) -> u16;
}

pub trait RectExt {
    fn change_height(self, new_height: u16) -> Rect;

    fn change_width(self, new_width: u16) -> Rect;

    fn margin_h(self, m: u16) -> Rect;

    fn margin_top(self, m: u16) -> Rect;

    fn margin_down(self, m: u16) -> Rect;

    fn margin_left(self, m: u16) -> Rect;

    fn margin_right(self, m: u16) -> Rect;

    fn expand_vertical(self, m: u16) -> Rect;

    fn block_inner(self) -> Rect;

    fn button_center(self, label_len: usize) -> Rect;
}

impl RectExt for Rect {
    #[track_caller]
    fn change_height(self, new_height: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: new_height,
        }
    }

    #[track_caller]
    fn change_width(self, new_width: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: new_width,
            height: self.height,
        }
    }

    #[track_caller]
    fn margin_h(self, x: u16) -> Rect {
        Rect {
            x: self.x + x,
            y: self.y,
            width: self.width - 2 * x,
            height: self.height,
        }
    }

    #[track_caller]
    fn margin_top(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y + m,
            width: self.width,
            height: self.height - m,
        }
    }

    #[track_caller]
    fn margin_down(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height - m,
        }
    }

    #[track_caller]
    fn margin_left(self, m: u16) -> Rect {
        Rect {
            x: self.x + m,
            y: self.y,
            width: self.width - m,
            height: self.height,
        }
    }

    #[track_caller]
    fn margin_right(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width - m,
            height: self.height,
        }
    }

    #[track_caller]
    fn expand_vertical(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y - m,
            width: self.width,
            height: self.height + 2 * m,
        }
    }

    #[track_caller]
    fn block_inner(self) -> Rect {
        Rect {
            x: self.x + 1,
            y: self.y + 1,
            width: self.width - 2,
            height: self.height - 2,
        }
    }

    #[track_caller]
    fn button_center(self, label_len: usize) -> Rect {
        let button_width = (label_len + 2) as u16;
        let x = self.x + (self.width.saturating_sub(button_width)) / 2;
        Rect {
            x,
            y: self.y,
            width: button_width,
            height: self.height,
        }
    }
}

pub trait EventExt {
    fn is_key_press(&self) -> bool;

    fn is_mouse_left_click_or_hover(&self) -> bool;

    fn is_mouse_left_click(&self) -> bool;

    fn is_mouse_hover(&self) -> bool;

    fn key_event(&self) -> Option<&KeyEvent>;

    fn is_key_pressed(&self, key: KeyCode) -> bool;

    fn key_code(&self) -> Option<KeyCode>;
}

impl EventExt for Event {
    fn is_key_press(&self) -> bool {
        matches!(
            self,
            Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                ..
            })
        )
    }

    fn is_mouse_left_click_or_hover(&self) -> bool {
        matches!(
            self,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Moved,
                ..
            })
        )
    }

    fn is_mouse_left_click(&self) -> bool {
        matches!(
            self,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                ..
            })
        )
    }

    fn is_mouse_hover(&self) -> bool {
        matches!(
            self,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Moved,
                ..
            })
        )
    }

    fn key_event(&self) -> Option<&KeyEvent> {
        if let Event::Key(key_event) = self {
            Some(key_event)
        } else {
            None
        }
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.key_event()
            .is_some_and(|ke| ke.kind == KeyEventKind::Press && ke.code == key)
    }

    fn key_code(&self) -> Option<KeyCode> {
        self.key_event().map(|ke| ke.code)
    }
}

impl EventExt for Option<&Event> {
    fn is_key_press(&self) -> bool {
        self.map(|event| event.is_key_press()).unwrap_or(false)
    }

    fn is_mouse_left_click_or_hover(&self) -> bool {
        self.map(|event| event.is_mouse_left_click_or_hover())
            .unwrap_or(false)
    }

    fn is_mouse_left_click(&self) -> bool {
        self.map(|event| event.is_mouse_left_click())
            .unwrap_or(false)
    }

    fn is_mouse_hover(&self) -> bool {
        self.map(|event| event.is_mouse_hover()).unwrap_or(false)
    }

    fn key_event(&self) -> Option<&KeyEvent> {
        self.and_then(|event| event.key_event())
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.map(|event| event.is_key_pressed(key)).unwrap_or(false)
    }

    fn key_code(&self) -> Option<KeyCode> {
        self.and_then(|event| event.key_code())
    }
}

pub trait KeyEventExt {
    fn is_pressed(&self, key: KeyCode) -> bool;
}

impl KeyEventExt for Option<&KeyEvent> {
    fn is_pressed(&self, key: KeyCode) -> bool {
        self.map(|key_event| key_event.is_pressed(key))
            .unwrap_or(false)
    }
}

impl KeyEventExt for &KeyEvent {
    fn is_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self,
            KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers: KeyModifiers::NONE,
                ..
            } if *code == key
        )
    }
}

pub trait MouseEventExt {
    fn is_left_click(&self) -> bool;

    fn is(&self, kind: MouseEventKind) -> bool;

    fn position(&self) -> Position;
}

impl MouseEventExt for MouseEvent {
    #[inline]
    fn is_left_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Left))
    }

    #[inline]
    fn is(&self, kind: MouseEventKind) -> bool {
        self.kind == kind
    }

    #[inline]
    fn position(&self) -> Position {
        Position {
            x: self.column,
            y: self.row,
        }
    }
}

pub trait PositionExt {
    fn sort(self, other: Position) -> (Position, Position);

    fn nearest_inner(self, area: Rect) -> Position;
}

impl PositionExt for Position {
    /// Returns the two positions in top-to-bottom, left-to-right order.
    ///
    /// If `self` appears after `other` in reading order, the positions are swapped
    /// so that the returned tuple is always `(start, end)`.
    fn sort(self, other: Position) -> (Position, Position) {
        if self.y > other.y || (self.y == other.y && self.x > other.x) {
            (other, self)
        } else {
            (self, other)
        }
    }

    fn nearest_inner(self, area: Rect) -> Position {
        if area.contains(self) {
            self
        } else if self.x < area.x {
            if self.y < area.y {
                Position::new(area.x, area.y)
            } else {
                Position::new(area.x, self.y.min(area.y + area.height - 1))
            }
        } else if self.y < area.y {
            Position::new(self.x.min(area.x + area.width), area.y)
        } else {
            Position::new(
                self.x.min(area.x + area.width),
                self.y.min(area.y + area.height - 1),
            )
        }
    }
}
