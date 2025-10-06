use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    widgets::BorderType,
};

pub trait Thematize {
    fn popup(&self) -> Self;

    fn error_popup(&self) -> Self;

    fn style(&self) -> Style;

    fn style_dim(&self) -> Style;

    fn border_type(&self) -> BorderType;

    fn button_focused(&self) -> Style;

    fn button_notfocused(&self) -> Style;

    fn select_focused(&self) -> Style;

    fn select_active(&self) -> Style;

    fn select_inactive(&self) -> Style;

    fn boxed(&self) -> bool;
}

#[derive(Default)]
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

    fn style(&self) -> Style {
        Style::default()
    }

    fn style_dim(&self) -> Style {
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

    fn button_notfocused(&self) -> Style {
        Style::default()
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

    fn select_active(&self) -> Style {
        Style::default().bold()
    }

    fn select_inactive(&self) -> Style {
        Style::default().not_bold().fg(Color::Gray)
    }

    fn boxed(&self) -> bool {
        true
    }
}
