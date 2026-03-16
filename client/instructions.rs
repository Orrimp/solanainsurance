//! Instruction builders for the Insurance (Pension) program.
//!
//! Pure functions: no side effects, no keypair references. Each builder mirrors the
//! account ordering defined in the processor exactly — changing one requires updating
//! the other.

use borsh::to_vec;
use solana_instruction::{AccountMeta, Instruction};
use solana_address::Address as Pubkey;
use solana_system_interface::program as system_program;
use insurance::instructions::PensionInstruction;
use insurance::state::PensionMetaData;

/// Build an [`PensionInstruction::InitializePensioner`] instruction.
///
/// Creates a new, rent-exempt pension account in `PrePension` status.
///
/// # Accounts (ordered)
/// 0. `[writable, signer]` `pension_account` — new keypair to hold on-chain state
/// 1. `[signer]` `authority` — insurance company / DAO keypair
/// 2. `[]` system program
///
/// # Arguments
/// * `date_of_birth` — Unix timestamp of the pensioner's birth; **must be > 0**.
/// * `date_of_retirement` — Expected retirement timestamp; pass `0` if unknown.
/// * `metadata` — Pension plan configuration; pass `PensionMetaData::default()` if not yet set.
/// * `spouse` — Spouse's account pubkey; pass `Pubkey::default()` if none.
pub fn initialize_pensioner(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    pensioner_pubkey: Pubkey,
    monthly_payment: u64,
    date_of_birth: i64,
    date_of_retirement: i64,
    metadata: PensionMetaData,
    spouse: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::InitializePensioner {
        pensioner_pubkey,
        monthly_payment,
        date_of_birth,
        date_of_retirement,
        metadata,
        spouse,
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

/// Build a [`PensionInstruction::Contribute`] instruction.
///
/// Transfers `lamports` into the pension account via CPI and optionally records pension
/// points for `year`. Pass `points = 0` for a lamports-only contribution; pass
/// `lamports = 0` for a points-only contribution.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account` — target pension account
/// 1. `[signer]`   `authority`       — insurance authority (contributor)
/// 2. `[]`         system program    — required for CPI lamport transfer
///
/// # Arguments
/// * `lamports` — SOL amount in lamports to transfer; pass `0` for points-only.
/// * `points`   — Pension points to add for `year`; pass `0` for lamports-only.
/// * `year`     — Contribution year, used only when `points > 0`.
///
/// # Errors
/// - `InvalidAuthority` when the contributor is not the registered authority.
/// - `YearAlreadyExists` if a points entry for `year` already exists.
/// - `PointsCapacityExceeded` when the 64-entry points array is full.
pub fn contribute(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    lamports: u64,
    points: u64,
    year: u16,
) -> Instruction {
    let ix_data = PensionInstruction::Contribute { lamports, points, year };
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction { program_id, accounts, data }
}

/// Build a [`PensionInstruction::GetAllPoints`] instruction.
///
/// Logs the complete pension-points history to the transaction log.
/// This instruction is read-only — it mutates no on-chain state.
///
/// # Accounts (ordered)
/// 0. `[readonly]` `pension_account` — source pension account
pub fn get_all_points(
    program_id: Pubkey,
    pension_account: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::GetAllPoints;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![AccountMeta::new_readonly(pension_account, false)];
    Instruction { program_id, accounts, data }
}

/// Build a [`PensionInstruction::StartPayoutPeriod`] instruction.
///
/// Enables the payout phase **without** setting a payout recipient, distinguishing
/// it from [`start_payout`] which additionally sets `payout_recipient`.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account` — pension account to update
/// 1. `[signer]`   `authority`       — insurance authority
///
/// # Errors
/// - `InvalidAuthority` when the signer is not the registered authority.
/// - `PayoutAlreadyActive` when `payout_enabled` is already `1`.
pub fn start_payout_period(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::StartPayoutPeriod;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
    ];
    Instruction { program_id, accounts, data }
}

/// Build a [`PensionInstruction::WithdrawMonthly`] instruction.
///
/// Deducts one month's payment from `total_contributions_lamports` and updates
/// `last_payment_timestamp`. Requires **dual signatures**: the authority and the
/// pensioner must both sign.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account` — pension account to debit
/// 1. `[signer]`   `authority`       — insurance authority
/// 2. `[signer]`   `pensioner`       — pensioner's wallet keypair
///
/// # Errors
/// - `InvalidAuthority` when index 1 is not the registered authority.
/// - `PayoutNotActive` when `payout_enabled == 0`.
/// - `PensionerNotActive` when the account status is not `Active`.
pub fn withdraw_monthly(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    pensioner: Pubkey,
) -> Instruction {
    let ix_data = PensionInstruction::WithdrawMonthly;
    let data = to_vec(&ix_data).expect("Serialize PensionInstruction");
    let accounts = vec![
        AccountMeta::new(pension_account, false),
        AccountMeta::new(authority, true),
        AccountMeta::new_readonly(pensioner, true),
    ];
    Instruction { program_id, accounts, data }
}
