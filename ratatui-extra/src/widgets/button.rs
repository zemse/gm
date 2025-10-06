use crate::extensions::BorderedWidget;
use crate::extensions::KeyEventExt;
use crate::extensions::MouseEventExt;
use crate::extensions::RectExt;
use crate::thematize::Thematize;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::MouseButton;
use ratatui::crossterm::event::MouseEventKind;
use ratatui::widgets::WidgetRef;
use ratatui::{crossterm::event::Event, layout::Rect, style::Style, text::Line, widgets::Block};

#[derive(Debug)]
pub struct Button {
    pub focus: bool,
    pub label: &'static str,
}

impl Button {
    pub fn handle_event<E, F>(
        event: &Event,
        area: Rect,
        label: &'static str,
        on_press: F,
    ) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        match event {
            Event::Key(key_event) => {
                if key_event.is_pressed(KeyCode::Enter) {
                    on_press()?;
                }
            }
            Event::Mouse(mouse_event) => {
                let button_area = Self::area(label, area);

                if button_area.contains(mouse_event.position()) {
                    if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
                        on_press()?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    pub fn area(label: &'static str, area: Rect) -> Rect {
        Rect {
            width: (label.len() + 2) as u16,
            height: 3,
            x: area.x,
            y: area.y,
        }
    }

    pub fn render(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        theme: &impl Thematize,
    ) where
        Self: Sized,
    {
        let button_area = Self::area(self.label, area);

        if theme.boxed() {
            Line::from(self.label).render_with_block(
                button_area,
                buf,
                Block::bordered()
                    .border_type(theme.border_type())
                    .style(if self.focus {
                        theme.button_focused()
                    } else {
                        Style::default()
                    }),
                false,
                (),
            );
        } else {
            Block::default()
                .style(if self.focus {
                    theme.button_focused()
                } else {
                    theme.button_notfocused()
                })
                .render_ref(button_area, buf);

            Line::from(self.label).render_ref(button_area.block_inner(), buf);
        }
    }
}
