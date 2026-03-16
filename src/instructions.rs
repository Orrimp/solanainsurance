//! Instruction types for the pension insurance program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::PensionMetaData;

/// Instructions that the pension insurance program can execute
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PensionInstruction {
    /// Initialize a new pension account for a pensioner.
    ///
    /// The account is created in `PrePension` status. The authority must transition it
    /// to `Active` via a separate instruction before payments can be processed.
    ///
    /// # Accounts expected
    /// 0. `[writable, signer]` Pension account to initialize (new, unallocated keypair)
    /// 1. `[signer]` Authority (insurance company/DAO)
    /// 2. `[]` System program
    ///
    /// # Errors
    /// Returns [`crate::errors::PensionError::InvalidDateOfBirth`] if `date_of_birth` is 0.
    InitializePensioner {
        /// The public key of the pensioner's wallet.
        pensioner_pubkey: Pubkey,
        /// Initial monthly payment amount in lamports.
        monthly_payment: u64,
        /// Pensioner's date of birth as a Unix timestamp. **Must be > 0.**
        date_of_birth: i64,
        /// Expected retirement date as a Unix timestamp. Pass `0` if not yet determined.
        date_of_retirement: i64,
        /// Pension plan configuration. Pass `PensionMetaData::default()` if not yet configured.
        metadata: PensionMetaData,
        /// Pensioner's spouse account. Pass `Pubkey::default()` if none.
        spouse: Pubkey,
    },

    /// Mark a pensioner as deceased to stop payments
    /// 
    /// Accounts expected:
    /// 0. `[writable]` Pension account to update
    /// 1. `[signer]` Authority (must match the authority in the pension account)
    MarkDeceased,

    /// Calculate and log the payment currently due to the pensioner
    /// 
    /// Accounts expected:
    /// 0. `[]` Pension account (read-only)
    CalculateDuePayment,

    /// Add pension points for a specific year and a specific month
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pension account
    /// 1. `[signer]` Authority
    AddPoints {
        /// The year to which the points are being added
        year: u16,

        /// The month to which the points are being added (1-12)
        month: u16,

        /// The number of points to add
        points: u64
    },

    /// Retrieve pension points for a year (logs them)
    ///
    /// Accounts expected:
    /// 0. `[read]` Pension account
    GetPoints { year: u16 },


    /// Retrieve pension points for the whole history (logs them)
    /// 
    /// Accounts expected:
    /// 0. `[read]` Pension account
    GetAllPoints,

    /// Start payout to a recipient address
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pension account
    /// 1. `[signer]` Authority
    /// 2. `[]` Recipient (readonly for validation)
    StartPayout { recipient: Pubkey },

    /// Stop payout
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pension account
    /// 1. `[signer]` Authority
    StopPayout,

    /// Change payout recipient (must be active payout)
    ///
    /// Accounts expected:
    /// 0. `[writable]` Pension account
    /// 1. `[signer]` Authority
    /// 2. `[]` New recipient
    ChangePayoutRecipient { new_recipient: Pubkey },

    /// Recalculate monthly payment: new_monthly = base_lamports + total_points * point_multiplier_lamports
    /// Accounts expected:
    /// 0. `[writable]` Pension account
    /// 1. `[signer]` Authority
    RecalculateMonthlyFromPoints { base_lamports: u64, point_multiplier_lamports: u64 },
    /// Contribute funds and optionally points in a single instruction
    /// Accounts: 0. [writable] Pension account 1. [signer] Contributor (authority or org) 2. [] System program
    Contribute { lamports: u64, points: u64, year: u16 },
    /// Begin payout period (enable payouts)
    /// Accounts: 0. [writable] Pension account 1. [signer] Authority
    StartPayoutPeriod,
    /// Withdraw a monthly payment from contributed balance (simulated payout)
    /// Accounts: 0. [writable] Pension account 1. [signer] Authority 2. [signer] Pensioner
    WithdrawMonthly,
}