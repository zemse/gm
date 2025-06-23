use ratatui::prelude::Color;
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
        write!(f, "{:?}", self)
    }
}

impl ThemeName {
    pub fn from_str(theme_name: &str) -> Self {
        match theme_name {
            "Monochrome" => Self::Monochrome,
            "MonochromeModern" => Self::MonochromeModern,
            "Dark" => Self::Dark,
            "DarkModern" => Self::DarkModern,
            _ => Default::default(),
        }
    }

    pub fn list() -> Vec<String> {
        Self::iter().map(|theme| theme.to_string()).collect()
    }
}

#[derive(Clone)]
pub struct Theme {
    pub text: Color,
    pub bg: Color,
    pub select: Option<Color>,
    pub popup_bg: Option<Color>,
    pub error_popup_bg: Option<Color>,
    pub border_type: BorderType,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            text: Color::Reset,
            bg: Color::Reset,
            select: None,
            popup_bg: None,
            error_popup_bg: None,
            border_type: BorderType::Plain,
        }
    }
}

impl From<&Theme> for Style {
    fn from(val: &Theme) -> Self {
        Style::default().bg(val.bg).fg(val.text)
    }
}

impl From<&mut Theme> for Style {
    fn from(val: &mut Theme) -> Self {
        Style::default().bg(val.bg).fg(val.text)
    }
}

impl From<&Theme> for BorderType {
    fn from(val: &Theme) -> Self {
        val.border_type
    }
}

impl Theme {
    pub fn new(theme_name: ThemeName) -> Theme {
        match theme_name {
            ThemeName::Monochrome => Default::default(),
            ThemeName::MonochromeModern => Theme {
                border_type: BorderType::Rounded,
                ..Default::default()
            },
            ThemeName::Dark => Theme {
                text: Color::White,
                bg: Color::Black,
                select: Some(Color::Yellow),
                popup_bg: Some(Color::Blue),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
            ThemeName::DarkModern => Theme {
                text: Color::White,
                bg: Color::Black,
                select: Some(Color::Yellow),
                popup_bg: Some(Color::Blue),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Rounded,
            },
        }
    }

    pub fn select(&self) -> Option<Style> {
        self.select
            .map(|select| Into::<Style>::into(self).bg(select))
    }

    pub fn popup_bg(&self) -> Theme {
        if let Some(popup_bg) = self.popup_bg {
            Theme {
                bg: popup_bg,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }

    pub fn error_popup_bg(&self) -> Theme {
        if let Some(error_popup_bg) = self.error_popup_bg {
            Theme {
                bg: error_popup_bg,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}
