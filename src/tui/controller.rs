pub mod navigation;

use crossterm::event::{KeyCode, KeyEventKind};
use navigation::Navigation;

use super::events::Event;

#[derive(Default)]
pub struct Controller {
    pub exit: bool,
    pub eth_price: Option<String>,
    pub cursor: Navigation,
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
                        KeyCode::Esc => {
                            if !self.cursor.esc() {
                                self.exit = true;
                            }
                        }
                        KeyCode::Enter => {
                            // go to next menu
                            self.cursor.enter();
                        }
                        KeyCode::Up => {
                            self.cursor.up();
                        }
                        KeyCode::Down => {
                            self.cursor.down();
                        }
                        KeyCode::Left => {
                            self.cursor.left();
                        }
                        KeyCode::Right => {
                            // TODO first implement what to do in the right side
                            // self.cursor.right();
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
