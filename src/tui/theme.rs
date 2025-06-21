use ratatui::prelude::Color;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::BorderType;
use std::fmt::Formatter;

#[derive(Default, Debug)]
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
}
#[derive(Clone)]
pub enum Borders {
    Square,
    Rounded,
}
#[derive(Clone)]
pub struct Theme {
    pub text: Color,
    pub bg: Color,
    pub select: Option<Color>,
    pub select_popup: Option<Color>,
    pub error_popup: Option<Color>,
    pub borders: Borders,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            text: Color::Reset,
            bg: Color::Reset,
            select: None,
            select_popup: None,
            error_popup: None,
            borders: Borders::Square,
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
        match val.borders {
            Borders::Square => BorderType::Plain,
            Borders::Rounded => BorderType::Rounded,
        }
    }
}
impl Theme {
    pub fn new(theme_name: ThemeName) -> Theme {
        match theme_name {
            ThemeName::Monochrome => Default::default(),
            ThemeName::MonochromeModern => Theme {
                borders: Borders::Rounded,
                ..Default::default()
            },
            ThemeName::Dark => Theme {
                text: Color::White,
                bg: Color::Black,
                select: Some(Color::Yellow),
                select_popup: Some(Color::Blue),
                error_popup: Some(Color::Red),
                borders: Borders::Square,
            },
            ThemeName::DarkModern => Theme {
                text: Color::White,
                bg: Color::Black,
                select: Some(Color::Yellow),
                select_popup: Some(Color::Blue),
                error_popup: Some(Color::Red),
                borders: Borders::Rounded,
            },
        }
    }
    pub fn select(&self) -> Option<Style> {
        self.select
            .map(|select| Into::<Style>::into(self).bg(select))
    }
    pub fn select_popup(&self) -> Style {
        if let Some(select_popup) = self.select_popup {
            Into::<Style>::into(self)
                .bg(select_popup)
                .remove_modifier(Modifier::REVERSED)
        } else {
            Style::default().remove_modifier(Modifier::REVERSED)
        }
    }
    pub fn error_popup(&self) -> Theme {
        if let Some(error_popup) = self.error_popup {
            Theme {
                bg: error_popup,
                ..self.clone()
            }
        } else {
            Theme::default()
        }
    }
}
