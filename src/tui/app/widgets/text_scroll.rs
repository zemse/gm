pub struct TextScroll {
    text: String,
    scroll_offset: usize,
}

impl TextScroll {
    pub fn new(text: String) -> Self {
        Self {
            text,
            scroll_offset: 0,
        }
    }

    fn lines(&self, width: usize) -> Vec<&str> {
        self.text
            .lines()
            .flat_map(|line| split_str_by_width(line, width))
            .collect()
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.scroll_offset = 0; // Reset scroll offset when text changes
    }

    pub fn scroll_up(&mut self, lines: usize) {
        if self.scroll_offset >= lines {
            self.scroll_offset -= lines;
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_scroll = self.text.lines().count().saturating_sub(1);
        if self.scroll_offset + lines <= max_scroll {
            self.scroll_offset += lines;
        } else {
            self.scroll_offset = max_scroll;
        }
    }

    pub fn get_visible_text(&self, width: usize) -> String {
        let lines: Vec<&str> = self.text.lines().collect();
        let visible_lines: Vec<&str> = lines
            .iter()
            .skip(self.scroll_offset)
            .take(width)
            .map(|line| line.trim_end())
            .collect();
        visible_lines.join("\n")
    }
}

fn split_str_by_width(s: &str, width: usize) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut count = 0;

    for (i, _) in s.char_indices() {
        if count == width {
            result.push(&s[start..i]);
            start = i;
            count = 0;
        }
        count += 1;
    }

    if start < s.len() {
        result.push(&s[start..]);
    }

    result
}
