use serde::{Deserialize, Serialize};

pub mod bridge_client;
pub mod events;
pub mod query_client;
pub mod utils;

#[derive(Debug)]
pub enum TransactionEvent {
    Mint(MintEventData),
    Burn(BurnEventData),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MintEventData {
    pub to: String,
    pub value: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BurnEventData {
    pub from: String,
    pub btc_addr: String,
    pub value: u64,
    pub operator_id: u64,
}
