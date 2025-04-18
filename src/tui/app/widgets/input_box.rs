use std::cmp::min;

use ratatui::{
    layout::{Offset, Rect},
    text::Span,
    widgets::{Block, Widget},
};

use crate::tui::traits::WidgetHeight;

pub struct InputBox<'a> {
    pub focus: bool,
    pub label: &'a String,
    pub text: &'a String,
}

impl Widget for InputBox<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let lines = split_string(self.text, (area.width - 2) as usize);
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: (2 + lines.len()) as u16,
        };

        let block = Block::bordered().title(self.label.clone());
        let inner_area = block.inner(area_used);
        block.render(area_used, buf);

        if self.focus {
            Span::from("|").render(
                Rect {
                    x: inner_area.x + lines.last().unwrap().len() as u16,
                    y: inner_area.y + lines.len() as u16 - 1,
                    width: 1,
                    height: 1,
                },
                buf,
            );
        }

        for (idx, line) in lines.into_iter().enumerate() {
            line.render(
                inner_area.offset(Offset {
                    x: 0,
                    y: idx as i32,
                }),
                buf,
            );
        }
    }
}

impl WidgetHeight for InputBox<'_> {
    fn height_used(&self, area: ratatui::prelude::Rect) -> u16 {
        let lines = split_string(self.text, (area.width - 2) as usize);
        (2 + lines.len()) as u16
    }
}

fn split_string(s: &str, max_width: usize) -> Vec<&str> {
    let mut lines = vec![];

    let mut ptr = 0;
    let s_len = s.len();
    while ptr < s_len {
        let next = min(ptr + max_width, s_len);
        let s = s.get(ptr..next).expect("couldnt slice"); // can't go wrong
        lines.push(s);
        ptr = next;
    }

    if lines.is_empty() {
        lines.push("");
    }

    lines
}

#[cfg(test)]
mod test {
    use crate::tui::app::widgets::input_box::split_string;

    #[test]
    fn test_split_string() {
        assert_eq!(split_string("hello what is up", 3).len(), 3);
    }
}
