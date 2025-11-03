use crate::extensions::BorderedWidget;
use crate::extensions::KeyEventExt;
use crate::extensions::MouseEventExt;
use crate::extensions::RectExt;
use crate::thematize::Thematize;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::MouseButton;
use ratatui::crossterm::event::MouseEventKind;
use ratatui::widgets::WidgetRef;
use ratatui::{crossterm::event::Event, layout::Rect, text::Line, widgets::Block};

pub enum ButtonResult {
    Pressed,
    HoverIn(bool),
}

#[derive(Debug, Default)]
pub struct Button {
    pub hover_focus: bool,
    pub is_success: bool,
    pub label: &'static str,
}

impl Button {
    pub fn new(label: &'static str) -> Self {
        Self {
            hover_focus: false,
            is_success: true,
            label,
        }
    }

    pub fn with_success_kind(mut self, is_success: bool) -> Self {
        self.is_success = is_success;
        self
    }

    pub fn handle_event(
        &mut self,
        event: Option<&Event>,
        area: Rect,
        focus: bool,
    ) -> Option<ButtonResult> {
        let mut result = None;

        if let Some(event) = event {
            match event {
                Event::Key(key_event) => {
                    if focus && key_event.is_pressed(KeyCode::Enter) {
                        result = Some(ButtonResult::Pressed);
                    }
                }
                Event::Mouse(mouse_event) => {
                    let button_area = self.area(area);

                    match mouse_event.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            if button_area.contains(mouse_event.position()) {
                                result = Some(ButtonResult::Pressed);
                            }
                        }
                        MouseEventKind::Moved => {
                            let new_focus = button_area.contains(mouse_event.position());
                            self.hover_focus = new_focus;
                            result = Some(ButtonResult::HoverIn(new_focus));
                        }
                        _ => {}
                    }
                }
                _ => {}
            };
        }

        result
    }

    pub fn area(&self, area: Rect) -> Rect {
        Rect {
            width: (self.label.len() + 2) as u16,
            height: 3,
            x: area.x,
            y: area.y,
        }
    }

    pub fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        focus: bool,
        theme: &impl Thematize,
    ) where
        Self: Sized,
    {
        let button_area = self.area(area);

        let focus = focus || self.hover_focus;

        if theme.boxed() {
            Line::from(self.label).render_with_block(
                button_area,
                buf,
                Block::bordered()
                    .border_type(theme.border_type())
                    .style(if focus {
                        theme.button_focused()
                    } else {
                        theme.style_dim()
                    }),
                false,
                (),
            );
        } else {
            Block::default()
                .style(if focus {
                    if self.is_success {
                        theme.cursor()
                    } else {
                        theme.cursor_cancelled()
                    }
                } else {
                    theme.button_notfocused()
                })
                .render_ref(button_area, buf);

            Line::from(self.label).render_ref(button_area.block_inner(), buf);
        }
    }
}
