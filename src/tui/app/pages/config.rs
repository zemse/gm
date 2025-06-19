use std::sync::{atomic::AtomicBool, mpsc, Arc};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use strum::EnumIter;

use crate::{
    disk::{Config, DiskInterface},
    tui::{
        app::{
            widgets::form::{Form, FormItemIndex, FormWidget},
            SharedState,
        },
        events::Event,
        theme,
        traits::{Component, HandleResult},
    },
};

#[derive(EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    AlchemyApiKey,
    TestnetMode,
    DeveloperMode,
    ThemeName,
    SaveButton,
    DisplayText,
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
            FormItem::Heading => FormWidget::Heading("Configuration"),
            FormItem::AlchemyApiKey => FormWidget::InputBox {
                label: "Alchemy API key",
                text: String::new(),
                empty_text: Some("Please get an Alchemy API key from https://www.alchemy.com/"),
                currency: None,
            },
            FormItem::TestnetMode => FormWidget::BooleanInput {
                label: "Testnet Mode",
                value: false,
            },
            FormItem::DeveloperMode => FormWidget::BooleanInput {
                label: "Developer Mode",
                value: false,
            },
            FormItem::ThemeName => FormWidget::InputBox {
                label: "Theme Name",
                text: "Monochrome".to_string(),
                empty_text: Some("Enter a theme name."),
                currency: None,
            },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::DisplayText => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

pub struct ConfigPage {
    pub form: Form<FormItem>,
}

impl ConfigPage {
    pub fn new() -> crate::Result<Self> {
        let form = Form::init(|form| {
            let config = Config::load()?;
            *form.get_text_mut(FormItem::AlchemyApiKey) =
                config.alchemy_api_key.clone().unwrap_or_default();
            *form.get_boolean_mut(FormItem::TestnetMode) = config.testnet_mode;
            *form.get_boolean_mut(FormItem::DeveloperMode) = config.developer_mode;
            *form.get_text_mut(FormItem::ThemeName) = config.theme_name;
            Ok(())
        })?;

        Ok(Self { form })
    }
}

impl Component for ConfigPage {
    fn set_focus(&mut self, focus: bool) {
        self.form.set_form_focus(focus);
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _area: Rect,
        _transmitter: &mpsc::Sender<Event>,
        _shutdown_signal: &Arc<AtomicBool>,
        _shared_state: &SharedState,
    ) -> crate::Result<HandleResult> {
        let display_text = self.form.get_text_mut(FormItem::DisplayText);
        *display_text = "".to_string();

        let mut handle_result = HandleResult::default();

        self.form.handle_event(event, |_, form| {
            handle_result.reload = true;

            let mut config = Config::load()?;
            config.alchemy_api_key = Some(form.get_text(FormItem::AlchemyApiKey).clone());
            config.testnet_mode = form.get_boolean(FormItem::TestnetMode);
            config.developer_mode = form.get_boolean(FormItem::DeveloperMode);
            let theme_name = form.get_text(FormItem::ThemeName).clone();
            config.theme_name = theme::ThemeName::from_str(&theme_name).to_string();

            config.save()?;

            let display_text = form.get_text_mut(FormItem::DisplayText);
            *display_text = "Configuration saved".to_string();

            Ok(())
        })?;

        Ok(handle_result)
    }

    fn render_component(&self, area: Rect, buf: &mut Buffer, _: &SharedState) -> Rect
    where
        Self: Sized,
    {
        self.form.render(area, buf);
        area
    }
}
