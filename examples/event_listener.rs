use anyhow::Result;
use async_trait::async_trait;
use solana_bitvm_bridge_client::events::{EventHandler, EventMonitor};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// Implementation of event handler
struct BitVMEventHandler;

#[async_trait]
impl EventHandler for BitVMEventHandler {
    async fn handle_mint(&self, to: String, value: u64) {
        println!("Mint event detected:");
        println!("  To: {}", to);
        println!("  Amount: {}", value);
    }

    async fn handle_burn(&self, from: String, btc_address: String, value: u64, operator_id: u64) {
        println!("Burn event detected:");
        println!("  From: {}", from);
        println!("  Amount: {}", value);
        println!("  BTC Address: {}", btc_address);
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

    // Create and start event monitor
    let mut monitor = EventMonitor::new(url, program_id, handler);
    monitor.start_monitoring().await?;
    Ok(())
}
