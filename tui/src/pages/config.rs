use std::{
    str::FromStr,
    sync::{atomic::AtomicBool, mpsc, Arc},
};

use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};

use crate::{
    app::SharedState,
    events::Event,
    theme::{self, ThemeName},
    traits::{Actions, Component},
};
use gm_ratatui_extra::{
    act::Act,
    widgets::{
        filter_select_popup::FilterSelectPopup,
        form::{Form, FormItemIndex, FormWidget},
    },
};
use gm_utils::disk::{Config, DiskInterface};

#[derive(Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    AlchemyApiKey,
    TestnetMode,
    DeveloperMode,
    Theme,
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
            FormItem::Theme => FormWidget::SelectInput {
                label: "Theme",
                text: String::new(),
                empty_text: Some("Select a theme"),
                popup: FilterSelectPopup::new("Select a theme", Some("No themes available")),
            },
            FormItem::SaveButton => FormWidget::Button { label: "Save" },
            FormItem::DisplayText => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

pub struct ConfigPage {
    pub form: Form<FormItem, crate::Error>,
}

impl ConfigPage {
    pub fn new() -> crate::Result<Self> {
        let form = Form::init(|form| {
            let config = Config::load()?;
            *form.get_text_mut(FormItem::AlchemyApiKey) =
                config.alchemy_api_key.clone().unwrap_or_default();
            *form.get_boolean_mut(FormItem::TestnetMode) = config.testnet_mode;
            *form.get_boolean_mut(FormItem::DeveloperMode) = config.developer_mode;
            *form.get_text_mut(FormItem::Theme) = config.theme_name.clone();
            let popup = form.get_popup_mut(FormItem::Theme);
            popup.set_items(Some(ThemeName::list()));
            popup.set_cursor(&config.theme_name);
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
    ) -> crate::Result<Actions> {
        let display_text = self.form.get_text_mut(FormItem::DisplayText);
        *display_text = "".to_string();

        let mut handle_result = Actions::default();

        let r = self.form.handle_event(
            event.key_event(),
            |_, _| Ok(()),
            |_, form| {
                handle_result.reload = true;

                let mut config = Config::load()?;
                config.alchemy_api_key = Some(form.get_text(FormItem::AlchemyApiKey).clone());
                config.testnet_mode = form.get_boolean(FormItem::TestnetMode);
                config.developer_mode = form.get_boolean(FormItem::DeveloperMode);
                let theme_name = form.get_text(FormItem::Theme).clone();
                config.theme_name = theme::ThemeName::from_str(&theme_name)?.to_string();

                config.save()?;

                let display_text = form.get_text_mut(FormItem::DisplayText);
                *display_text = "Configuration saved".to_string();

                Ok(())
            },
        )?;
        handle_result.merge(r);

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
