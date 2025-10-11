use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    widgets::Block,
};

use crate::{
    act::Act,
    extensions::{EventExt, RectExt, ThemedWidget},
    thematize::Thematize,
};

use super::{button::Button, popup::Popup, text_scroll::TextScroll};

struct Areas {
    body_area: Rect,
    confirm_button_area: Rect,
    cancel_button_area: Rect,
}

#[derive(Debug)]
pub struct ConfirmPopup {
    title: &'static str,
    text: TextScroll,
    open: bool,
    confirm_button: Button,
    cancel_button: Button,
    is_confirm_focused: bool,
}

impl ConfirmPopup {
    pub fn new(
        title: &'static str,
        text: String,
        confirm_button_label: &'static str,
        cancel_button_label: &'static str,
        initial_cursor_on_confirm: bool,
    ) -> Self {
        Self {
            title,
            text: TextScroll::new(text, true),
            confirm_button: Button::new(confirm_button_label).with_success_kind(true),
            cancel_button: Button::new(cancel_button_label).with_success_kind(true),
            open: false,
            is_confirm_focused: initial_cursor_on_confirm,
        }
    }

    pub fn open_already(mut self) -> Self {
        self.open();
        self
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    // Opens the popup with the fresh items.
    pub fn open(&mut self) {
        self.open = true;
        // self.is_confirm_focused = false;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text.text
    }

    pub fn handle_event<A, E, F1, F2>(
        &mut self,
        input_event: Option<&Event>,
        area: Rect,
        mut on_confirm: F1,
        mut on_cancel: F2,
    ) -> Result<A, E>
    where
        A: Act,
        F1: FnMut() -> Result<(), E>,
        F2: FnMut() -> Result<(), E>,
    {
        let mut act = A::default();

        if self.open {
            act.ignore_left();
            act.ignore_right();

            let areas = self.get_areas(area);

            if let Some(input_event) = input_event {
                self.text
                    .handle_event(input_event.key_event(), areas.body_area);

                // if self.is_confirm_focused {
                self.confirm_button.handle_event(
                    Some(input_event),
                    areas.confirm_button_area,
                    || {
                        on_confirm()?;
                        self.open = false;
                        Ok(())
                    },
                    |is_focused| {
                        if is_focused {
                            self.is_confirm_focused = true;
                        }
                        Ok(())
                    },
                )?;
                // } else {
                self.cancel_button.handle_event(
                    Some(input_event),
                    areas.cancel_button_area,
                    || {
                        on_cancel()?;
                        self.open = false;
                        Ok(())
                    },
                    |is_focused| {
                        if is_focused {
                            self.is_confirm_focused = false;
                        }
                        Ok(())
                    },
                )?;
                // }

                if let Event::Key(key_event) = input_event {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Left => {
                                self.is_confirm_focused = false;
                            }
                            KeyCode::Right => {
                                self.is_confirm_focused = true;
                            }
                            KeyCode::Esc => {
                                on_cancel()?;
                                self.close();
                            }
                            _ => {}
                        }
                    }
                }

                act.ignore_esc();
            }
        }

        // Ensure key events and mouse events cause proper effects
        // TODO this is like a duck tape, find a better way
        // if self.is_confirm_focused {
        //     self.confirm_button.set_focus(true);
        //     self.cancel_button.set_focus(false);
        // } else {
        //     self.confirm_button.set_focus(false);
        //     self.cancel_button.set_focus(true);
        // }

        Ok(act)
    }

    fn get_areas(&self, area: Rect) -> Areas {
        let inner_area = Popup::inner_area(area);
        let [body_area, button_area] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).areas(inner_area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(button_area);

        Areas {
            body_area,
            cancel_button_area: left_area.button_center(self.cancel_button.label.len()),
            confirm_button_area: right_area.button_center(self.confirm_button.label.len()),
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &impl Thematize)
    where
        Self: Sized,
    {
        if self.is_open() {
            let theme = theme.popup();

            Popup.render(area, buf, &theme);
            Block::bordered().title(self.title).inner(area);

            let areas = self.get_areas(area);

            self.text.render(areas.body_area, buf, &theme);

            self.cancel_button.render(
                areas.cancel_button_area,
                buf,
                !self.is_confirm_focused,
                &theme,
            );

            self.confirm_button.render(
                areas.confirm_button_area,
                buf,
                self.is_confirm_focused,
                &theme,
            );
        }
    }
}
