use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use async_trait::async_trait;
use solana_client_sdk::events::{EventHandler, EventMonitor};
use std::str::FromStr;

// Implementation of event handler
struct BitVMEventHandler;

#[async_trait]
impl EventHandler for BitVMEventHandler {
    async fn handle_mint(&self, tx_slot: u64, tx_signature: String, to: String, value: u64) {
        println!("Mint event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Signature: {}", tx_signature);
        println!("  To: {}", to);
        println!("  Amount: {}", value);
    }

    async fn handle_burn(
        &self,
        tx_slot: u64,
        tx_signature: String,
        from: String,
        btc_address: String,
        value: u64,
        operator_id: u64,
    ) {
        println!("Burn event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Signature: {}", tx_signature);
        println!("  From: {}", from);
        println!("  BTC Address: {}", btc_address);
        println!("  Amount: {}", value);
        println!("  Operator ID: {}", operator_id);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "HWyR228YqC5im7bgpzU2ZDBf5TnPJKDQYe5xoHEowxm6";

    // Initialize program ID and event handler
    let program_id = Pubkey::from_str(bitvm_bridge_program_id).unwrap();
    let handler = Box::new(BitVMEventHandler);

    let last_signature = None;
    // Create and start event monitor
    let mut monitor = EventMonitor::new(url, program_id, handler, last_signature, 1);
    monitor.start_monitoring().await?;
    Ok(())
}
