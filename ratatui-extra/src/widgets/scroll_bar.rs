use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect},
    text::Span,
    widgets::WidgetRef,
};

use crate::thematize::Thematize;

pub struct CustomScrollBar {
    // Cursor range must be 0 to total_items - 1
    pub cursor: usize,
    pub total_items: usize,
    pub paginate: bool,
}

impl CustomScrollBar {
    pub fn render(self, area: Rect, buf: &mut Buffer, theme: &impl Thematize) {
        if self.cursor >= self.total_items {
            panic!(
                "CustomScrollBar out-of-range error: cursor {} must be less than total_items {}",
                self.cursor, self.total_items
            );
        }

        let max_height = area.height as usize;
        let num_pages = if self.paginate {
            self.total_items.div_ceil(max_height)
        } else if self.total_items > max_height {
            std::cmp::min(self.total_items - max_height + 1, max_height)
        } else {
            self.total_items
        };
        let current_page = if self.paginate {
            self.cursor / max_height
        } else if self.total_items > max_height {
            let scaled_cursor = self.cursor * num_pages / self.total_items;
            if scaled_cursor > 0 {
                std::cmp::min(scaled_cursor, num_pages - 1)
            } else {
                0
            }
        } else {
            self.cursor
        };

        let top = (0..current_page)
            .map(|i| get_page_height(i, max_height, num_pages))
            .sum::<usize>();
        let middle = get_page_height(current_page, max_height, num_pages);
        let bottom = ((current_page + 1)..num_pages)
            .map(|i| get_page_height(i, max_height, num_pages))
            .sum::<usize>();
        assert_eq!(
            top + middle + bottom,
            max_height,
            "self.cursor = {}, self.total_items = {}, max_height = {}, current_page = {current_page}, num_pages = {num_pages}, top = {top}, middle = {middle}, bottom = {bottom}",
            self.cursor, self.total_items, max_height
        );

        let mut i = 0;
        for _ in 0..top {
            Span::raw("║")
                .style(theme.select_inactive())
                .render_ref(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
        for _ in 0..middle {
            Span::raw("█")
                .style(theme.select_inactive())
                .render_ref(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
        for _ in 0..bottom {
            Span::raw("║")
                .style(theme.select_inactive())
                .render_ref(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
    }
}

fn get_page_height(i: usize, max_height: usize, num_pages: usize) -> usize {
    let base = max_height.saturating_div(num_pages);
    if i < max_height % num_pages {
        base + 1
    } else {
        base
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_page_height() {
        for max_height in 10..100 {
            for num_pages in 1..15 {
                let mut sum = 0;
                for i in 0..num_pages {
                    sum += get_page_height(i, max_height, num_pages);
                }

                if sum != max_height {
                    println!("capacity: {max_height}, num_pages: {num_pages}");
                    for i in 0..num_pages {
                        let h = get_page_height(i, max_height, num_pages);
                        println!("page {i}: {h}");
                    }
                }

                assert_eq!(
                    sum, max_height,
                    "capacity: {max_height}, num_pages: {num_pages}"
                );
            }
        }
    }
}
