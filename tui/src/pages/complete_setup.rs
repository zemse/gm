use std::sync::{atomic::AtomicBool, mpsc, Arc};

use gm_ratatui_extra::{
    act::Act,
    form::{Form, FormItemIndex, FormWidget},
};
use gm_utils::{config::Config, disk_storage::DiskStorageInterface};
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};

use super::{account::AccountPage, Page};
use crate::{
    app::SharedState,
    events::Event,
    traits::{Actions, Component},
};

#[derive(Debug, Display, PartialEq, EnumIter)]
pub enum FormItem {
    Heading,
    SubHeading,
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
            FormItem::Heading => FormWidget::Heading("Complete Setup"),
            FormItem::SubHeading => {
                FormWidget::StaticText("Complete the following steps to get started:")
            }
            FormItem::CreateOrImportWallet => FormWidget::Button {
                label: "Create or Import Wallet",
            },
            FormItem::AlchemyApiKey => FormWidget::InputBox {
                label: "Alchemy API Key",
                text: Config::alchemy_api_key(false).ok().unwrap_or_default(),
                empty_text: Some("Please get an Alchemy API key from https://www.alchemy.com/"),
                currency: None,
            },
            FormItem::Save => FormWidget::Button { label: "Save" },
            FormItem::Display => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct CompleteSetupPage {
    pub form: Form<FormItem, crate::Error>,
}

impl CompleteSetupPage {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            form: Form::init(|form| {
                let config = Config::load()?;
                if config.get_current_account().is_ok() {
                    form.hide_item(FormItem::CreateOrImportWallet);
                }
                if config
                    .get_alchemy_api_key(false)
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
impl Component for CompleteSetupPage {
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
    ) -> crate::Result<Actions> {
        let display_text = self.form.get_text_mut(FormItem::Display);
        *display_text = "".to_string();

        let mut handle_result = Actions::default();

        let r = self.form.handle_event(
            event.key_event(),
            |_, _| Ok(()),
            |label, form| {
                if label == FormItem::CreateOrImportWallet {
                    handle_result
                        .page_inserts
                        .push(Page::Account(AccountPage::new()?));
                } else {
                    handle_result.reload = true;

                    Config::set_alchemy_api_key(form.get_text(FormItem::AlchemyApiKey).clone())?;
                    transmitter.send(Event::ConfigUpdate)?;

                    let display_text = form.get_text_mut(FormItem::Display);
                    *display_text = "Configuration saved".to_string();
                }
                Ok(())
            },
        )?;
        handle_result.merge(r);

        if self.form.valid_count() == 0 {
            handle_result.page_pops += 1;
        }

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, ss: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf, &ss.theme);

        area
    }
}
