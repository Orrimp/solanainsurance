use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::{Keypair, Signer}};
use std::{env, str::FromStr, thread::sleep, time::Duration};

pub fn resolve_program_id() -> Pubkey {
    env::var("PROGRAM_ID")
        .ok()
        .and_then(|s| Pubkey::from_str(&s).ok())
        .unwrap_or_else(|| Pubkey::from_str("B4yfzKC4NsUsYCetguU7tiewFWi8EDrQA9fEJFiYagVw").expect("valid fallback"))
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
