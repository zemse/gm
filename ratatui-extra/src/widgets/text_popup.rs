use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    widgets::Block,
};

use super::{popup::Popup, text_scroll::TextScroll};
use crate::{
    act::Act,
    extensions::{BorderedWidget, KeyEventExt, RectExt},
    thematize::Thematize,
};

pub struct TextPopup {
    title: &'static str,
    text_scroll: TextScroll,
}

impl TextPopup {
    pub fn new(title: &'static str, break_words: bool) -> Self {
        let text = String::new();
        Self {
            title,
            text_scroll: TextScroll::new(text, break_words),
        }
    }

    pub fn is_shown(&self) -> bool {
        !self.text_scroll.text.is_empty()
    }

    pub fn clear(&mut self) {
        self.text_scroll.text.clear();
    }

    pub fn set_text(&mut self, text: String) {
        self.text_scroll.text = text;
        // self.text_scroll.scroll_offset = 0;
    }

    pub fn handle_event<A>(
        &mut self,
        key_event: Option<&KeyEvent>,
        area: ratatui::prelude::Rect,
    ) -> A
    where
        A: Act,
    {
        let mut act = A::default();

        if self.is_shown() {
            act.ignore_esc();
        }

        let text_area = Popup::inner_area(area).block_inner();
        self.text_scroll.handle_event(key_event, text_area);

        if key_event.is_pressed(KeyCode::Esc) {
            self.clear();
        }

        act
    }
    pub fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        theme: &impl Thematize,
    ) where
        Self: Sized,
    {
        if self.text_scroll.text.is_empty() {
            return;
        }
        let theme = theme.popup();

        Popup.render(area, buf, &theme);

        let popup_inner_area = Popup::inner_area(area);

        let block = Block::bordered()
            .style(theme.block())
            .border_type(theme.border_type())
            .title(self.title)
            .title_bottom("press ESC to dismiss");

        self.text_scroll
            .render_with_block(popup_inner_area, buf, block);
    }
}
