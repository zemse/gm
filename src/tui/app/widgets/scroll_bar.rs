use ratatui::{layout::Offset, widgets::Widget};

pub struct CustomScrollBar {
    pub cursor: usize,
    pub capacity: usize,
    pub max: usize,
}

impl Widget for CustomScrollBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let num_pages = self.max.div_ceil(self.capacity);
        let current_page = self.cursor / self.capacity;

        let height = area.height as usize;

        let get_page_height = |i| {
            let base = height.div_ceil(num_pages);
            if height % num_pages <= i {
                base
            } else {
                base - 1
            }
        };

        let top = (0..current_page).map(get_page_height).sum::<usize>();
        let middle = get_page_height(current_page);
        let bottom = ((current_page + 1)..num_pages)
            .map(get_page_height)
            .sum::<usize>();
        assert_eq!(
            top + middle + bottom,
            height,
            "current_page = {current_page}, num_pages = {num_pages}, top = {top}, middle = {middle}, bottom = {bottom}, height = {height}"
        );

        let mut i = 0;
        for _ in 0..top {
            "║".render(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
        for _ in 0..middle {
            "█".render(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
        for _ in 0..bottom {
            "║".render(area.offset(Offset { x: 0, y: i }), buf);
            i += 1;
        }
    }
}

// num_pages 3
// height 10
// first 4
// second 3
// third 3

// num_pages 3
// height 11
// first 4
// second 4
// third 3

// num_pages 3
// height 20
// first 7
// second 7
// third 6

// num_pages 3
// height 21
// first 7
// second 7
// third 7
