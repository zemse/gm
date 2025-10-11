use gm_utils::text::split_string;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    layout::{Position, Rect},
    text::{Span, Text, ToLine},
    widgets::{Block, Paragraph, Widget, WidgetRef, Wrap},
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

impl RenderTextWrapped for &str {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::raw(*self))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);
    }
}

impl RenderTextWrapped for String {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::raw(self.as_str()))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);
    }
}

impl RenderTextWrapped for Vec<&str> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::raw(self.join("\n").as_str()))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);
    }
}

impl RenderTextWrapped for Vec<String> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(Text::raw(self.join("\n").as_str()))
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);
    }
}

impl<'a> RenderTextWrapped for Span<'a> {
    fn render_wrapped(&self, area: Rect, buf: &mut Buffer) {
        let mut text = Text::default();
        let style = self.style;

        text.push_line(self.to_line().style(style));

        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .to_owned()
            .render(area, buf);
    }
}

pub trait WidgetHeight {
    fn height_used(&self, area: Rect) -> u16;
}

pub trait CustomRender<Args = ()> {
    fn render(&self, area: Rect, buf: &mut Buffer, args: Args) -> Rect;
}

impl<const N: usize> CustomRender for [&str; N] {
    fn render(&self, area: Rect, buf: &mut Buffer, _: ()) -> Rect
    where
        Self: Sized,
    {
        // TODO implement wrapping so that insufficient width does not overflow text
        let mut area = area;
        for line in self {
            let line_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 1,
            };
            line.render_ref(line_area, buf);
            area.y += 1;
        }
        area.height = N as u16;
        area
    }
}

impl<const N: usize> CustomRender<bool> for [String; N] {
    fn render(&self, full_area: Rect, buf: &mut Buffer, leave_space: bool) -> Rect
    where
        Self: Sized,
    {
        let mut area = full_area;
        for line in self {
            let segs = split_string(line, area.width as usize);
            for seg in segs {
                if let Some(new_area) = area.height_consumed(1) {
                    seg.render_ref(area, buf);
                    area = new_area;
                } else {
                    break;
                }
            }

            if leave_space {
                if let Some(new_area) = area.height_consumed(1) {
                    area = new_area;
                } else {
                    break;
                }
            }
        }
        full_area.change_height(full_area.height - area.height)
    }
}

pub trait RectExt {
    fn width_consumed(self, width: u16) -> Option<Rect>;

    fn height_consumed(self, height: u16) -> Option<Rect>;

    fn consume_width(&mut self, width: u16);

    fn consume_height(&mut self, height: u16);

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
    fn width_consumed(self, width: u16) -> Option<Rect> {
        if self.width >= width {
            Some(Rect {
                x: self.x + width,
                y: self.y,
                width: self.width - width,
                height: self.height,
            })
        } else {
            None
        }
    }

    fn height_consumed(self, height: u16) -> Option<Rect> {
        if self.height >= height {
            Some(Rect {
                x: self.x,
                y: self.y + height,
                width: self.width,
                height: self.height - height,
            })
        } else {
            None
        }
    }

    #[track_caller]
    fn consume_width(&mut self, width: u16) {
        *self = self
            .width_consumed(width)
            .expect("consume_width failed, if your terminal width is too small, try increasing it otherwise this is a bug");
    }

    #[track_caller]
    fn consume_height(&mut self, height: u16) {
        *self = self
            .height_consumed(height)
            .expect("consume_height failed, if your terminal height is too small, try increasing it otherwise this is a bug");
    }

    fn change_height(self, new_height: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: new_height,
        }
    }
    fn change_width(self, new_width: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: new_width,
            height: self.height,
        }
    }

    fn margin_h(self, x: u16) -> Rect {
        Rect {
            x: self.x + x,
            y: self.y,
            width: self.width - 2 * x,
            height: self.height,
        }
    }

    fn margin_top(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y + m,
            width: self.width,
            height: self.height - m,
        }
    }

    fn margin_down(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height - m,
        }
    }

    fn margin_left(self, m: u16) -> Rect {
        Rect {
            x: self.x + m,
            y: self.y,
            width: self.width - m,
            height: self.height,
        }
    }

    fn margin_right(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width - m,
            height: self.height,
        }
    }

    fn expand_vertical(self, m: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y - m,
            width: self.width,
            height: self.height + 2 * m,
        }
    }

    fn block_inner(self) -> Rect {
        Rect {
            x: self.x + 1,
            y: self.y + 1,
            width: self.width - 2,
            height: self.height - 2,
        }
    }

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
