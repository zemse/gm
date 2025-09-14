use gm_utils::text::split_string;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    widgets::{Block, Widget},
};

pub trait BorderedWidget {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized;
}

impl<T: Widget> BorderedWidget for T {
    fn render_with_block(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        block: Block<'_>,
    ) where
        Self: Sized,
    {
        let inner_area = block.inner(area);
        block.render(area, buf);
        self.render(inner_area, buf);
    }
}

pub trait WidgetHeight {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16;
}

pub trait CustomRender<Args = ()> {
    fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        args: Args,
    ) -> ratatui::prelude::Rect;
}

impl<const N: usize> CustomRender for [&str; N] {
    fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _: (),
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        // TODO implement wrapping so that insufficient width does not overflow text
        let mut area = area;
        for line in self {
            let line_area = ratatui::prelude::Rect {
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
    fn render(
        &self,
        full_area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        leave_space: bool,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        let mut area = full_area;
        for line in self {
            let segs = split_string(line, area.width as usize);
            for seg in segs {
                if let Some(new_area) = area.consume_height(1) {
                    seg.render(area, buf);
                    area = new_area;
                } else {
                    break;
                }
            }

            if leave_space {
                if let Some(new_area) = area.consume_height(1) {
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
    fn consume_height(self, height: u16) -> Option<ratatui::prelude::Rect>;

    fn change_height(self, height: u16) -> ratatui::prelude::Rect;

    fn margin_h(self, m: u16) -> ratatui::prelude::Rect;

    fn margin_top(self, m: u16) -> ratatui::prelude::Rect;

    fn margin_down(self, m: u16) -> ratatui::prelude::Rect;

    fn expand_vertical(self, m: u16) -> ratatui::prelude::Rect;

    fn block_inner(self) -> ratatui::prelude::Rect;
}

impl RectExt for Rect {
    fn consume_height(self, height: u16) -> Option<Rect> {
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
