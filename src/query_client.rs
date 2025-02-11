use anchor_client::{
    solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig},
    solana_sdk::commitment_config::CommitmentConfig,
};

use anyhow::Result;

use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::str::FromStr;

use crate::{utils, TransactionEvent};

pub struct QueryClient {
    rpc_client: RpcClient,
}

impl QueryClient {
    pub fn new(url: String) -> Result<Self> {
        Ok(Self {
            rpc_client: RpcClient::new(url),
        })
    }

    pub async fn get_transaction(
        &self,
        signature: &str,
    ) -> anyhow::Result<EncodedConfirmedTransactionWithStatusMeta> {
        let signature = Signature::from_str(signature)?;
        let tx = self.rpc_client.get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )?;
        Ok(tx)
    }

    pub async fn parse_transaction_event(
        &self,
        signature: &str,
    ) -> anyhow::Result<Option<TransactionEvent>> {
        let tx = self.get_transaction(signature).await?;
        utils::parse_transaction_event(&tx)
    }
}
