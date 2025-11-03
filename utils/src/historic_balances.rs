use std::collections::HashMap;

use alloy::primitives::{Address, ChainId, U256};

pub struct HistoricBalance {
    pub chain_id: ChainId,
    pub block_number: u64,
    pub timestamp: u64,
    pub token: Option<Address>,
    pub price_in_usd: f64,
    pub balance: U256,
}

#[allow(dead_code)]
pub struct Store {
    block_numbers: HashMap<ChainId, Vec<u64>>,
    balances: Vec<HistoricBalance>,
}

// TODO Given a time gap I need to query historical balances in it
// First figure out the block numbers that correspond to the approx timestamps
// Then for each block number query the balances for the given address and token (or native)
// Make sure we don't make a lot of requests at the same time to avoid rate limiting
