use anyhow::anyhow;
use anyhow::Result;
use bs58;
use solana_bitvm_bridge_client::BitvmBridgeClient;
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow!("SOLANA_PRIVATE_KEY not found in environment"))?;

    let private_key = bs58::decode(private_key).into_vec()?;
    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "HWyR228YqC5im7bgpzU2ZDBf5TnPJKDQYe5xoHEowxm6";
    let btc_light_client_program_id = "H2WfnhhCB3hPsSjNSbzQDw4ivDWjAHSo1QwXc6kZxMG1";
    // Initialize payer from private key
    let payer = Keypair::from_bytes(&private_key)?;

    // Create client instance
    let client = BitvmBridgeClient::new(
        url.to_string(),
        bitvm_bridge_program_id.to_string(),
        btc_light_client_program_id.to_string(),
        payer,
    )?;

    // Execute burn operation
    let result = client
        .burn_tokens(150000, "btc_address_string".to_string(), 1)
        .await?;

    println!("Burn success! Signature: {}", result);
    Ok(())
}
