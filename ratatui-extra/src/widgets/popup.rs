use crate::{
    act::Act,
    extensions::{EventExt, RectExt, RenderTextWrapped, ThemedWidget},
    thematize::Thematize,
};
use gm_utils::text_wrap::text_wrap;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::{Margin, Rect},
    text::Span,
    widgets::{Block, Clear, Widget},
};

/// A trait for widgets that contain a Popup, it implements the basic methods so that
/// the inner popup can be controlled.
pub trait PopupWidget {
    fn get_popup(&self) -> &Popup;
    fn get_popup_mut(&mut self) -> &mut Popup;

    /*
     * Builder methods
     */

    fn with_title(mut self, title: &'static str) -> Self
    where
        Self: Sized,
    {
        self.get_popup_mut().title = Some(title);
        self
    }

    fn with_open(mut self, open: bool) -> Self
    where
        Self: Sized,
    {
        self.get_popup_mut().open = open;
        self
    }

    /*
     * Utility Methods
     */

    fn is_open(&self) -> bool {
        let popup = self.get_popup();
        popup.open
    }

    fn open(&mut self) {
        let popup = self.get_popup_mut();
        popup.open = true;
    }

    fn close(&mut self) {
        let popup = self.get_popup_mut();
        popup.open = false;
    }

    fn body_area(&self, popup_area: Rect) -> Rect {
        let popup = self.get_popup();
        let areas = popup.get_areas(popup_area);
        areas.body_area
    }
}

pub struct Areas {
    pub title_area: Rect,
    pub body_area: Rect,
}

#[derive(Debug, Default)]
pub struct Popup {
    title: Option<&'static str>,
    open: bool,
}

impl PopupWidget for Popup {
    fn get_popup(&self) -> &Popup {
        self
    }

    fn get_popup_mut(&mut self) -> &mut Popup {
        self
    }
}

impl Popup {
    pub fn handle_event<A: Act>(&mut self, input_event: Option<&Event>, actions: &mut A) {
        if self.open {
            actions.ignore_esc();

            if input_event.is_some_and(|input_event| input_event.is_key_pressed(KeyCode::Esc)) {
                self.close();
            }
        }
    }

    pub fn get_areas(&self, popup_area: Rect) -> Areas {
        let inner_area = popup_area.inner(Margin::new(2, 1));

        if let Some(title) = self.title {
            let title_height = text_wrap(title, inner_area.width).len() as u16;

            let title_area = inner_area.change_height(title_height + 1);
            let body_area = inner_area.margin_top(title_height + 1);
            Areas {
                title_area,
                body_area,
            }
        } else {
            Areas {
                title_area: Rect::default(),
                body_area: inner_area,
            }
        }
    }
}

impl ThemedWidget for Popup {
    fn render(&self, popup_area: Rect, buf: &mut Buffer, theme: &impl Thematize) {
        Clear.render(popup_area, buf);

        let theme = theme.popup();
        if theme.boxed() {
            Block::bordered()
                .style(theme.style())
                .border_type(theme.border_type())
                .render(popup_area, buf);
        } else {
            Block::default()
                .style(theme.style())
                .render(popup_area, buf);
        }

        let areas = self.get_areas(popup_area);
        if let Some(title) = self.title {
            Span::raw(title)
                .style(theme.style())
                .render_wrapped(areas.title_area, buf);
        }
    }
}
