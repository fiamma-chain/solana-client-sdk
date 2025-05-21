use anyhow::anyhow;
use anyhow::Result;

use solana_client_sdk::bridge_client::BitvmBridgeClient;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow!("SOLANA_PRIVATE_KEY not found in environment"))?;

    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "8hPLqJVKkmSVoM7JYvFJ8KN5B2RTrJxx8rbBoh8hX1An";
    let btc_light_client_program_id = "F14fXdFjBbhEjXjFuhSharSt7UxGPWknkKYmpJd2Rvka";

    // Create client instance
    let client = BitvmBridgeClient::new(
        url,
        bitvm_bridge_program_id,
        btc_light_client_program_id,
        &private_key,
    )?;

    // Execute burn operation
    let result = client
        .burn_tokens(
            300000,
            "bcrt1phcnl4zcl2fu047pv4wx6y058v8u0n02at6lthvm7pcf2wrvjm5tqatn90k",
            1,
        )
        .await?;

    println!("Burn success! Signature: {}", result);
    Ok(())
}
