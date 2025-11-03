use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Constraint, Layout, Rect},
};

use super::{popup::Popup, text_interactive::TextInteractive};
use crate::{
    act::Act,
    extensions::{EventExt, RectExt, ThemedWidget},
    popup::PopupWidget,
    thematize::Thematize,
};

pub enum TextPopupEvent {
    Closed,
}

struct Areas {
    text_area: Rect,
    note_area: Rect,
}

/// A popup that displays text content with scrolling capability.
/// It is shown only when it contains text. And if text is updated to
/// empty it is closed.
#[derive(Debug, Default)]
pub struct TextPopup {
    pub(crate) popup: Popup,
    pub(crate) text: TextInteractive,
    pub(crate) note: Option<TextInteractive>,
}

impl PopupWidget for TextPopup {
    fn get_base_popup(&self) -> &Popup {
        &self.popup
    }

    fn get_base_popup_mut(&mut self) -> &mut Popup {
        &mut self.popup
    }

    fn get_popup_inner(&self) -> &dyn PopupWidget {
        &self.popup
    }

    fn get_popup_inner_mut(&mut self) -> &mut dyn PopupWidget {
        &mut self.popup
    }
}

impl TextPopup {
    pub fn with_text(mut self, text: String) -> Self {
        self.set_text(text, false);
        self
    }

    pub fn with_note(mut self, note: &'static str) -> Self {
        self.note = Some(TextInteractive::default().with_text(note.to_string()));
        self
    }

    pub fn text(&self) -> &str {
        self.text.text()
    }

    pub fn set_text(&mut self, text: String, scroll_to_top: bool) {
        if text.is_empty() {
            self.popup.close();
        } else {
            self.popup.open();
        }

        self.text.set_text(text, scroll_to_top);
    }

    pub fn handle_event<A>(
        &mut self,
        event: Option<&Event>,
        popup_area: Rect,
        actions: &mut A,
    ) -> Option<TextPopupEvent>
    where
        A: Act,
    {
        let mut text_popup_event = None;

        if let Some(event) = event {
            let Areas {
                text_area,
                note_area,
            } = self.get_areas(popup_area);

            self.text.handle_event(Some(event), text_area, actions);

            if let Some(note) = &mut self.note {
                note.handle_event(Some(event), note_area, actions);
            }

            if !actions.is_esc_ignored()
                && (event.is_key_pressed(KeyCode::Esc) || event.is_key_pressed(KeyCode::Enter))
            {
                self.close();
                text_popup_event = Some(TextPopupEvent::Closed);
            }
        }

        if self.is_open() {
            actions.ignore_esc();
        }

        text_popup_event
    }

    fn get_areas(&self, popup_area: Rect) -> Areas {
        let body_area = self.body_area(popup_area);

        if let Some(note) = &self.note {
            let note_lines_count = note.lines_count(body_area.width as usize) as u16;
            let [text_area, note_area] =
                Layout::vertical([Constraint::Min(1), Constraint::Length(note_lines_count + 1)])
                    .areas(body_area);

            Areas {
                text_area,
                note_area: note_area.margin_top(1),
            }
        } else {
            Areas {
                text_area: body_area,
                note_area: Rect::default(),
            }
        }
    }
}

impl ThemedWidget for TextPopup {
    fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            self.popup.render(popup_area, buf, &theme);

            let Areas {
                text_area,
                note_area,
            } = self.get_areas(popup_area);

            self.text.render(text_area, buf, &theme);

            if let Some(note) = &self.note {
                note.render(note_area, buf, &theme);
            }
        }
    }
}
