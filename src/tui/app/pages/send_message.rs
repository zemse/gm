use std::sync::mpsc;
use std::sync::{atomic::AtomicBool, Arc};

use ratatui::widgets::Widget;

use crate::tui::app::SharedState;
use crate::tui::{
    app::widgets::form::{Form, FormItem}, // <- Using your custom form system
    events::Event,
    traits::{Component, HandleResult},
};
use crate::Result;

pub struct SendMessagePage {
    pub form: Form,
}

impl Default for SendMessagePage {
    fn default() -> Self {
        Self {
            form: Form {
                cursor: 1,
                items: vec![
                    FormItem::Heading("Send a Message"),
                    FormItem::InputBox {
                        label: "To",
                        text: String::new(),
                        empty_text: Some("<press SPACE to select from address book>"),
                    },
                    FormItem::InputBox {
                        label: "Message",
                        text: String::new(),
                        empty_text: None,
                    },
                    FormItem::Button {
                        label: "Send Message",
                    },
                ],
            },
        }
    }
}

impl Component for SendMessagePage {
    fn handle_event(
        &mut self,
        event: &Event,
        _tr: &mpsc::Sender<Event>,
        _sd: &Arc<AtomicBool>,
    ) -> Result<HandleResult> {
        self.form.handle_event(event, |_label, _form| {})?;

        Ok(HandleResult::default())
    }

    fn render_component(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        _: &SharedState,
    ) -> ratatui::prelude::Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);

        area
    }
}
