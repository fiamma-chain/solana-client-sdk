use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
        system_program,
    },
    Client, Cluster, Program,
};
use anchor_spl::{
    associated_token::{get_associated_token_address, spl_associated_token_account},
    token::spl_token,
};
use anyhow::Result;
use bitvm_bridge::{accounts, instruction as bridge_instruction, state::BridgeState};
use solana_sdk::{signature::Signature, transaction::Transaction};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::{rc::Rc, str::FromStr};

pub mod events;

pub struct BitvmBridgeClient {
    bitvm_bridge_program: Program<Rc<Keypair>>,
    btc_light_client_program: Program<Rc<Keypair>>,
    payer: Rc<Keypair>,
}

impl BitvmBridgeClient {
    pub fn new(
        url: String,
        bitvm_bridge_contract: String,
        btc_light_client_contract: String,
        payer: Keypair,
    ) -> Result<Self> {
        let payer = Rc::new(payer);
        let cluster = Cluster::Custom(url.clone(), url.clone());
        let client =
            Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
        let bitvm_bridge_program = client.program(Pubkey::from_str(&bitvm_bridge_contract)?)?;
        let btc_light_client_program =
            client.program(Pubkey::from_str(&btc_light_client_contract)?)?;

        Ok(Self {
            bitvm_bridge_program,
            btc_light_client_program,
            payer,
        })
    }

    pub async fn mint_tokens(
        &self,
        recipient: String,
        tx_id: [u8; 32],
        amount: u64,
    ) -> Result<String> {
        // Get bridge state PDA
        let (bridge_state, _) =
            Pubkey::find_program_address(&[b"bridge_state"], &self.bitvm_bridge_program.id());

        // Fetch and deserialize bridge state data
        let bridge_state_data = self
            .bitvm_bridge_program
            .account::<BridgeState>(bridge_state)
            .await?;

        let mint_account = bridge_state_data.mint_account;
        let recipient = Pubkey::from_str(&recipient)?;

        // Get associated token account for recipient
        let ata = get_associated_token_address(&recipient, &mint_account);

        // Get tx minted state PDA
        let (tx_minted_state, _) = Pubkey::find_program_address(
            &[b"tx_minted_state", &tx_id.to_vec()],
            &self.bitvm_bridge_program.id(),
        );

        // Get tx verified state PDA if verification is required
        let tx_verified_state = if bridge_state_data.skip_tx_verification {
            None
        } else {
            let (tx_verified_state, _) = Pubkey::find_program_address(
                &[b"tx_verified_state", &tx_id.to_vec()],
                &self.btc_light_client_program.id(),
            );
            Some(tx_verified_state)
        };

        let accounts = accounts::MintToken {
            mint_authority: self.payer.pubkey(),
            recipient,
            mint_account,
            associated_token_account: ata,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            system_program: system_program::ID,
            bridge_state,
            tx_minted_state,
            tx_verified_state,
        };

        // Send mint instruction
        let signature = self
            .bitvm_bridge_program
            .request()
            .accounts(accounts)
            .args(bridge_instruction::Mint { tx_id, amount })
            .signer(&*self.payer)
            .send()
            .await?;

        Ok(signature.to_string())
    }

    pub async fn burn_tokens(
        &self,
        amount: u64,
        btc_addr: String,
        operator_id: u64,
    ) -> Result<String> {
        // Get bridge state PDA
        let (bridge_state, _) =
            Pubkey::find_program_address(&[b"bridge_state"], &self.bitvm_bridge_program.id());

        // Fetch and deserialize bridge state data
        let bridge_state_data = self
            .bitvm_bridge_program
            .account::<BridgeState>(bridge_state)
            .await?;

        let mint_account = bridge_state_data.mint_account;
        let ata = get_associated_token_address(&self.payer.pubkey(), &mint_account);

        let accounts = accounts::BurnToken {
            authority: self.payer.pubkey(),
            mint_account,
            associated_token_account: ata,
            token_program: spl_token::ID,
            bridge_state,
        };

        // Send burn instruction
        let signature = self
            .bitvm_bridge_program
            .request()
            .accounts(accounts)
            .args(bridge_instruction::Burn {
                amount,
                btc_addr,
                operator_id,
            })
            .signer(&*self.payer)
            .send()
            .await?;

        Ok(signature.to_string())
    }

    pub async fn get_transaction(
        &self,
        signature: &str,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let signature = Signature::from_str(signature)?;
        let tx = self
            .bitvm_bridge_program
            .rpc()
            .get_transaction(&signature, UiTransactionEncoding::Json)?;
        Ok(tx)
    }
}
