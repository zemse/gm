use gm_utils::text::split_string;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    widgets::{Block, Widget},
};

pub trait BorderedWidget {
    fn render_with_block(
        self,
        area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        leave_horizontal_space: bool,
    ) where
        Self: Sized;
}

impl<T: Widget> BorderedWidget for T {
    fn render_with_block(
        self,
        area: Rect,
        buf: &mut Buffer,
        block: Block<'_>,
        leave_horizontal_space: bool,
    ) where
        Self: Sized,
    {
        let inner_area = block
            .inner(area)
            .margin_h(if leave_horizontal_space { 1 } else { 0 });
        block.render(area, buf);
        self.render(inner_area, buf);
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
            line.render(line_area, buf);
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
                    seg.render(area, buf);
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

    fn change_height(self, height: u16) -> Rect;

    fn margin_h(self, m: u16) -> Rect;

    fn margin_top(self, m: u16) -> Rect;

    fn margin_down(self, m: u16) -> Rect;

    fn margin_left(self, m: u16) -> Rect;

    fn margin_right(self, m: u16) -> Rect;

    fn expand_vertical(self, m: u16) -> Rect;

    fn block_inner(self) -> Rect;
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

    fn consume_width(&mut self, width: u16) {
        *self = self
            .width_consumed(width)
            .expect("consume_width failed, if your terminal width is too small, try increasing it otherwise this is a bug");
    }

    fn consume_height(&mut self, height: u16) {
        *self = self
            .height_consumed(height)
            .expect("consume_height failed, if your terminal height is too small, try increasing it otherwise this is a bug");
    }

    fn change_height(self, height: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height,
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
}

pub trait KeyEventExt {
    fn is_pressed(&self, key: KeyCode) -> bool;
}

impl KeyEventExt for Option<&KeyEvent> {
    fn is_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self,
            Some(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                modifiers: KeyModifiers::NONE,
                ..
            }) if *code == key
        )
    }
}
