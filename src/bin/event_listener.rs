use anchor_client::solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use anchor_client::solana_sdk::signature::Signature;
use anchor_client::{
    anchor_lang::{AnchorDeserialize, Discriminator},
    solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
};
use base64::{engine::general_purpose, Engine as Base64Engine};
use bitvm_bridge::events::BurnEvent;
use dotenv::dotenv;
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};
use std::env;
use std::{str::FromStr, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;
    let api_key = env::var("HELIUS_API_KEY").expect("cannot find HELIUS_API_KEY in .env");
    let rpc_client =
        RpcClient::new("https://devnet.helius-rpc.com/?api-key=".to_string() + &api_key);

    let program_id = bitvm_bridge::ID;
    // record the last processed signature
    let mut last_signature: Option<Signature> = None;
    let burn_event_discriminator = BurnEvent::discriminator();

    loop {
        let config = GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: last_signature,
            limit: Some(1000),
            commitment: None,
        };

        match rpc_client.get_signatures_for_address_with_config(&program_id, config) {
            Ok(signatures) => {
                if signatures.is_empty() {
                    std::thread::sleep(Duration::from_secs(60));
                    continue;
                }

                for sig_info in signatures.iter().rev() {
                    if let Err(e) =
                        process_signature(&rpc_client, sig_info, &burn_event_discriminator)
                    {
                        eprintln!("Error processing signature {}: {}", sig_info.signature, e);
                        continue;
                    }
                }

                if let Some(sig) = signatures.first() {
                    last_signature = Some(Signature::from_str(&sig.signature)?);
                }
            }
            Err(e) => {
                eprintln!("Error fetching signatures: {}", e);
                std::thread::sleep(Duration::from_secs(60));
            }
        }
    }
}

fn process_signature(
    rpc_client: &RpcClient,
    sig_info: &RpcConfirmedTransactionStatusWithSignature,
    burn_event_discriminator: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let signature = Signature::from_str(&sig_info.signature)?;
    let tx = rpc_client.get_transaction(&signature, UiTransactionEncoding::Json)?;

    if let Some(meta) = tx.transaction.meta {
        if let OptionSerializer::Some(logs) = meta.log_messages {
            for log in logs {
                if !log.starts_with("Program data: ") {
                    continue;
                }

                let data = log.replace("Program data: ", "");
                if let Ok(decoded) = general_purpose::STANDARD.decode(data) {
                    if decoded.len() <= 8 || &decoded[0..8] != burn_event_discriminator {
                        continue;
                    }

                    if let Ok(burn_event) = BurnEvent::deserialize(&mut &decoded[8..]) {
                        println!("Parsed Burn event:");
                        println!("  sig: {:?}", sig_info.signature);
                        println!("  From: {}", burn_event.from);
                        println!("  BTC address: {}", burn_event.btc_addr);
                        println!("  Amount: {}", burn_event.value);
                        println!("  Operator ID: {}", burn_event.operator_id);
                    }
                }
            }
        }
    }
    Ok(())
}
