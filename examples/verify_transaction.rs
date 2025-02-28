use anyhow::{anyhow, Result};
use solana_client_sdk::bridge_client::BitvmBridgeClient;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    // Get private key from environment variables
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .map_err(|_| anyhow!("SOLANA_PRIVATE_KEY not found in environment"))?;

    // Set connection parameters
    let url = "https://api.devnet.solana.com";
    let bitvm_bridge_program_id = "J64ucfNboe9e3FtoeSxMzmkobfTxEkJgiLwY4nYgQLBe";
    let btc_light_client_program_id = "8dCJMu6sum1A3W2R6iB1gHus45ayrnRmChSfGus4rRsL";

    // Create client instance
    let client = BitvmBridgeClient::new(
        url,
        bitvm_bridge_program_id,
        btc_light_client_program_id,
        &private_key,
    )?;

    // Bitcoin transaction verification parameters
    // Note: These are example values, replace with actual values
    let block_height = 71883; // Bitcoin block height

    // Block header (80 bytes)
    let block_header_hex = "00000020ac4051ad1135646ec3b65ab93bfcb42623f10f2861181b46043c0000000000002040291045b1a55e6db13db7dbd8c3147bf2d8f583b5381b0293bd62c7532f632666c067ffff001d266f87e3";
    let block_header = hex::decode(block_header_hex)?;

    // Transaction ID (32 bytes)
    let tx_id_hex = "444fc273af34a87424a4b72c10b6393b0353505049128c5cb383da0358ab0b58";
    let mut tx_id: [u8; 32] = hex::decode(tx_id_hex)?
        .try_into()
        .map_err(|_| anyhow!("Invalid tx_id length"))?;
    tx_id.reverse();

    // Transaction index in the block
    let tx_index = 7;

    // Merkle proof (each element is 32 bytes)
    let merkle_proof_hex = "7c71c06b93661123517e2c212e71c4b94995cd695d02138a58af26d59095a29639e334b03828f792713b73abf3d3ed0c7202224e390f8d2f6b93f22dca85ea0b81c84af181d04680ade41b5dc3006876a3bf9fe4bf21193ffef1f0124e461ecd5c796b6c9707e804882e136418bcce72f005f59f8fa1bea691bf3738e77ab27830ecfe6fab2e947c9e452b53fc29773bb6e2f9d2c68c494b3d8f34ecd1b3dde4cf735b17a31639e35b885201a60d955aa1a83b057ba5fc59e6c5c27d2f90752da6c10f67250e44003cf0f554fa71cfa5e1aa4a6c1b676f422474dfd185afe378d2efbbda3e25ff85145d37b6fb3c51cfc9e3aedeca7ffe87e500cff2d55944acf1fb3c918c3fb6ced073e824323d02c78533780f15c49ce124dcc69f31b7260b2fd97ba6b3aed1a550db3e5418804fb8ce812b23e10fed70a65c29907c1ec9f7108d669f898dc0926043d8314d199e89f1113463e57997b7fe86212dbb735935112d761bb89f9e69ecd0d1471032d964847fc3a0318d07ce36f09e6537a3011e";
    let merkle_bytes = hex::decode(&merkle_proof_hex)?;
    let mut merkle_path = Vec::new();
    for chunk in merkle_bytes.chunks(32) {
        if chunk.len() == 32 {
            let mut hash: [u8; 32] = chunk.try_into().unwrap();
            hash.reverse();
            merkle_path.push(hash);
        }
    }

    // Raw transaction data
    let raw_tx_hex = "020000000196a29590d526af588a13025d69cd9549b9c4712e212c7e51231166936bc0717c0100000000ffffffff0220a10700000000002200204ee81665c8c767c0dfe02e6d7c1b446ab01e84dc29b4c3634bc5daae24b52cbdca906d3500000000225120418ac1703f758fe750fecd897aac19c65cf41aeb58520b564316b3c02051305b00000000";
    let raw_tx = hex::decode(raw_tx_hex)?;

    // Output index
    let output_index = 0;

    // Expected amount (in satoshis)
    let expected_amount = 500000; // 0.005 BTC

    // Expected script hash (32 bytes)
    let expected_script_hash_hex =
        "4ee81665c8c767c0dfe02e6d7c1b446ab01e84dc29b4c3634bc5daae24b52cbd";
    let expected_script_hash: [u8; 32] = hex::decode(expected_script_hash_hex)?
        .try_into()
        .map_err(|_| anyhow!("Invalid expected_script_hash length"))?;

    println!("Verifying Bitcoin transaction: {}", tx_id_hex);
    println!("Block height: {}", block_height);
    println!("Transaction index: {}", tx_index);

    // Call verify transaction method
    let result = client
        .verify_transaction(
            block_height,
            &block_header,
            tx_id,
            tx_index,
            merkle_path,
            &raw_tx,
            output_index,
            expected_amount,
            expected_script_hash,
        )
        .await?;

    println!("Transaction verification submitted successfully!");
    println!("Signature: {}", result);

    // Wait for transaction confirmation
    println!("Waiting for transaction confirmation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check verification status
    let verification_status = client.get_tx_verification_status(tx_id).await?;
    println!(
        "Transaction verification status: {}",
        if verification_status {
            "Verified"
        } else {
            "Not verified"
        }
    );

    // If transaction is not verified, poll for status
    if !verification_status {
        println!("Transaction not yet verified. Polling for status...");

        let mut retries = 5;
        while retries > 0 && !client.get_tx_verification_status(tx_id).await? {
            println!(
                "Still waiting for verification... {} attempts left",
                retries
            );
            retries -= 1;
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        let final_status = client.get_tx_verification_status(tx_id).await?;
        println!(
            "Final verification status: {}",
            if final_status {
                "Verified"
            } else {
                "Not verified"
            }
        );
    }

    Ok(())
}
