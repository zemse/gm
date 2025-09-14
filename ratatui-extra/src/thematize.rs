use ratatui::{
    style::{Modifier, Style},
    widgets::BorderType,
};

pub trait Thematize {
    fn popup(&self) -> Self;

    fn error_popup(&self) -> Self;

    fn block(&self) -> Style;

    fn border_type(&self) -> BorderType;

    fn button_focused(&self) -> Style;

    fn select_focused(&self) -> Style;
}

pub struct DefaultTheme {
    reversed: bool,
}

impl Thematize for DefaultTheme {
    fn popup(&self) -> DefaultTheme {
        DefaultTheme {
            reversed: !self.reversed,
        }
    }

    fn error_popup(&self) -> DefaultTheme {
        DefaultTheme {
            reversed: !self.reversed,
        }
    }

    fn block(&self) -> Style {
        Style::default()
    }

    fn border_type(&self) -> BorderType {
        BorderType::Plain
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

    fn select_focused(&self) -> Style {
        if self.reversed {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::REVERSED)
        } else {
            Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
        }
    }
}
