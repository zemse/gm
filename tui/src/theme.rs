use gm_ratatui_extra::thematize::Thematize;
use ratatui::prelude::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::BorderType;
use std::fmt::Formatter;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Default, Debug, EnumIter)]
pub enum ThemeName {
    #[default]
    Monochrome,
    MonochromeModern,
    Dark,
    DarkModern,
}

impl std::fmt::Display for ThemeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::str::FromStr for ThemeName {
    type Err = crate::Error;

    fn from_str(theme_name: &str) -> crate::Result<Self> {
        match theme_name {
            "Monochrome" => Ok(Self::Monochrome),
            "MonochromeModern" => Ok(Self::MonochromeModern),
            "Dark" => Ok(Self::Dark),
            "DarkModern" => Ok(Self::DarkModern),
            _ => Err(crate::Error::InternalError(format!(
                "Unknown theme name: {theme_name}"
            ))),
        }
    }
}

impl ThemeName {
    pub fn list() -> Vec<String> {
        Self::iter().map(|theme| theme.to_string()).collect()
    }
}

#[derive(Clone)]
pub struct Theme {
    pub text: Option<Color>,
    pub bg: Option<Color>,
    pub reversed: bool,
    pub select_focus: Option<Color>,
    pub popup_reversed: bool,
    pub popup_bg: Option<Color>,
    pub error_popup_bg: Option<Color>,
    pub border_type: BorderType,
}

// impl From<&Theme> for Style {
//     fn from(theme: &Theme) -> Self {}
// }

// impl From<&mut Theme> for Style {
//     fn from(theme: &mut Theme) -> Self {
//         let theme = &*theme;
//         theme.into()
//     }
// }

// impl From<&Theme> for BorderType {
//     fn from(val: &Theme) -> Self {
//         val.border_type
//     }
// }

impl Theme {
    pub fn new(theme_name: ThemeName) -> Theme {
        match theme_name {
            ThemeName::Monochrome => Theme {
                text: None,
                bg: None,
                select_focus: None,
                reversed: false,
                popup_reversed: true,
                popup_bg: None,
                error_popup_bg: None,
                border_type: BorderType::Plain,
            },
            ThemeName::MonochromeModern => Theme {
                text: None,
                bg: None,
                select_focus: None,
                reversed: false,
                popup_reversed: true,
                popup_bg: None,
                error_popup_bg: None,
                border_type: BorderType::Rounded,
            },
            ThemeName::Dark => Theme {
                text: Some(Color::White),
                bg: Some(Color::Black),
                select_focus: None,
                reversed: false,
                popup_reversed: false,
                popup_bg: Some(Color::Blue),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
            ThemeName::DarkModern => Theme {
                text: Some(Color::White),
                bg: Some(Color::Black),
                select_focus: None,
                reversed: false,
                popup_reversed: false,
                popup_bg: Some(Color::Blue),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Rounded,
            },
        }
    }
}

impl Thematize for Theme {
    fn button_focused(&self) -> Style {
        if self.reversed {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }

    fn border_type(&self) -> BorderType {
        self.border_type
    }

    fn block(&self) -> Style {
        let mut style = Style::default();
        if let Some(text_color) = self.text {
            style = style.fg(text_color);
        }
        if let Some(bg_color) = self.bg {
            style = style.bg(bg_color);
        }
        if self.reversed {
            style = style.add_modifier(Modifier::REVERSED);
        } else {
            style = style.remove_modifier(Modifier::REVERSED);
        }
        style
    }

    fn select_focused(&self) -> Style {
        if let Some(select_focus) = self.select_focus {
            Style::default()
                .bg(select_focus)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }

    // fn select_popup(&self) -> Style {
    //     if let Some(select_focus) = self.select_focus {
    //         Style::default()
    //             .bg(select_focus)
    //             .add_modifier(Modifier::BOLD)
    //     } else {
    //         Style::default()
    //             .add_modifier(Modifier::BOLD)
    //             .remove_modifier(Modifier::REVERSED)
    //     }
    // }

    fn popup(&self) -> Theme {
        Theme {
            bg: self.popup_bg,
            reversed: self.popup_reversed,
            ..self.clone()
        }
    }

    fn error_popup(&self) -> Theme {
        let s = self.popup();
        Theme {
            bg: s.error_popup_bg,
            ..s.clone()
        }
    }
}
