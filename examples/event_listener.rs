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
        tx_signature: &str,
        to: &str,
        value: u64,
    ) -> Result<()> {
        println!("Mint event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Signature: {}", tx_signature);
        println!("  To: {}", to);
        println!("  Amount: {}", value);
        Ok(())
    }

    async fn handle_burn(
        &self,
        tx_slot: u64,
        tx_signature: &str,
        from: &str,
        btc_address: &str,
        value: u64,
        operator_id: u64,
    ) -> Result<()> {
        println!("Burn event detected:");
        println!("  Slot: {}", tx_slot);
        println!("  Signature: {}", tx_signature);
        println!("  From: {}", from);
        println!("  BTC Address: {}", btc_address);
        println!("  Amount: {}", value);
        println!("  Operator ID: {}", operator_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://solana-devnet.g.alchemy.com/v2/xS1PQwOzOrX7U4AzG9IYnkgMWcdxQbX4";
    let bitvm_bridge_program_id = "Fdj7bMrz8u4ZLyHt3TAnbdqNxtNwQUtqEtgCM84SNWTG";

    // Initialize program ID and event handler
    let handler = Box::new(BitVMEventHandler);

    let last_signature = None;
    // Create and start event monitor
    let mut monitor = EventMonitor::new(url, bitvm_bridge_program_id, handler, last_signature, 1)?;
    monitor.start_monitoring().await?;
    Ok(())
}
