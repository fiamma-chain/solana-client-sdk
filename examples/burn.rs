use anyhow::anyhow;
use anyhow::Result;

use solana_client_sdk::bridge_client::BitvmBridgeClient;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow!("SOLANA_PRIVATE_KEY not found in environment"))?;

    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "Fdj7bMrz8u4ZLyHt3TAnbdqNxtNwQUtqEtgCM84SNWTG";
    let btc_light_client_program_id = "Dna44JZJLBgg3UCh5VCQvxgsPVhahZmL6o4Hc4ewqk6i";

    // Create client instance
    let client = BitvmBridgeClient::new(
        url,
        bitvm_bridge_program_id,
        btc_light_client_program_id,
        &private_key,
    )?;

    // Execute burn operation
    let result = client.burn_tokens(1000000, "btc_address_string", 1).await?;

    println!("Burn success! Signature: {}", result);
    Ok(())
}
