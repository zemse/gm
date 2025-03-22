use alloy::{primitives::Address,hex, signers::SignerSync};
use crate::disk::Config;
use crate::actions::account::load_wallet;


pub async fn send_message(to: Address, msg: String) {
    let current_account = Config::current_account();
    let wallet = load_wallet(current_account).expect("must load wallet");

    println!("\nSending on-chain message...");
    println!("From: {:?}", current_account);
    println!("To: {:?}", to);
    println!("Message: {:?}", msg);

    let calldata = hex::encode(msg.as_bytes());

    let tx = wallet.send_transaction(to, calldata).await;

    match tx {
        Ok(tx_hash) => println!("Transaction sent! Hash: {:?}", tx_hash),
        Err(e) => eprintln!("Failed to send transaction: {:?}", e),
    }
}
