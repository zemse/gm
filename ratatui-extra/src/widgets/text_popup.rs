use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use super::{popup::Popup, text_scroll::TextScroll};
use crate::{
    act::Act,
    extensions::{KeyEventExt, ThemedWidget},
    popup::PopupWidget,
    thematize::Thematize,
};

/// A popup that displays text content with scrolling capability.
/// It is shown only when it contains text. And if text is updated to
/// empty it is closed.
#[derive(Debug, Default)]
pub struct TextPopup {
    popup: Popup,
    text_scroll: TextScroll,
}

impl PopupWidget for TextPopup {
    fn get_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }
}

impl TextPopup {
    pub fn with_break_words(mut self, break_words: bool) -> Self {
        self.text_scroll.break_words = break_words;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.set_text(text);
        self
    }

    pub fn set_text(&mut self, text: String) {
        if text.is_empty() {
            self.popup.close();
        } else {
            self.popup.open();
        }

        self.text_scroll.text = text;
    }

    pub fn handle_event<A>(
        &mut self,
        key_event: Option<&KeyEvent>,
        popup_area: Rect,
        actions: &mut A,
    ) where
        A: Act,
    {
        if self.is_open() {
            actions.ignore_esc();
        }

        let text_area = self.body_area(popup_area);
        self.text_scroll.handle_event(key_event, text_area);

        if key_event.is_pressed(KeyCode::Esc) || key_event.is_pressed(KeyCode::Enter) {
            self.close();
        }
    }
}

impl ThemedWidget for TextPopup {
    fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.text_scroll.text.is_empty() {
            return;
        }
        let theme = theme.popup();

        self.popup.render(popup_area, buf, &theme);

        self.text_scroll
            .render(self.body_area(popup_area), buf, &theme);
    }
}
