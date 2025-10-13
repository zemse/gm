use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
    text::Span,
    widgets::{Block, Widget},
};

use crate::{
    act::Act,
    extensions::{MouseEventExt, RectExt, WidgetHeight},
    thematize::Thematize,
};

#[derive(Debug)]
pub struct BooleanInput {
    pub label: &'static str,
    pub value: bool,
}

impl BooleanInput {
    pub fn new(label: &'static str, value: bool) -> Self {
        Self { label, value }
    }

    pub fn toggle(&mut self) {
        self.value = !self.value;
    }

    pub fn handle_event<A: Act>(
        &mut self,
        input_event: Option<&Event>,
        area: Rect,
        actions: &mut A,
    ) {
        if let Some(input_event) = input_event {
            match input_event {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Left => {
                            if self.value {
                                self.value = false;

                                // Signal that we are consuming left key
                                actions.ignore_left();
                            }
                        }
                        KeyCode::Right => {
                            if !self.value {
                                self.value = true;

                                // Signal that we are consuming left key
                                actions.ignore_right();
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    let switch_area = area.block_inner().change_width(3);
                    if switch_area.contains(mouse_event.position()) && mouse_event.is_left_click() {
                        self.toggle();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, focus: bool, theme: &impl Thematize) {
        let area_used = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 3,
        };

        let style = if focus {
            theme.style()
        } else {
            theme.style_dim()
        };

        let inner_area = if theme.boxed() {
            let block = Block::bordered()
                .border_type(theme.border_type())
                .style(style)
                .title(self.label);
            let inner_area = block.inner(area_used);
            block.render(area_used, buf);
            inner_area.margin_h(1)
        } else {
            Span::raw(self.label).style(style).render(area_used, buf);
            area_used.block_inner().margin_h(1)
        };

        if focus {
            let mut draw_area = inner_area;

            if self.value {
                let part_1 = "Off ──◉";
                Span::raw(part_1)
                    .style(theme.style())
                    .render(draw_area, buf);
                draw_area.consume_width(8);

                Span::raw("On").style(theme.cursor()).render(draw_area, buf);
            } else {
                let part_1 = "Off";
                Span::raw(part_1)
                    .style(theme.cursor_cancelled())
                    .render(inner_area, buf);
                draw_area.consume_width(4);

                Span::raw("◉── On")
                    .style(theme.style())
                    .render(draw_area, buf);
            }
        } else {
            let text = if self.value {
                "Off ──◯ On"
            } else {
                "Off ◯── On"
            };

            Span::raw(text).style(style).render(inner_area, buf)
        }
    }
}

impl WidgetHeight for BooleanInput {
    fn height_used(&self, area: Rect) -> u16 {
        3.min(area.height)
    }
}
