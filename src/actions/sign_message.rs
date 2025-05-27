use super::account::load_wallet;
use crate::disk::Config;

use alloy::{hex, signers::SignerSync};

pub fn sign_message(msg: String) {
    let current_account = Config::current_account().unwrap();
    let wallet = load_wallet(current_account).expect("must load wallet");

    let signature = wallet
        .sign_message_sync(msg.as_bytes())
        .expect("signing failed");

    println!("\nMessage: {:?}", msg);
    println!("Account: {:?}", current_account);
    println!(
        "Signature: {}\n",
        hex::encode_prefixed(signature.as_bytes())
    );

    // TODO upload to etherscan or similar
}
