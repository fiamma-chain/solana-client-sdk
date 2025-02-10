use anchor_client::solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use anchor_client::{
    anchor_lang::{AnchorDeserialize, Discriminator},
    solana_client::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signature},
};

use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use bitvm_bridge::events::{BurnEvent, MintEvent};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

/// Event handler trait for processing bridge events
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_mint(&self, tx_slot: u64, tx_signature: String, to: String, value: u64);
    async fn handle_burn(
        &self,
        tx_slot: u64,
        tx_signature: String,
        from: String,
        btc_addr: String,
        value: u64,
        operator_id: u64,
    );
}

/// Monitor for bridge events
pub struct EventMonitor {
    program_id: Pubkey,
    handler: Box<dyn EventHandler>,
    rpc_client: RpcClient,
    last_signature: Option<Signature>,
}

impl EventMonitor {
    pub fn new(
        rpc_url: &str,
        program_id: Pubkey,
        handler: Box<dyn EventHandler>,
        last_signature: Option<Signature>,
    ) -> Self {
        Self {
            program_id,
            handler,
            rpc_client: RpcClient::new(rpc_url.to_string()),
            last_signature,
        }
    }

    pub async fn start_monitoring(&mut self) -> Result<()> {
        // Initialize event discriminators
        let mint_discriminator = MintEvent::discriminator();
        let burn_discriminator = BurnEvent::discriminator();

        loop {
            // Configure signature fetch parameters
            let config = GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: self.last_signature,
                limit: Some(1000),
                commitment: Some(CommitmentConfig::confirmed()),
            };

            // Process new transactions
            if let Ok(signatures) = self
                .rpc_client
                .get_signatures_for_address_with_config(&self.program_id, config)
            {
                for sig_info in signatures.iter().rev() {
                    let signature = Signature::from_str(&sig_info.signature)?;
                    if let Ok(tx) = self
                        .rpc_client
                        .get_transaction(&signature, UiTransactionEncoding::Json)
                    {
                        if let Some(meta) = tx.transaction.meta {
                            if let OptionSerializer::Some(logs) = meta.log_messages {
                                for log in logs {
                                    if let Some(data) = log.strip_prefix("Program data: ") {
                                        if let Ok(decoded) = general_purpose::STANDARD.decode(data)
                                        {
                                            // Handle Mint event
                                            if decoded.starts_with(&mint_discriminator) {
                                                if let Ok(event) =
                                                    MintEvent::try_from_slice(&decoded[8..])
                                                {
                                                    self.handler
                                                        .handle_mint(
                                                            sig_info.slot,
                                                            sig_info.signature.clone(),
                                                            event.to.to_string(),
                                                            event.value,
                                                        )
                                                        .await;
                                                }
                                            }
                                            // Handle Burn event
                                            else if decoded.starts_with(&burn_discriminator) {
                                                if let Ok(event) =
                                                    BurnEvent::try_from_slice(&decoded[8..])
                                                {
                                                    self.handler
                                                        .handle_burn(
                                                            sig_info.slot,
                                                            sig_info.signature.clone(),
                                                            event.from.to_string(),
                                                            event.btc_addr,
                                                            event.value,
                                                            event.operator_id,
                                                        )
                                                        .await;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(last) = signatures.first() {
                    self.last_signature = Some(Signature::from_str(&last.signature)?);
                }
            }

            // Wait before next polling cycle
            sleep(Duration::from_secs(1)).await;
        }
    }
}
