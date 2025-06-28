use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, text::Line, widgets::Widget};
use strum::EnumIter;

use super::{account::AccountPage, Page};
use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{
            widgets::form::{Form, FormItemIndex, FormWidget},
            SharedState,
        },
        events::Event,
        traits::{Component, HandleResult, RectUtil},
    },
};

#[derive(PartialEq, EnumIter)]
pub enum FormItem {
    CreateOrImportWallet,
    AlchemyApiKey,
    Save,
    Display,
}

impl FormItemIndex for FormItem {
    fn index(self) -> usize {
        self as usize
    }
}

impl TryFrom<FormItem> for FormWidget {
    type Error = crate::Error;
    fn try_from(value: FormItem) -> crate::Result<Self> {
        let widget = match value {
            FormItem::CreateOrImportWallet => FormWidget::Button {
                label: "Create or Import Wallet",
            },
            FormItem::AlchemyApiKey => {
                let mut config = Config::load()?;
                if config.alchemy_api_key.is_none() {
                    config.alchemy_api_key = Some("".to_string());
                };

                FormWidget::InputBox {
                    label: "Alchemy API Key",
                    text: config.alchemy_api_key.unwrap_or_default(),
                    empty_text: Some("Please get an Alchemy API key from https://www.alchemy.com/"),
                    currency: None,
                }
            }
            FormItem::Save => FormWidget::Button { label: "Save" },
            FormItem::Display => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

pub struct SetupPage {
    pub form: Form<FormItem>,
}

impl SetupPage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|form| {
                let config = Config::load()?;
                if config.current_account.is_some() {
                    form.hide_item(FormItem::CreateOrImportWallet);
                }
                if config
                    .alchemy_api_key
                    .map(|s| !s.is_empty())
                    .unwrap_or(false)
                {
                    form.hide_item(FormItem::AlchemyApiKey);
                    form.hide_item(FormItem::Save);
                    form.hide_item(FormItem::Display);
                }
                Ok(())
            })?,
        })
    }
}
impl Component for SetupPage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn reload(&mut self, _ss: &SharedState) -> crate::Result<()> {
        let fresh = Self::new()?;
        self.form = fresh.form;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let display_text = self.form.get_text_mut(FormItem::Display);
        *display_text = "".to_string();

        let mut handle_result = HandleResult::default();

        self.form.handle_event(event, |label, form| {
            if label == FormItem::CreateOrImportWallet {
                handle_result
                    .page_inserts
                    .push(Page::Account(AccountPage::new()?));
            } else {
                handle_result.reload = true;

                let mut config = Config::load()?;
                config.alchemy_api_key = Some(form.get_text(FormItem::AlchemyApiKey).clone());
                config.save()?;
                transmitter.send(Event::ConfigUpdate)?;

                let display_text = form.get_text_mut(FormItem::Display);
                *display_text = "Configuration saved".to_string();
            }
            Ok(())
        })?;

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, ss: &SharedState) -> Rect
    where
        Self: Sized,
    {
        Line::from("Setup").bold().render(area, buf);
        let Ok(area) = area.consume_height(2) else {
            return area;
        };

        if self.form.visible_count() == 0 {
            Line::from("You have completed the setup please press ESC to return back.")
                .render(area, buf);
        } else {
            "Complete the following steps to get started:".render(area, buf);
            let Ok(area) = area.consume_height(2) else {
                return area;
            };

            self.form.render(area, buf, &ss.theme);
        }
        area
    }
}
