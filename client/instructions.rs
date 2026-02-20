//! Instruction builders for the Insurance (Pension) program
//! Pure functions: no side effects, no keypair references.

use borsh::to_vec;
use solana_sdk::{instruction::{AccountMeta, Instruction}, pubkey::Pubkey, system_program};
use insurance::instructions::PensionInstruction;

/// Build InitializePensioner instruction.
/// Accounts (ordered):
/// 0. [writable, signer] pension account (new account to hold state)
/// 1. [signer] authority (insurance/DAO)
/// 2. [] system program
pub fn initialize_pensioner(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    pensioner_pubkey: Pubkey,
    monthly_payment: u64,
) -> Instruction {
    let ix_data = PensionInstruction::InitializePensioner {
        pensioner_pubkey,
        monthly_payment,
    };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, true),
        AccountMeta::new_readonly(authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction { program_id, accounts, data }
}

/// Build MarkDeceased instruction.
/// Accounts: 0. [writable] pension account, 1. [signer] authority
pub fn mark_deceased(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::MarkDeceased;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
    ];
    Instruction { program_id, accounts, data }
}

/// Build CalculateDuePayment instruction.
/// Accounts: 0. [readonly] pension account
pub fn calculate_due_payment(
    program_id: Pubkey,
    pension_account: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::CalculateDuePayment;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![AccountMeta::new_readonly(pension_account, false)];
    Instruction { program_id, accounts, data }
}

pub fn add_year_points(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    year: u16,
    points: u64,
    month: u16,
) -> Instruction {
    let ix_data = PensionInstruction::AddPoints { year, month, points};
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
    ];
    Instruction { program_id, accounts, data }
}

pub fn get_year_points(
    program_id: Pubkey,
    pension_account: Pubkey,
    year: u16,
) -> Instruction {
    let ix_data = PensionInstruction::GetPoints { year };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![AccountMeta::new_readonly(pension_account, false)];
    Instruction { program_id, accounts, data }
}

pub fn start_payout(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    recipient: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::StartPayout { recipient };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
        AccountMeta::new_readonly(recipient, false),
    ];
    Instruction { program_id, accounts, data }
}

pub fn stop_payout(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::StopPayout;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
    ];
    Instruction { program_id, accounts, data }
}

pub fn change_payout_recipient(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    new_recipient: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::ChangePayoutRecipient { new_recipient };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
        AccountMeta::new_readonly(new_recipient, false),
    ];
    Instruction { program_id, accounts, data }
}

pub fn recalculate_monthly_from_points(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    base_lamports: u64,
    point_multiplier_lamports: u64,
) -> Instruction {
    let ix_data = PensionInstruction::RecalculateMonthlyFromPoints { base_lamports, point_multiplier_lamports };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
    ];
    Instruction { program_id, accounts, data }
}
