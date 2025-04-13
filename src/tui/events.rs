pub mod eth_price;
pub mod input;

pub enum Event {
    Input(crossterm::event::KeyEvent),
    EthPriceUpdate(String),
}
