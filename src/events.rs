use anchor_client::solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use anchor_client::{
    solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig},
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
};

use anyhow::Result;
use async_trait::async_trait;
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

use crate::{utils, TransactionEvent};

/// Event handler trait for processing bridge events
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_mint(
        &self,
        tx_slot: u64,
        tx_signature: &str,
        to: &str,
        value: u64,
    ) -> Result<()>;
    async fn handle_burn(
        &self,
        tx_slot: u64,
        tx_signature: &str,
        from: &str,
        btc_addr: &str,
        value: u64,
        operator_id: u64,
    ) -> Result<()>;
}

/// Monitor for bridge events
pub struct EventMonitor {
    program_id: Pubkey,
    handler: Box<dyn EventHandler>,
    rpc_client: RpcClient,
    last_signature: Option<Signature>,
    query_interval: u64,
}

impl EventMonitor {
    pub fn new(
        rpc_url: &str,
        program_id: &str,
        handler: Box<dyn EventHandler>,
        last_signature: Option<String>,
        query_interval: u64,
    ) -> anyhow::Result<Self> {
        let program_id = Pubkey::from_str(program_id)?;
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let last_signature = if let Some(s) = last_signature {
            Some(Signature::from_str(&s)?)
        } else {
            None
        };
        Ok(Self {
            program_id,
            handler,
            rpc_client,
            last_signature,
            query_interval,
        })
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
                    if let Ok(tx) = self.rpc_client.get_transaction_with_config(
                        &Signature::from_str(&sig_info.signature)?,
                        RpcTransactionConfig {
                            encoding: Some(UiTransactionEncoding::Json),
                            commitment: Some(CommitmentConfig::confirmed()),
                            max_supported_transaction_version: Some(0),
                        },
                    ) {
                        if let Ok(Some(event)) = utils::parse_transaction_event(&tx) {
                            match event {
                                TransactionEvent::Mint(mint_event) => {
                                    self.handler
                                        .handle_mint(
                                            sig_info.slot,
                                            &sig_info.signature,
                                            &mint_event.to,
                                            mint_event.value,
                                        )
                                        .await?;
                                }
                                TransactionEvent::Burn(burn_event) => {
                                    self.handler
                                        .handle_burn(
                                            sig_info.slot,
                                            &sig_info.signature,
                                            &burn_event.from,
                                            &burn_event.btc_addr,
                                            burn_event.value,
                                            burn_event.operator_id,
                                        )
                                        .await?;
                                }
                            }
                        }
                    }
                }

                if let Some(last) = signatures.first() {
                    self.last_signature = Some(Signature::from_str(&last.signature)?);
                }
            }

            sleep(Duration::from_secs(self.query_interval)).await;
        }
    }
}
