use std::{fmt::Display, sync::Arc};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
    text::Span,
    widgets::{Block, Widget},
};

use super::popup::Popup;
use crate::{
    act::Act,
    extensions::{EventExt, RectExt},
    filter_select::FilterSelect,
    select_owned::SelectEvent,
    thematize::Thematize,
};

#[derive(Debug)]
pub struct FilterSelectPopup<Item: Display + PartialEq> {
    title: &'static str,
    open: bool,
    filter_select: FilterSelect<Item>,
}

impl<Item: Display + PartialEq> FilterSelectPopup<Item> {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            open: false,
            filter_select: FilterSelect::default(),
        }
    }

    pub fn with_empty_text(mut self, empty_text: &'static str) -> Self {
        self.filter_select = self.filter_select.with_empty_text(empty_text);
        self
    }

    pub fn set_items(&mut self, items: Option<Vec<Item>>) {
        self.filter_select.set_items(items);
    }

    pub fn display_selection(&self) -> String {
        self.filter_select
            .get_focussed_item()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_focused_item(&mut self, item: Item) {
        self.filter_select.select.set_focussed_item(Arc::new(item));
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.open = true;
        self.filter_select.reset();
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn handle_event<'a, A>(
        &'a mut self,
        input_event: Option<&Event>,
        popup_area: Rect,
        actions: &mut A,
    ) -> Option<&'a Arc<Item>>
    where
        A: Act,
    {
        let mut result = None;

        if self.open {
            // TODO handle using popup widget
            if input_event.is_some_and(|input_event| input_event.is_key_pressed(KeyCode::Esc)) {
                self.close();
            }

            if let Some(SelectEvent::Select(item)) =
                self.filter_select.handle_event(input_event, popup_area)
            {
                result = Some(item);
            }

            // if self.items.is_some() {

            //     if let Some(key_event) = key_event {
            //         if key_event.kind == KeyEventKind::Press {
            //             match key_event.code {
            //                 KeyCode::Char(char) => {
            //                     self.search_string.push(char);
            //                 }
            //                 KeyCode::Backspace => {
            //                     self.search_string.pop();
            //                 }
            //                 KeyCode::Enter => {
            //                     self.close();
            //                     let items = self.items.as_ref().unwrap();
            //                     result = Some(&items[self.cursor.current]);
            //                 }
            //                 _ => {}
            //             }
            //         }
            //     }
            // }

            actions.ignore_esc();
        }

        result
    }
    pub fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(popup_area, buf, &theme);

            if theme.boxed() {
                let block = Block::bordered()
                    .border_type(theme.border_type())
                    .style(theme.style());
                block.render(popup_area, buf);
            }

            let mut inner_area = Popup::inner_area(popup_area);

            Span::raw(self.title).render(inner_area, buf);
            inner_area.consume_height(2);

            self.filter_select.render(inner_area, buf, &theme);
        }
    }
}
