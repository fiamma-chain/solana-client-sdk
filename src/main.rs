use std::rc::Rc;
use std::str::FromStr; 
use anchor_client::{
    solana_sdk::{
        pubkey::Pubkey, signature::read_keypair_file, signer::Signer, system_program
    },
    Client, Cluster,
};
use anchor_spl::{associated_token::{get_associated_token_address, spl_associated_token_account}, token::spl_token};
use bitvm_bridge::{accounts, instruction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let payer = read_keypair_file("/home/ubuntu/.config/solana/id.json")?;
    let payer = Rc::new(payer);
    let client = Client::new(Cluster::Devnet, payer.clone());

    let mint_token = Pubkey::from_str("HBhPZKQ9axPpbSn4ELExrH5w8fWifeWGzLcb5fvHGVKH")?;

    // Create program
    let program = client.program(bitvm_bridge::ID)?;

    let ata = get_associated_token_address(&payer.pubkey(), &mint_token);

    let accounts = accounts::MintToken {
        mint_authority: payer.pubkey(),
        recipient: payer.pubkey(),
        mint_account: mint_token,
        associated_token_account: ata,
        token_program: spl_token::ID,
        associated_token_program: spl_associated_token_account::ID,
        system_program: system_program::ID,
    };

    // call mint instruction
    program
        .request()
        .accounts(accounts)
        .args(instruction::Mint { amount: 100 })
        .signer(&payer)
        .send()
        .await?;


    println!("mint success!");
    println!("Mint address: {}", payer.pubkey());

    Ok(())
}