use std::{fmt::Display, sync::Arc};

use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect};

use super::popup::Popup;
use crate::{
    act::Act, extensions::ThemedWidget, filter_select::FilterSelect, popup::PopupWidget,
    select::SelectEvent, thematize::Thematize,
};

#[derive(Debug)]
pub struct FilterSelectPopup<Item: Display + PartialEq> {
    popup: Popup,
    filter_select: FilterSelect<Item>,
}

impl<Item: Display + PartialEq> Default for FilterSelectPopup<Item> {
    fn default() -> Self {
        Self {
            popup: Popup::default(),
            filter_select: FilterSelect::default().with_focus(true),
        }
    }
}

impl<Item: Display + PartialEq> PopupWidget for FilterSelectPopup<Item> {
    fn get_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }
}

impl<Item: Display + PartialEq> FilterSelectPopup<Item> {
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
        self.popup.is_open()
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.popup.open();
        self.filter_select.reset();
    }

    pub fn close(&mut self) {
        self.popup.close();
    }

    pub fn handle_event<'a, A>(
        &'a mut self,
        input_event: Option<&Event>,
        popup_area: Rect,
        actions: &mut A,
    ) -> crate::Result<Option<&'a Arc<Item>>>
    where
        A: Act,
    {
        let mut result = None;

        if self.is_open() {
            self.popup.handle_event(input_event, actions);

            if let Some(SelectEvent::Select(item)) =
                self.filter_select.handle_event(input_event, popup_area)?
            {
                result = Some(item);
            }

            actions.ignore_esc();
        }

        Ok(result)
    }
}

impl<Item: Display + PartialEq> ThemedWidget for FilterSelectPopup<Item> {
    fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            self.popup.render(popup_area, buf, theme);

            // let theme = theme.popup();
            self.filter_select
                .render(self.body_area(popup_area), buf, theme);
        }
    }
}
