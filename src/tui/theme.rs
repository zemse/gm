use ratatui::prelude::Color;
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
pub enum Borders {
    Square,
    Rounded,
}
pub struct Theme {
    text: Color,
    bg: Color,
    select: Color,
    select_popup: Color,
    error_popup: Color,
    borders: Borders,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            text: Color::Reset,
            bg: Color::Reset,
            select: Color::Reset,
            select_popup: Color::Reset,
            error_popup: Color::Reset,
            borders: Borders::Square,
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
                select: Color::Yellow,
                select_popup: Color::Blue,
                error_popup: Color::Red,
                borders: Borders::Square,
            },
            ThemeName::DarkModern => Theme {
                text: Color::White,
                bg: Color::Black,
                select: Color::Yellow,
                select_popup: Color::Blue,
                error_popup: Color::Red,
                borders: Borders::Rounded,
            },
        }
    }
}
