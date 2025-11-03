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
    /// Every popup widget apart from the Popup widget itself must be building on top of
    /// another popup widget. This method allows accessing the inner popup widget.
    ///
    /// Returns a reference to the inner popup widget.
    fn get_popup_inner(&self) -> &dyn PopupWidget;

    /// Mutable version of `get_popup_inner`
    fn get_popup_inner_mut(&mut self) -> &mut dyn PopupWidget;

    /*
     * Internal methods used to access the base popup
     */
    /// This is automatically implemented in the Popups that implement this trait. It
    /// creates a chain of calls to reach the base popup getting it's reference.
    fn get_base_popup(&self) -> &Popup {
        self.get_popup_inner().get_base_popup()
    }

    /// Mutable version of `get_base_popup`
    fn get_base_popup_mut(&mut self) -> &mut Popup {
        self.get_popup_inner_mut().get_base_popup_mut()
    }

    /*
     * Builder methods
     */
    /// Sets the title of the popup
    fn with_title(mut self, title: &'static str) -> Self
    where
        Self: Sized,
    {
        self.get_base_popup_mut().title = Some(title);
        self
    }

    /// Sets whether the popup is open or closed at init
    fn with_open(mut self, open: bool) -> Self
    where
        Self: Sized,
    {
        self.get_base_popup_mut().open = open;
        self
    }

    /*
     * Utility Methods
     *
     * These methods chains the calls to reach the base popup, making sure any
     * overrides are triggered.
     */
    /// Checks whether the popup is open
    fn is_open(&self) -> bool {
        self.get_popup_inner().is_open()
    }

    /// Opens the popup
    fn open(&mut self) {
        self.get_popup_inner_mut().open();
    }

    /// Closes the popup
    fn close(&mut self) {
        self.get_popup_inner_mut().close();
    }

    /// Gets the body area of the popup
    fn body_area(&self, popup_area: Rect) -> Rect {
        self.get_popup_inner().body_area(popup_area)
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
    fn get_popup_inner(&self) -> &dyn PopupWidget {
        self
    }

    fn get_popup_inner_mut(&mut self) -> &mut dyn PopupWidget {
        self
    }

    fn get_base_popup(&self) -> &Popup {
        self
    }

    fn get_base_popup_mut(&mut self) -> &mut Popup {
        self
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn open(&mut self) {
        self.open = true;
    }

    fn close(&mut self) {
        self.open = false;
    }

    fn body_area(&self, popup_area: Rect) -> Rect {
        let popup = self.get_base_popup();
        let areas = popup.get_areas(popup_area);
        areas.body_area
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
