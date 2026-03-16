use solana_rpc_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_address::Address as Pubkey;
use solana_keypair::Keypair;
use solana_signer::Signer;
use std::{env, str::FromStr, thread::sleep, time::Duration};

pub fn resolve_program_id() -> Pubkey {
    // 1. Explicit override via environment variable.
    if let Some(pk) = env::var("PROGRAM_ID").ok().and_then(|s| Pubkey::from_str(&s).ok()) {
        return pk;
    }
    // 2. Derive from the deploy keypair so the client always targets the program
    //    that was last built, regardless of which validator run it lives on.
    let keypair_path = "target/deploy/insurance-keypair.json";
    solana_keypair::read_keypair_file(keypair_path)
        .unwrap_or_else(|e| panic!("Cannot read {keypair_path}: {e}.  Run `cargo build-sbf` first."))
        .pubkey()
}

pub fn create_client() -> RpcClient {
    RpcClient::new_with_commitment("http://localhost:8899".to_string(), CommitmentConfig::confirmed())
}

pub fn airdrop_sol(client: &RpcClient, recipient: &Keypair, amount: u64) -> anyhow::Result<u64> {
    let sig = client.request_airdrop(&recipient.pubkey(), amount)?;
    let mut attempts = 0u32;
    loop {
        let bal = client.get_balance(&recipient.pubkey())?;
        if bal >= amount { println!("Airdrop confirmed for {} balance={}", recipient.pubkey(), bal); return Ok(bal); }
        attempts += 1;
        if attempts % 10 == 0 { println!("Waiting for airdrop (attempt {}) sig={}", attempts, sig); }
        sleep(Duration::from_millis(250));
    }
}
