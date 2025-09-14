use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Clone, Debug, Default)]
pub struct Cursor {
    pub current: usize,
}

impl Cursor {
    pub fn reset(&mut self) {
        self.current = 0;
    }

    pub fn handle(&mut self, key_event: Option<&KeyEvent>, cursor_max: usize) {
        if let Some(key_event) = key_event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => {
                        self.move_up(cursor_max);
                    }
                    KeyCode::Down => {
                        self.move_down(cursor_max);
                    }
                    _ => {}
                }
            }
        }
    }

    fn move_up(&mut self, max: usize) {
        if max != 0 {
            self.current = (self.current + max - 1) % max;
        }
    }

    fn move_down(&mut self, max: usize) {
        if max != 0 {
            self.current = (self.current + 1) % max;
        }
    }
}
