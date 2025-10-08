use ratatui::crossterm::event::{Event, KeyCode};

use crate::extensions::EventExt;

#[derive(Debug)]
pub enum WidgetEvent {
    Tick,
    InputEvent(Event),
}

impl From<Event> for WidgetEvent {
    fn from(value: Event) -> Self {
        WidgetEvent::InputEvent(value)
    }
}

impl WidgetEvent {
    pub fn input_event(&self) -> Option<&Event> {
        match self {
            Self::InputEvent(event) => Some(event),
            _ => None,
        }
    }

    pub fn key_event(&self) -> Option<&ratatui::crossterm::event::KeyEvent> {
        self.input_event().and_then(|event| event.key_event())
    }

    pub fn is_tick(&self) -> bool {
        matches!(self, Self::Tick)
    }

    pub fn is_key_press(&self) -> bool {
        self.input_event()
            .map(|event| event.is_key_press())
            .unwrap_or(false)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input_event()
            .map(|event| event.is_key_pressed(key))
            .unwrap_or(false)
    }

    pub fn is_mouse_left_click(&self) -> bool {
        self.input_event()
            .map(|event| event.is_mouse_left_click())
            .unwrap_or(false)
    }
}
