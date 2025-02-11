use anchor_client::solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signature},
};

use anyhow::Result;
use async_trait::async_trait;

use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

use crate::{utils, TransactionEvent};

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
        loop {
            let config = GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: self.last_signature,
                limit: Some(1000),
                commitment: Some(CommitmentConfig::confirmed()),
            };

            if let Ok(signatures) = self
                .rpc_client
                .get_signatures_for_address_with_config(&self.program_id, config)
            {
                for sig_info in signatures.iter().rev() {
                    if let Ok(tx) = self.rpc_client.get_transaction(
                        &Signature::from_str(&sig_info.signature)?,
                        UiTransactionEncoding::Json,
                    ) {
                        if let Ok(Some(event)) = utils::parse_transaction_event(&tx) {
                            match event {
                                TransactionEvent::Mint(mint_event) => {
                                    self.handler
                                        .handle_mint(
                                            sig_info.slot,
                                            sig_info.signature.clone(),
                                            mint_event.to,
                                            mint_event.value,
                                        )
                                        .await;
                                }
                                TransactionEvent::Burn(burn_event) => {
                                    self.handler
                                        .handle_burn(
                                            sig_info.slot,
                                            sig_info.signature.clone(),
                                            burn_event.from,
                                            burn_event.btc_addr,
                                            burn_event.value,
                                            burn_event.operator_id,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                }

                if let Some(last) = signatures.first() {
                    self.last_signature = Some(Signature::from_str(&last.signature)?);
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    }
}
