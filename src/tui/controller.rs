pub mod navigation;

use crossterm::event::{KeyCode, KeyEventKind};
use navigation::Navigation;

use super::events::Event;

#[derive(Default)]
pub struct Controller<'a> {
    pub exit: bool,
    pub eth_price: Option<String>,
    pub navigation: Navigation<'a>,
}
impl Controller<'_> {
    pub fn exit(&self) -> bool {
        self.exit
    }

    /// Make changes to the Tui state based on the event received.
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Input(key_event) => {
                // handle all the navigation and text input captures
                self.navigation.handle(key_event);

                if self.navigation.pages.is_empty() {
                    self.exit = true;
                }

                // check if we should exit on 'q' press
                if key_event.kind == KeyEventKind::Press {
                    #[allow(clippy::single_match)]
                    match key_event.code {
                        KeyCode::Char(char) => {
                            if self.navigation.text_input.is_none() && char == 'q' {
                                self.exit = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::EthPriceUpdate(eth_price) => {
                self.eth_price = Some(eth_price);
            }
        };
    }
}
