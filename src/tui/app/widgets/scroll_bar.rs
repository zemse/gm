use ratatui::{layout::Offset, widgets::Widget};

pub struct CustomScrollBar {
    pub cursor: usize,
    pub total: usize,
    pub paginate: bool,
}

impl Widget for CustomScrollBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let capacity = area.height as usize;
        let num_pages = if self.paginate {self.total.div_ceil(capacity)} else {self.total};
        let current_page = if self.paginate {self.cursor / capacity} else {self.cursor};

        let top = (0..current_page)
            .map(|i| get_page_height(i, capacity, num_pages))
            .sum::<usize>();
        let middle = get_page_height(current_page, capacity, num_pages);
        let bottom = ((current_page + 1)..num_pages)
            .map(|i| get_page_height(i, capacity, num_pages))
            .sum::<usize>();
        assert_eq!(
            top + middle + bottom,
            capacity,
            "self.total = {}, capacity = {}, current_page = {current_page}, num_pages = {num_pages}, top = {top}, middle = {middle}, bottom = {bottom}",
            self.total, capacity
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

fn get_page_height(i: usize, capacity: usize, num_pages: usize) -> usize {
    let base = capacity / num_pages;
    if i < capacity % num_pages {
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
        for capacity in 10..100 {
            for num_pages in 1..15 {
                let mut sum = 0;
                for i in 0..num_pages {
                    sum += get_page_height(i, capacity, num_pages);
                }

                if sum != capacity {
                    println!("capacity: {capacity}, num_pages: {num_pages}");
                    for i in 0..num_pages {
                        let h = get_page_height(i, capacity, num_pages);
                        println!("page {i}: {h}");
                    }
                }

                assert_eq!(
                    sum, capacity,
                    "capacity: {capacity}, num_pages: {num_pages}"
                );
            }
        }
    }
}
