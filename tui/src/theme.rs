use gm_ratatui_extra::thematize::Thematize;
use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::BorderType,
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Display, Default, Debug, EnumIter, EnumString)]
pub enum ThemeName {
    Monochrome,
    MonochromeBoxed,
    #[default]
    DarkHacker,
    DarkHackerBoxed,
}

impl ThemeName {
    pub fn list() -> Vec<String> {
        Self::iter().map(|theme| theme.to_string()).collect()
    }
}

#[derive(Clone)]
pub struct Theme {
    pub boxed: bool,

    pub primary_color: Option<Color>,
    pub cancel_color: Option<Color>,

    pub fg: Option<Color>,
    pub fg_dim: Option<Color>,
    pub bg: Option<Color>,
    pub bg_dim: Option<Color>,

    pub reversed: bool,

    pub select_active_bold: Option<bool>,
    pub select_active_fg: Option<Color>,
    pub select_inactive_fg: Option<Color>,
    pub select_focus_fg: Option<Color>,
    pub select_focus_bg: Option<Color>,

    pub popup_reversed: bool,
    pub popup_bg: Option<Color>,
    pub error_popup_bg: Option<Color>,
    pub border_type: BorderType,
}

impl Theme {
    pub fn new(theme_name: ThemeName) -> Theme {
        match theme_name {
            ThemeName::Monochrome => Theme {
                boxed: false,

                primary_color: None,
                cancel_color: None,

                fg: Some(Color::White),
                fg_dim: Some(Color::Gray),
                bg: None,
                bg_dim: Some(Color::DarkGray),

                reversed: false,

                select_active_bold: Some(true),
                select_active_fg: Some(Color::White),
                select_inactive_fg: Some(Color::Gray),
                select_focus_fg: Some(Color::Black),
                select_focus_bg: Some(Color::White),

                popup_reversed: false,
                popup_bg: Some(Color::Black),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
            ThemeName::MonochromeBoxed => Theme {
                boxed: true,

                primary_color: None,
                cancel_color: None,

                fg: Some(Color::White),
                fg_dim: Some(Color::Gray),
                bg: None,
                bg_dim: Some(Color::DarkGray),

                reversed: false,

                select_active_bold: Some(true),
                select_active_fg: Some(Color::White),
                select_inactive_fg: Some(Color::Gray),
                select_focus_fg: Some(Color::Black),
                select_focus_bg: Some(Color::White),

                popup_reversed: false,
                popup_bg: Some(Color::Black),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
            ThemeName::DarkHacker => Theme {
                boxed: false,

                primary_color: Some(Color::LightGreen),
                cancel_color: Some(Color::LightRed),

                fg: Some(Color::White),
                fg_dim: Some(Color::Gray),
                bg: None,
                bg_dim: Some(Color::DarkGray),

                reversed: false,

                select_active_bold: Some(true),
                select_active_fg: Some(Color::White),
                select_inactive_fg: Some(Color::Gray),
                select_focus_fg: Some(Color::Black),
                select_focus_bg: Some(Color::LightGreen),

                popup_reversed: false,
                popup_bg: Some(Color::Black),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
            ThemeName::DarkHackerBoxed => Theme {
                boxed: true,

                primary_color: Some(Color::LightGreen),
                cancel_color: Some(Color::LightRed),

                fg: Some(Color::White),
                fg_dim: Some(Color::Gray),
                bg: None,
                bg_dim: Some(Color::DarkGray),

                reversed: false,

                select_active_bold: Some(true),
                select_active_fg: Some(Color::White),
                select_inactive_fg: Some(Color::Gray),
                select_focus_fg: Some(Color::Black),
                select_focus_bg: Some(Color::LightGreen),

                popup_reversed: false,
                popup_bg: Some(Color::Black),
                error_popup_bg: Some(Color::Red),
                border_type: BorderType::Plain,
            },
        }
    }
}

impl Thematize for Theme {
    fn cursor(&self) -> Style {
        if let Some(primary_color) = self.primary_color {
            Style::default().fg(Color::Black).bg(primary_color)
        } else {
            Style::default().add_modifier(Modifier::REVERSED)
        }
    }

    fn cursor_cancelled(&self) -> Style {
        if let Some(cancel_color) = self.cancel_color {
            Style::default().fg(Color::Black).bg(cancel_color)
        } else {
            Style::default().add_modifier(Modifier::REVERSED)
        }
    }

    fn toast(&self) -> Style {
        Style::default().add_modifier(Modifier::REVERSED)
    }

    fn button_focused(&self) -> Style {
        if self.reversed {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }

    fn button_notfocused(&self) -> Style {
        if let Some(bg_dim) = self.bg_dim {
            Style::default().bg(bg_dim).fg(Color::Black)
        } else {
            Style::default()
        }
    }

    fn border_type(&self) -> BorderType {
        self.border_type
    }

    fn style(&self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg {
            style = style.fg(fg);
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

    fn style_dim(&self) -> Style {
        let mut style = Style::default();
        if let Some(fg) = self.fg_dim {
            style = style.fg(fg);
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
        let mut style = Style::default();

        if let Some(select_focus_fg) = self.select_focus_fg {
            style = style.fg(select_focus_fg)
        };

        if let Some(select_active_bold) = self.select_active_bold {
            if select_active_bold {
                style = style.add_modifier(Modifier::BOLD)
            } else {
                style = style.remove_modifier(Modifier::BOLD)
            }
        };

        if let Some(select_focus_bg) = self.select_focus_bg {
            style = style.bg(select_focus_bg)
        } else {
            style = style.add_modifier(Modifier::REVERSED)
        };

        if self.reversed {
            style = style.remove_modifier(Modifier::REVERSED);
        }

        style
    }

    fn select_active(&self) -> Style {
        let mut style = Style::default();

        if let Some(select_active_fg) = self.select_active_fg {
            style = style.fg(select_active_fg)
        };

        if let Some(select_active_bold) = self.select_active_bold {
            if select_active_bold {
                style = style.add_modifier(Modifier::BOLD)
            } else {
                style = style.remove_modifier(Modifier::BOLD)
            }
        };

        style
    }

    fn select_inactive(&self) -> Style {
        let mut style = Style::default().not_bold();

        if let Some(select_inactive_fg) = self.select_inactive_fg {
            style = style.fg(select_inactive_fg)
        };

        style
    }

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

    fn boxed(&self) -> bool {
        self.boxed
    }
}
