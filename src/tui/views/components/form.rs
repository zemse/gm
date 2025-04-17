use ratatui::{style::Stylize, text::Line, widgets::Widget};

use crate::tui::traits::WidgetHeight;

use super::{button::Button, input_box::InputBox};

pub enum FormItem<'a> {
    Heading(&'a str),
    InputBox {
        focus: bool,
        label: &'a String,
        text: &'a String,
    },
    Button {
        focus: bool,
        label: &'a String,
    },
    Error {
        label: &'a Option<&'a String>,
    },
}

pub struct Form<'a> {
    pub items: Vec<FormItem<'a>>,
}

impl Widget for Form<'_> {
    fn render(self, mut area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        for item in self.items {
            match item {
                FormItem::Heading(heading) => {
                    Line::from(heading).bold().render(area, buf);
                    area.y += 2;
                }
                FormItem::InputBox { focus, label, text } => {
                    let widget = InputBox { focus, label, text };
                    let height_used = widget.height_used(area); // to see height based on width
                    widget.render(area, buf);
                    area.y += height_used;
                }
                FormItem::Button { focus, label } => {
                    Button { focus, label }.render(area, buf);
                    area.y += 3;
                }
                FormItem::Error { label } => {
                    if let Some(label) = label {
                        area.y += 1; // leave a line before the error text
                        label.render(area, buf);
                        area.y += 1;
                    }
                }
            }
        }
    }
}
