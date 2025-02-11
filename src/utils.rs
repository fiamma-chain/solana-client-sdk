use anchor_client::anchor_lang::{AnchorDeserialize, Discriminator};
use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use bitvm_bridge::events::{BurnEvent, MintEvent};
use solana_transaction_status::{
    option_serializer::OptionSerializer, EncodedConfirmedTransactionWithStatusMeta,
};

use crate::{BurnEventData, MintEventData, TransactionEvent};

pub fn parse_transaction_event(
    tx: &EncodedConfirmedTransactionWithStatusMeta,
) -> Result<Option<TransactionEvent>> {
    if let Some(meta) = &tx.transaction.meta {
        if let OptionSerializer::Some(logs) = &meta.log_messages {
            let mint_discriminator = MintEvent::discriminator();
            let burn_discriminator = BurnEvent::discriminator();

            for log in logs {
                if let Some(data) = log.strip_prefix("Program data: ") {
                    if let Ok(decoded) = general_purpose::STANDARD.decode(data) {
                        if decoded.starts_with(&mint_discriminator) {
                            if let Ok(event) = MintEvent::try_from_slice(&decoded[8..]) {
                                return Ok(Some(TransactionEvent::Mint(MintEventData {
                                    to: event.to.to_string(),
                                    value: event.value,
                                })));
                            }
                        } else if decoded.starts_with(&burn_discriminator) {
                            if let Ok(event) = BurnEvent::try_from_slice(&decoded[8..]) {
                                return Ok(Some(TransactionEvent::Burn(BurnEventData {
                                    from: event.from.to_string(),
                                    btc_addr: event.btc_addr,
                                    value: event.value,
                                    operator_id: event.operator_id,
                                })));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
