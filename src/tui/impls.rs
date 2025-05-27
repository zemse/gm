use ratatui::{
    layout::Rect,
    widgets::{Block, Widget},
};

use crate::utils::text::split_string;

use super::traits::{BorderedWidget, CustomRender, RectUtil};

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
                seg.render(area, buf);
                area = area.consume_height(1);
            }

            area = area.consume_height(if leave_space { 2 } else { 1 });
        }
        full_area.change_height(full_area.height - area.height)
    }
}

impl RectUtil for Rect {
    fn consume_height(self, height: u16) -> Rect {
        Rect {
            x: self.x,
            y: self.y + height,
            width: self.width,
            height: self.height - height,
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
}
