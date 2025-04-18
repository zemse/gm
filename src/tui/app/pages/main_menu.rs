use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    actions::Action,
    tui::{
        app::widgets::select::Select,
        events::Event,
        traits::{Component, HandleResult},
    },
};

use super::address_book::AddressBookPage;

pub struct MainMenuPage {
    cursor: usize,
    list: Vec<Action>,
}

impl Default for MainMenuPage {
    fn default() -> Self {
        Self {
            list: Action::get_menu(),
            cursor: 0,
        }
    }
}

impl Component for MainMenuPage {
    fn reload(&mut self) {
        let fresh = Self::default();
        self.list = fresh.list;
    }

    fn handle_event(&mut self, event: &Event) -> HandleResult {
        let cursor_max = self.list.len();

        let mut result = HandleResult::default();
        if let Event::Input(key_event) = event {
            if key_event.kind == KeyEventKind::Press {
                match key_event.code {
                    KeyCode::Up => {
                        self.cursor = (self.cursor + cursor_max - 1) % cursor_max;
                    }
                    KeyCode::Down => {
                        self.cursor = (self.cursor + 1) % cursor_max;
                    }
                    KeyCode::Enter => match &self.list[self.cursor] {
                        Action::AddressBook { .. } => {
                            result
                                .page_inserts
                                .push(super::Page::AddressBook(AddressBookPage::default()));
                        }
                        _ => unimplemented!(),
                    },
                    _ => {}
                }
            }
        };

        result
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        Select {
            list: &self.list,
            cursor: Some(&self.cursor),
        }
        .render(area, buf);

        area
    }
}
