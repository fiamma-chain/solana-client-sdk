use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction,
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

use bitvm_bridge::{accounts, instruction as bridge_instruction, state::BridgeState};
use btc_light_client::state::BtcLightClientState;
use std::{str::FromStr, sync::Arc};

use crate::query_client::QueryClient;

pub struct BitvmBridgeClient {
    query_client: QueryClient,
    payer: Arc<Keypair>,
    bitvm_bridge_program: Program<Arc<Keypair>>,
    btc_light_client_program: Program<Arc<Keypair>>,
}

impl BitvmBridgeClient {
    pub fn new(
        url: &str,
        bitvm_bridge_contract: &str,
        btc_light_client_contract: &str,
        private_key: &str,
    ) -> anyhow::Result<Self> {
        let private_key = bs58::decode(private_key).into_vec()?;
        let payer = Keypair::from_bytes(&private_key)?;
        let payer = Arc::new(payer);
        let cluster = Cluster::Custom(url.to_string(), url.to_string());

        let client =
            Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());

        let bitvm_bridge_program = client.program(Pubkey::from_str(bitvm_bridge_contract)?)?;
        let btc_light_client_program =
            client.program(Pubkey::from_str(btc_light_client_contract)?)?;

        let query_client = QueryClient::new(url.to_string())?;

        Ok(Self {
            query_client,
            payer,
            bitvm_bridge_program,
            btc_light_client_program,
        })
    }

    pub async fn mint_tokens(
        &self,
        recipient: &str,
        tx_id: [u8; 32],
        amount: u64,
    ) -> anyhow::Result<String> {
        let recipient = Pubkey::from_str(recipient)?;
        // Get bridge state PDA
        let (bridge_state, _) =
            Pubkey::find_program_address(&[b"bridge_state"], &self.bitvm_bridge_program.id());

        // Fetch and deserialize bridge state data
        let bridge_state_data = self
            .bitvm_bridge_program
            .account::<BridgeState>(bridge_state)
            .await?;

        let mint_account = bridge_state_data.mint_account;

        // Get associated token account for recipient
        let ata = get_associated_token_address(&recipient, &mint_account);

        // Get tx minted state PDA
        let (tx_minted_state, _) = Pubkey::find_program_address(
            &[b"tx_minted_state", &tx_id],
            &self.bitvm_bridge_program.id(),
        );

        // Get tx verified state PDA if verification is required
        let tx_verified_state = if bridge_state_data.skip_tx_verification {
            None
        } else {
            let (tx_verified_state, _) = Pubkey::find_program_address(
                &[b"tx_verified_state", &tx_id],
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
        let payer = self.payer.clone();
        let signature = self
            .bitvm_bridge_program
            .request()
            .accounts(accounts)
            .args(bridge_instruction::Mint { tx_id, amount })
            .signer(payer)
            .send()
            .await?;

        Ok(signature.to_string())
    }

    pub async fn burn_tokens(
        &self,
        amount: u64,
        btc_addr: &str,
        operator_id: u64,
    ) -> anyhow::Result<String> {
        let btc_addr = btc_addr.to_string();
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
        let payer = self.payer.clone();
        let signature = self
            .bitvm_bridge_program
            .request()
            .accounts(accounts)
            .args(bridge_instruction::Burn {
                amount,
                btc_addr,
                operator_id,
            })
            .signer(payer)
            .send()
            .await?;

        Ok(signature.to_string())
    }

    pub async fn query_latest_block_height(&self) -> anyhow::Result<u64> {
        let (btc_light_client_state, _) = Pubkey::find_program_address(
            &[b"btc_light_client"],
            &self.btc_light_client_program.id(),
        );

        let btc_light_client_state_data = self
            .btc_light_client_program
            .account::<BtcLightClientState>(btc_light_client_state)
            .await?;

        Ok(btc_light_client_state_data.latest_block_height)
    }

    pub async fn query_min_confirmations(&self) -> anyhow::Result<u64> {
        let (btc_light_client_state, _) = Pubkey::find_program_address(
            &[b"btc_light_client"],
            &self.btc_light_client_program.id(),
        );

        let btc_light_client_state_data = self
            .btc_light_client_program
            .account::<BtcLightClientState>(btc_light_client_state)
            .await?;

        Ok(btc_light_client_state_data.min_confirmations)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn verify_transaction(
        &self,
        block_height: u64,
        block_header: &[u8],
        tx_id: [u8; 32],
        tx_index: u32,
        merkle_proof: Vec<[u8; 32]>,
        raw_tx: &[u8],
        output_index: u32,
        expected_amount: u64,
        expected_script_hash: [u8; 32],
    ) -> anyhow::Result<String> {
        // get block_hash_entry PDA
        let (block_hash_entry, _) = Pubkey::find_program_address(
            &[b"block_hash_entry", &block_height.to_le_bytes()],
            &self.btc_light_client_program.id(),
        );

        // get tx_verified_state PDA
        let (tx_verified_state, _) = Pubkey::find_program_address(
            &[b"tx_verified_state", &tx_id],
            &self.btc_light_client_program.id(),
        );

        // build tx proof
        let tx_proof = btc_light_client::instructions::verify_tx::BtcTxProof {
            block_header: block_header.to_vec(),
            tx_id,
            tx_index,
            merkle_proof,
            raw_tx: raw_tx.to_vec(),
            output_index,
            expected_amount,
            expected_script_hash,
        };

        let (btc_light_client_state, _) = Pubkey::find_program_address(
            &[b"btc_light_client"],
            &self.btc_light_client_program.id(),
        );

        // build accounts
        let accounts = btc_light_client::accounts::VerifyTransaction {
            state: btc_light_client_state,
            tx_verified_state,
            payer: self.payer.pubkey(),
            system_program: system_program::ID,
            block_hash_entry,
        };

        let request_compute_units_instruction =
            ComputeBudgetInstruction::set_compute_unit_limit(500000);

        // send verify transaction instruction
        let payer = self.payer.clone();
        let signature = self
            .btc_light_client_program
            .request()
            .instruction(request_compute_units_instruction)
            .accounts(accounts)
            .args(btc_light_client::instruction::VerifyTransaction {
                block_height,
                tx_proof,
            })
            .signer(payer)
            .send()
            .await?;

        Ok(signature.to_string())
    }
    pub async fn get_tx_verification_status(&self, tx_id: [u8; 32]) -> anyhow::Result<bool> {
        // Get bridge state PDA
        let (bridge_state, _) =
            Pubkey::find_program_address(&[b"bridge_state"], &self.bitvm_bridge_program.id());

        // Fetch and deserialize bridge state data
        let bridge_state_data = self
            .bitvm_bridge_program
            .account::<BridgeState>(bridge_state)
            .await?;
        if bridge_state_data.skip_tx_verification {
            return Ok(true);
        }

        let (tx_verified_state, _) = Pubkey::find_program_address(
            &[b"tx_verified_state", &tx_id],
            &self.btc_light_client_program.id(),
        );

        let tx_verified_state_data = self
            .btc_light_client_program
            .account::<btc_light_client::state::TxVerifiedState>(tx_verified_state)
            .await?;

        Ok(tx_verified_state_data.is_verified)
    }
    pub fn validate_solana_address(address: &str) -> anyhow::Result<()> {
        Pubkey::from_str(address)?;
        Ok(())
    }
}

impl std::ops::Deref for BitvmBridgeClient {
    type Target = QueryClient;

    fn deref(&self) -> &Self::Target {
        &self.query_client
    }
}

#[cfg(test)]
mod tests {
    use super::BitvmBridgeClient;
    use crate::query_client::QueryClient;
    use anchor_client::{
        solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
        Client, Cluster,
    };
    use std::{str::FromStr, sync::Arc};

    #[test]
    fn test_is_valid_solana_address() {
        // This is just a mock implementation for unit testing
        // Mock the client dependencies to allow offline testing
        let private_key = Keypair::new();
        let payer = Arc::new(private_key);
        let cluster = Cluster::Custom(
            "http://localhost:8899".to_string(),
            "http://localhost:8899".to_string(),
        );

        let client =
            Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
        let dummy_program_id = "11111111111111111111111111111111"; // System Program ID

        let bitvm_bridge_program = client
            .program(Pubkey::from_str(dummy_program_id).unwrap())
            .unwrap();
        let btc_light_client_program = client
            .program(Pubkey::from_str(dummy_program_id).unwrap())
            .unwrap();

        let query_client = QueryClient::new("http://localhost:8899".to_string()).unwrap();

        let client = BitvmBridgeClient {
            query_client,
            payer,
            bitvm_bridge_program,
            btc_light_client_program,
        };

        // Valid Solana addresses (base58 encoded)
        assert!(client.is_valid_solana_address("11111111111111111111111111111111")); // System Program
        assert!(client.is_valid_solana_address("AgCVMw9PkZqBCGiupLhdgKAcudKcjyUppShJCsJa7fY3")); // Random valid address
        assert!(client.is_valid_solana_address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")); // USDC mint

        // Invalid Solana addresses
        assert!(!client.is_valid_solana_address("0x0000000000000000000000000000000000000000")); // Ethereum style
        assert!(!client.is_valid_solana_address("not-a-valid-address"));
        assert!(!client.is_valid_solana_address("11111111")); // Too short
        assert!(!client
            .is_valid_solana_address("111111111111111111111111111111111111111111111111111111"));
        // Too long
    }
}
