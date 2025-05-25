// File: src/actions/receive_payment.rs

use crate::disk::{Config, DiskInterface};
use inquire::{Text, Select};

/// A request to receive a payment: account + amount + network.
#[derive(Debug, Clone)]
pub struct PaymentRequest {
    pub account: String,
    pub amount:   String,
    pub network:  String,
}

impl PaymentRequest {
    /// Build the URL string: tx.gm.com/{account}/{amount}/{network}
    pub fn generate_link(&self) -> String {
        format!(
            "tx.gm.com/{}/{}/{}",
            self.account, self.amount, self.network
        )
    }

    /// Prompt the user (via inquire) to fill in fields.
    pub fn from_user_input() -> Self {
        // 1) Load default account (fallback to “unknown.eth”)
        let account = Config::load()
            .current_account
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown.eth".to_string());

        // 2) Ask amount
        let amount = Text::new("Enter amount (e.g. 100USDC):")
            .with_default("0USDC")
            .prompt()
            .unwrap_or_else(|_| "0USDC".to_string());

        // 3) Let them pick a network
        let network_options = vec!["ethereum", "arbitrum", "optimism", "polygon"];
        let network_choice: &str = Select::new("Select network:", network_options)
            .with_vim_mode(true)
            .prompt()
            .unwrap_or("ethereum");
        let network = network_choice.to_string();

        PaymentRequest { account, amount, network }
    }
}