use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{
            widgets::form::{Form, FormItem},
            SharedState,
        },
        events::Event,
        traits::{Component, HandleResult},
    },
};

pub struct ConfigPage {
    pub form: Form,
}

impl Default for ConfigPage {
    fn default() -> Self {
        let mut config = Config::load();
        if config.alchemy_api_key.is_none() {
            config.alchemy_api_key = Some("".to_string());
        }
        Self {
            form: Form {
                cursor: 0,
                items: vec![
                    FormItem::Heading("Configuration"),
                    FormItem::InputBox {
                        label: "Alchemy API key",
                        text: config.alchemy_api_key.unwrap_or_default(),
                        empty_text: None,
                    },
                    FormItem::BooleanInput {
                        label: "Testnet Mode",
                        value: config.testnet_mode,
                    },
                    FormItem::DisplayText(String::new()),
                ],
            },
        }
    }
}
impl Component for ConfigPage {
    fn handle_event(
        &mut self,
        event: &Event,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        // InputBox::handle_events(self.text_input_mut(), event)?;

        let mut handle_result = HandleResult::default();

        self.form.handle_event(event, |label, form| {
            if label == "Alchemy API key" {
                handle_result.reload = true;

                let mut config = Config::load();
                config.alchemy_api_key = Some(form.get_input_text(1).clone());
                config.testnet_mode = form.get_boolean_value(2);
                config.save();
            }
        })?;

        let display_text = self.form.get_display_text_mut(3);
        *display_text = "Configuration saved".to_string();

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);
        // self.display.render(area, buf);
        area
    }
}
