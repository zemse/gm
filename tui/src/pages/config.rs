use std::{str::FromStr, sync::mpsc};

use gm_utils::{config::Config, disk_storage::DiskStorageInterface};
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    theme::{self, ThemeName},
    traits::{Actions, Component},
    AppEvent,
};
use gm_ratatui_extra::{
    act::Act,
    widgets::{
        filter_select_popup::FilterSelectPopup,
        form::{Form, FormItemIndex, FormWidget},
    },
};

#[derive(Debug, Display, EnumIter, PartialEq)]
pub enum FormItem {
    Heading,
    AlchemyApiKey,
    TestnetMode,
    DeveloperMode,
    Theme,
    HeliosEnabled,
    DisplayText,
    SaveButton,
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
            FormItem::HeliosEnabled => FormWidget::BooleanInput {
                label: "Enable Helios (requires restart)",
                value: false,
            },
            FormItem::SaveButton => FormWidget::Button {
                label: "Save",
                hover_focus: false,
            },
            FormItem::DisplayText => FormWidget::DisplayText(String::new()),
        };
        Ok(widget)
    }
}

#[derive(Debug)]
pub struct ConfigPage {
    pub form: Form<FormItem, crate::Error>,
}

impl ConfigPage {
    pub fn new() -> crate::Result<Self> {
        let form = Form::init(|form| {
            let config = Config::load()?;
            *form.get_text_mut(FormItem::AlchemyApiKey) =
                config.get_alchemy_api_key(false).unwrap_or_default();
            *form.get_boolean_mut(FormItem::TestnetMode) = config.get_testnet_mode();
            *form.get_boolean_mut(FormItem::DeveloperMode) = config.get_developer_mode();
            *form.get_text_mut(FormItem::Theme) = config.get_theme_name().to_string();
            let popup = form.get_popup_mut(FormItem::Theme);
            popup.set_items(Some(ThemeName::list()));
            popup.set_cursor(&config.get_theme_name().to_string());
            *form.get_boolean_mut(FormItem::HeliosEnabled) = config.get_helios_enabled();
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
        event: &AppEvent,
        area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<Actions> {
        let display_text = self.form.get_text_mut(FormItem::DisplayText);
        *display_text = "".to_string();

        let mut handle_result = Actions::default();

        let r = self.form.handle_event(
            event.input_event(),
            area,
            |_, _| Ok(()),
            |_, form| {
                handle_result.reload = true;

                let mut config = Config::load()?;
                config.set_values(
                    Some(form.get_text(FormItem::AlchemyApiKey).clone()),
                    form.get_boolean(FormItem::TestnetMode),
                    form.get_boolean(FormItem::DeveloperMode),
                    {
                        let theme_name = form.get_text(FormItem::Theme).clone();
                        theme::ThemeName::from_str(&theme_name)
                            .unwrap_or_default()
                            .to_string()
                    },
                    form.get_boolean(FormItem::HeliosEnabled),
                )?;

                let display_text = form.get_text_mut(FormItem::DisplayText);
                *display_text = "Configuration saved".to_string();

                handle_result.page_pop = true;

                Ok(())
            },
        )?;
        handle_result.merge(r);

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
