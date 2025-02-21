use anyhow::anyhow;
use anyhow::Result;

use anchor_client::solana_sdk::signature::{Keypair, Signer};
use solana_bitvm_bridge_client::bridge_client::BitvmBridgeClient;
use solana_transaction_status::option_serializer::OptionSerializer;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "HWyR228YqC5im7bgpzU2ZDBf5TnPJKDQYe5xoHEowxm6";
    let btc_light_client_program_id = "H2WfnhhCB3hPsSjNSbzQDw4ivDWjAHSo1QwXc6kZxMG1";
    dotenv::dotenv().ok();

    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow!("SOLANA_PRIVATE_KEY not found in environment"))?;

    let private_key = bs58::decode(private_key).into_vec()?;
    let payer = Keypair::from_bytes(&private_key)?;
    let recipient = payer.pubkey();

    println!("Current payer: {}", recipient);

    // Create query client

    // Create client instance
    let client = BitvmBridgeClient::new(
        url.to_string(),
        bitvm_bridge_program_id.to_string(),
        btc_light_client_program_id.to_string(),
        payer,
    )?;

    // Sample transaction ID
    let tx_id = [8u8; 32];

    // Execute mint operation
    let result = client
        .mint_tokens(recipient.to_string(), tx_id, 1000000)
        .await?;

    println!("Mint success! Signature: {}", result);

    println!("Waiting for transaction confirmation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let mut retries = 10;
    let mut tx_details = None;

    while retries > 0 {
        match client.get_transaction(&result).await {
            Ok(details) => {
                tx_details = Some(details);
                break;
            }
            Err(_) => {
                retries -= 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                println!("Retrying... {} attempts left", retries);
            }
        }
    }

    if let Some(details) = tx_details {
        if let Some(meta) = details.transaction.meta {
            println!("Transaction logs:");
            if let OptionSerializer::Some(logs) = meta.log_messages {
                for log in logs {
                    println!("{}", log);
                }
            }
        }
    } else {
        println!("Failed to get transaction details after multiple attempts");
    }

    Ok(())
}
