use anyhow::Result;
use async_trait::async_trait;
use solana_client_sdk::events::{EventHandler, EventMonitor};

// Implementation of event handler
struct BitVMEventHandler;

#[async_trait]
impl EventHandler for BitVMEventHandler {
    async fn handle_mint(
        &self,
        tx_slot: u64,
        tx_block_time: u64,
        tx_signature: &str,
        to: &str,
        value: u64,
    ) -> Result<()> {
        println!("Mint event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Block Time: {}", tx_block_time);
        println!("  Signature: {}", tx_signature);
        println!("  To: {}", to);
        println!("  Amount: {}", value);
        Ok(())
    }

    async fn handle_burn(
        &self,
        tx_slot: u64,
        tx_block_time: u64,
        tx_signature: &str,
        from: &str,
        btc_address: &str,
        fee_rate: u32,
        value: u64,
        operator_id: u64,
    ) -> Result<()> {
        println!("Burn event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Block Time: {}", tx_block_time);
        println!("  Signature: {}", tx_signature);
        println!("  From: {}", from);
        println!("  BTC Address: {}", btc_address);
        println!("  Fee Rate: {}", fee_rate);
        println!("  Amount: {}", value);
        println!("  Operator ID: {}", operator_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "8hPLqJVKkmSVoM7JYvFJ8KN5B2RTrJxx8rbBoh8hX1An";

    // Initialize program ID and event handler
    let handler = Box::new(BitVMEventHandler);

    let last_signature = None;
    // Create and start event monitor
    let mut monitor = EventMonitor::new(url, bitvm_bridge_program_id, handler, last_signature, 1)?;
    monitor.start_monitoring().await?;
    Ok(())
}
