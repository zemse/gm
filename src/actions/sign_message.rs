use super::account::load_wallet;
use crate::disk::{Config, DiskInterface};

use alloy::{hex, signers::SignerSync};

pub fn sign_message(msg: String) {
    let config = Config::load();

    let wallet = load_wallet(config.current_account).expect("must load wallet");

    let signature = wallet
        .sign_message_sync(msg.as_bytes())
        .expect("signing failed");

    println!("\nMessage: {:?}", msg);
    println!("Account: {:?}", config.current_account);
    println!(
        "Signature: {}\n",
        hex::encode_prefixed(signature.as_bytes())
    );

    // TODO upload to etherscan or similar
}
