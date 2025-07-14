use ratatui::{
    layout::Rect,
    widgets::{Block, Widget},
};

use crate::{tui::traits::CustomRender, utils::text::split_string};

use super::traits::{BorderedWidget, RectUtil};

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
                if let Ok(new_area) = area.consume_height(1) {
                    seg.render(area, buf);
                    area = new_area;
                } else {
                    break;
                }
            }

            if leave_space {
                if let Ok(new_area) = area.consume_height(1) {
                    area = new_area;
                } else {
                    break;
                }
            }
        }
        full_area.change_height(full_area.height - area.height)
    }
}

impl RectUtil for Rect {
    fn consume_height(self, height: u16) -> crate::Result<Rect> {
        if self.height >= height {
            Ok(Rect {
                x: self.x,
                y: self.y + height,
                width: self.width,
                height: self.height - height,
            })
        } else {
            Err(crate::Error::InternalErrorStr(
                "Cannot consume more height than available",
            ))
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
