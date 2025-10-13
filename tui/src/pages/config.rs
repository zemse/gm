use std::{str::FromStr, sync::mpsc};

use gm_utils::{config::Config, disk_storage::DiskStorageInterface};
use ratatui::{buffer::Buffer, layout::Rect};
use strum::{Display, EnumIter};
use tokio_util::sync::CancellationToken;

use crate::{
    app::SharedState,
    post_handle_event::PostHandleEventActions,
    theme::{self, ThemeName},
    traits::Component,
    AppEvent,
};
use gm_ratatui_extra::{
    boolean_input::BooleanInput,
    button::Button,
    form::FormEvent,
    input_box::InputBox,
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
                widget: InputBox::new("Alchemy API key")
                    .with_empty_text("Please get an Alchemy API key from https://www.alchemy.com/"),
            },
            FormItem::TestnetMode => FormWidget::BooleanInput {
                widget: BooleanInput::new("Testnet Mode", false),
            },
            FormItem::DeveloperMode => FormWidget::BooleanInput {
                widget: BooleanInput::new("Developer Mode", false),
            },
            FormItem::Theme => FormWidget::SelectInput {
                widget: InputBox::new("Theme")
                    .with_empty_text("Select a theme")
                    .make_immutable(true),
                popup: FilterSelectPopup::new("Select a theme")
                    .with_empty_text("No themes available"),
            },
            FormItem::HeliosEnabled => FormWidget::BooleanInput {
                // TODO remove the restart requirement for helios to be toggled from there
                widget: BooleanInput::new("Enable Helios (requires restart)", false),
            },
            FormItem::SaveButton => FormWidget::Button {
                widget: Button::new("Save"),
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
            form.set_text(
                FormItem::AlchemyApiKey,
                config.get_alchemy_api_key(false).unwrap_or_default(),
            );
            *form.get_boolean_mut(FormItem::TestnetMode) = config.get_testnet_mode();
            *form.get_boolean_mut(FormItem::DeveloperMode) = config.get_developer_mode();
            form.set_text(FormItem::Theme, config.get_theme_name().to_string());
            let popup = form.get_popup_mut(FormItem::Theme);
            popup.set_items(Some(ThemeName::list()));
            popup.set_focused_item(config.get_theme_name().to_string());
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
        popup_area: Rect,
        _transmitter: &mpsc::Sender<AppEvent>,
        _shutdown_signal: &CancellationToken,
        _shared_state: &SharedState,
    ) -> crate::Result<PostHandleEventActions> {
        self.form.set_text(FormItem::DisplayText, "".to_string());

        let mut handle_result = PostHandleEventActions::default();

        if let Some(FormEvent::ButtonPressed(label)) = self.form.handle_event(
            event.widget_event().as_ref(),
            area,
            popup_area,
            &mut handle_result,
        )? {
            if label == FormItem::SaveButton {
                handle_result.reload();

                let mut config = Config::load()?;
                config.set_values(
                    Some(self.form.get_text(FormItem::AlchemyApiKey).to_string()),
                    self.form.get_boolean(FormItem::TestnetMode),
                    self.form.get_boolean(FormItem::DeveloperMode),
                    {
                        let theme_name = self.form.get_text(FormItem::Theme);
                        theme::ThemeName::from_str(&theme_name)
                            .unwrap_or_default()
                            .to_string()
                    },
                    self.form.get_boolean(FormItem::HeliosEnabled),
                )?;

                self.form
                    .set_text(FormItem::DisplayText, "Configuration saved".to_string());
            }
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
