use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    widgets::{Block, Widget},
};

use super::{popup::Popup, text_scroll::TextScroll};
use crate::{
    act::Act,
    extensions::{KeyEventExt, RectExt, ThemedWidget},
    thematize::Thematize,
};

pub struct Areas {
    pub title_area: Rect,
    pub body_area: Rect,
}

/// A popup that displays text content with scrolling capability.
/// The popup does not have an explicit "open" state, it is shown only when it contains text.
/// And if text is an empty string it renders nothing.
#[derive(Debug)]
pub struct TextPopup {
    title: &'static str,
    text_scroll: TextScroll,
}

impl TextPopup {
    // TODO break_words should be in builder pattern
    pub fn new(title: &'static str, break_words: bool) -> Self {
        let text = String::new();
        Self {
            title,
            text_scroll: TextScroll::new(text, break_words),
        }
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text_scroll.text = text;
        self
    }

    pub fn is_open(&self) -> bool {
        !self.text_scroll.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.text_scroll.text.clear();
    }

    pub fn set_text(&mut self, text: String) {
        self.text_scroll.text = text;
        // self.text_scroll.scroll_offset = 0;
    }

    pub fn handle_event<A>(&mut self, key_event: Option<&KeyEvent>, area: Rect, actions: &mut A)
    where
        A: Act,
    {
        if self.is_open() {
            actions.ignore_esc();
        }

        let text_area = Popup::inner_area(area).block_inner();
        self.text_scroll.handle_event(key_event, text_area);

        if key_event.is_pressed(KeyCode::Esc) || key_event.is_pressed(KeyCode::Enter) {
            self.clear();
        }
    }

    pub fn get_areas(&self, popup_area: Rect) -> Areas {
        let inner = Popup::inner_area(popup_area);
        let title_area = inner.change_height(2);
        let body_area = inner.margin_top(2);
        Areas {
            title_area,
            body_area,
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

        Popup.render(popup_area, buf, &theme);

        if theme.boxed() {
            Block::bordered()
                .style(theme.style())
                .border_type(theme.border_type())
                .title_bottom("press ESC or Enter to dismiss")
                .render(popup_area, buf);
        }

        let areas = self.get_areas(popup_area);

        self.title.render(areas.title_area, buf);

        self.text_scroll.render(areas.body_area, buf, &theme);
    }
}
