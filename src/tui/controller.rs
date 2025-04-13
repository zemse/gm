pub mod navigation;

use crossterm::event::{KeyCode, KeyEventKind};
use navigation::Navigation;

use super::events::Event;

#[derive(Default)]
pub struct Controller {
    pub exit: bool,
    pub eth_price: Option<String>,
    pub navigation: Navigation,
}
impl Controller {
    pub fn exit(&self) -> bool {
        self.exit
    }

    /// Make changes to the Tui state based on the event received.
    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Input(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            self.exit = true;
                        }
                        // KeyCode::Esc => {
                        //     if !self.navigation.esc() {
                        //         self.exit = true;
                        //     }
                        // }
                        // KeyCode::Enter => {
                        //     // go to next menu
                        //     self.navigation.enter();
                        // }
                        KeyCode::Up => {
                            self.navigation.up();
                        }
                        KeyCode::Down => {
                            self.navigation.down();
                        }
                        // TODO
                        // KeyCode::Left => {}
                        // KeyCode::Right => {}
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
