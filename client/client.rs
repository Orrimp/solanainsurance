use solana_rpc_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::Instruction;
use solana_address::Address as Pubkey;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::{str::FromStr, thread::sleep, time::Duration};

fn main() {
    // Resolve program id: PROGRAM_ID env var, then deploy keypair, then panic.
    let program_id = std::env::var("PROGRAM_ID")
        .ok()
        .and_then(|s| Pubkey::from_str(&s).ok())
        .unwrap_or_else(|| {
            let path = "target/deploy/insurance-keypair.json";
            solana_keypair::read_keypair_file(path)
                .unwrap_or_else(|e| panic!("Cannot read {path}: {e}.  Run `cargo build-sbf` first."))
                .pubkey()
        });
    println!("Using Program ID: {}", program_id);

    // Connect to local validator
    let rpc_url = String::from("http://localhost:8899");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // Generate a new keypair for the payer (airdropped below)
    let payer = Keypair::new();

    // Airdrop 1 SOL to payer and confirm
    let airdrop_amount = 1_000_000_000_u64; // 1 SOL
    let sig = client
        .request_airdrop(&payer.pubkey(), airdrop_amount)
        .expect("Failed to request airdrop");
    // Poll for balance until the airdrop lands
    let mut attempts = 0u32;
    loop {
        let bal = client
            .get_balance(&payer.pubkey())
            .expect("Failed to fetch balance");
        if bal >= airdrop_amount {
            println!("Airdrop confirmed. Balance: {} lamports", bal);
            break;
        }
        attempts += 1;
        if attempts % 10 == 0 {
            println!("Waiting for airdrop confirmation... (attempt {}), sig={}", attempts, sig);
        }
        sleep(Duration::from_millis(300));
    }

    // Prepare empty instruction data (no payload)
    let data: Vec<u8> = Vec::new();
    let instruction = Instruction {
        program_id,
        accounts: vec![],
        data,
    };

    // Create and sign transaction
    let recent_blockhash = client.get_latest_blockhash().expect("blockhash");
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer], recent_blockhash);

    // Send and confirm the transaction
    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => println!("Transaction Signature: {}", signature),
        Err(err) => eprintln!("Error sending transaction: {}", err),
    }
}