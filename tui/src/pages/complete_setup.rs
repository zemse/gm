use std::sync::mpsc;

use gm_ratatui_extra::{
    button::Button,
    form::{Form, FormEvent, FormItemIndex, FormWidget},
    input_box::InputBox,
};
use gm_utils::{config::Config, disk_storage::DiskStorageInterface};
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use super::{account::AccountPage, Page};
use crate::{
    app::SharedState, post_handle_event::PostHandleEventActions, traits::Component, AppEvent,
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
                widget: Button::new("Create or Import Wallet"),
            },
            FormItem::AlchemyApiKey => FormWidget::InputBox {
                widget: InputBox::new("Alchemy API key")
                    .with_empty_text("Please get an Alchemy API key from https://www.alchemy.com/"),
            },
            FormItem::Save => FormWidget::Button {
                widget: Button::new("Save"),
            },
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
        event: &AppEvent,
        area: Rect,
        popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        self.form.set_text(FormItem::Display, "".to_string());

        let mut handle_result = PostHandleEventActions::default();

        if let Some(FormEvent::ButtonPressed(label)) = self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            popup_area,
            &mut handle_result,
        )? {
            if label == FormItem::CreateOrImportWallet {
                handle_result.page_insert(Page::Account(AccountPage::new()?));
            } else {
                handle_result.reload();

                Config::set_alchemy_api_key(
                    self.form.get_text(FormItem::AlchemyApiKey).to_string(),
                )?;
                handle_result.reload();

                self.form
                    .set_text(FormItem::Display, "Configuration saved".to_string());
            }
        }

        if self.form.valid_count() == 0 {
            handle_result.page_pop();
        }

        Ok(handle_result)
    }

    fn render_component(
        &self,
        area: Rect,
        popup_area: Rect,
        buf: &mut Buffer,
        ss: &SharedState,
    ) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, popup_area, buf, &ss.theme);

        area
    }
}
