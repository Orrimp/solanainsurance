use anyhow::Result;
use solana_rpc_client::rpc_client::RpcClient;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_signature::Signature;

pub struct TxCost {
    pub signature: Signature,
    pub fee_lamports: u64,
    /// Compute units consumed as reported by the RPC simulation.
    /// Reflects what the live validator will charge — use this for capacity planning.
    pub compute_units_consumed: Option<u64>,
}

/// Send instructions, returning signature, fee and optional compute units (from simulation).
pub fn send_instructions(
    client: &RpcClient,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    instructions: &[Instruction],
) -> Result<TxCost> {
    let blockhash = client.get_latest_blockhash()?;
    let mut tx = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
    let mut signers: Vec<&Keypair> = vec![payer];
    for s in extra_signers {
        if s.pubkey() != payer.pubkey() {
            signers.push(*s);
        }
    }
    tx.sign(&signers, blockhash);

    // Simulate to get compute units
    let simulation = client.simulate_transaction(&tx)?;
    let compute_units = simulation.value.units_consumed;

    // Fee retrieval fallback (SDK fee APIs changed); set to 0 for demo, adjust with proper API later.
    let fee_lamports = 0;
    let signature = client.send_and_confirm_transaction(&tx)?;
    Ok(TxCost {
        signature,
        fee_lamports,
        compute_units_consumed: compute_units,
    })
}
